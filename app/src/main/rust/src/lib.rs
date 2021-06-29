use std::{
    intrinsics::transmute,
    io::{Read, Write},
    ptr::slice_from_raw_parts,
    sync::{atomic::AtomicU32, Arc},
};

use aaudio::{AAudioStream, AAudioStreamBuilder, AAudioStreamInfo, CallbackResult};
use jni::{
    objects::JClass,
    sys::{jfloat, jlong},
    JNIEnv,
};
use log::{debug, error, warn};
use ringbuf::RingBuffer;
use tdpsola::{AlternatingHat, Speed, TdpsolaAnalysis, TdpsolaSynthesis};

struct VoiceChanger {
    input_stream: AAudioStream,
    output_stream: AAudioStream,
    pitch: Arc<AtomicU32>,
}

impl VoiceChanger {
    fn start(wavelength: f32) -> Result<VoiceChanger, aaudio::Error> {
        android_logger::init_once(
            android_logger::Config::default().with_min_level(log::Level::Debug),
        );

        let pitch = Arc::new(AtomicU32::new(unsafe { transmute(1.0f32) }));
        let read_pitch = pitch.clone();

        let bufffer = RingBuffer::<u8>::new(4096 * 8);
        let (mut write_buffer, mut read_buffer) = bufffer.split();

        let mut window = AlternatingHat::new(wavelength);
        let mut analysis = TdpsolaAnalysis::new(&window);
        let mut synthesis = TdpsolaSynthesis::new(Speed::from_f32(1.0), 5.0);

        let input_stream_builder = AAudioStreamBuilder::new()?
            .set_direction(aaudio::Direction::Input)
            .set_format(aaudio::Format::F32)
            .set_channel_count(1)
            .set_performance_mode(aaudio::PerformanceMode::LowLatency)
            .set_sharing_mode(aaudio::SharingMode::Exclusive)
            .set_input_preset(aaudio::InputPreset::VoicePerformance)
            .set_sample_rate(48000)
            .set_frames_per_data_callback(96)
            .set_callbacks(
                move |_stream_info, input, _num_frames| {
                    let floats: &[f32] = unsafe {
                        slice_from_raw_parts(input.as_ptr() as *const f32, input.len() / 4).as_ref()
                    }
                    .unwrap();
                    for float in floats {
                        analysis.push_sample(*float, &mut window);
                    }
                    let mut byte_count = 0;
                    let pitch: f32 =
                        unsafe { transmute(read_pitch.load(std::sync::atomic::Ordering::Relaxed)) };
                    synthesis.set_wavelength(wavelength * pitch);
                    for transformed in synthesis.iter(&analysis).take(floats.len()) {
                        let bytes: [u8; 4] = unsafe { transmute(transformed) };
                        if let Err(e) = write_buffer.write_all(&bytes) {
                            error!("read: {}", e);
                            return CallbackResult::Stop;
                        }
                        byte_count += 4;
                    }
                    if byte_count != input.len() {
                        error!(
                            "byte count mismatch: expected:{} but got:{}",
                            input.len(),
                            byte_count
                        );
                        return CallbackResult::Stop;
                    }
                    CallbackResult::Continue
                },
                handle_error,
            );

        let output_stream_builder = AAudioStreamBuilder::new()?
            .set_direction(aaudio::Direction::Output)
            .set_format(aaudio::Format::F32)
            .set_sharing_mode(aaudio::SharingMode::Shared)
            .set_channel_count(1)
            .set_performance_mode(aaudio::PerformanceMode::LowLatency)
            .set_usage(aaudio::Usage::Media)
            .set_sample_rate(48000)
            .set_callbacks(
                move |_stream_info, out, _num_frames| {
                    if let Err(e) = read_buffer.read_exact(out) {
                        error!("write: {}", e);
                        // return CallbackResult::Stop;
                    }
                    CallbackResult::Continue
                },
                handle_error,
            );

        let mut input_stream = input_stream_builder.open_stream()?;
        let mut output_stream = output_stream_builder.open_stream()?;

        input_stream.request_start()?;
        output_stream.request_start()?;

        Ok(VoiceChanger {
            input_stream,
            output_stream,
            pitch,
        })
    }

    fn set_pitch(&mut self, value: f32) {
        self.pitch.store(
            unsafe { transmute(value) },
            std::sync::atomic::Ordering::Relaxed,
        );
    }

    fn stop(mut self) -> Result<(), aaudio::Error> {
        self.input_stream.request_stop()?;
        self.input_stream.release()?;
        self.output_stream.request_stop()?;
        self.output_stream.release()?;
        Ok(())
    }
}

fn handle_error(_stream_info: &AAudioStreamInfo, error: aaudio::Error) {
    error!("{}", error);
}

#[no_mangle]
pub extern "system" fn Java_me_tatarka_voicechanger_SoundProcessorKt_start(
    _env: JNIEnv,
    _class: JClass,
    wavelength: f32,
) -> jlong {
    let processor = VoiceChanger::start(wavelength);
    match processor {
        Ok(processor) => {
            let processor_ref = Box::leak(Box::new(processor));
            processor_ref as *const VoiceChanger as jlong
        }
        Err(e) => {
            error!("{}", e);
            0
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_me_tatarka_voicechanger_SoundProcessorKt_stop(
    _env: JNIEnv,
    _class: JClass,
    processor_ref: jlong,
) -> jlong {
    let processor = unsafe { Box::from_raw(processor_ref as *mut VoiceChanger) };
    match processor.stop() {
        Ok(_) => 1,
        Err(e) => {
            error!("{}", e);
            0
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_me_tatarka_voicechanger_SoundProcessorKt_setPitch(
    _env: JNIEnv,
    _class: JClass,
    processor_ref: jlong,
    pitch: jfloat,
) {
    debug!("processor_ref: {}", processor_ref);
    let processor = unsafe { (processor_ref as *mut VoiceChanger).as_mut().unwrap() };
    processor.set_pitch(pitch);
}
