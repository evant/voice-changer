package me.tatarka.voicechanger

import android.Manifest
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.material.*
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.PlayArrow
import androidx.compose.material.icons.filled.Stop
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Alignment.Companion.CenterHorizontally
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.focus.focusModifier
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.*
import kotlinx.coroutines.launch

val AppIcons = Icons.Default

class MainActivity : ComponentActivity() {

    private var soundProcessor by mutableStateOf<SoundProcessor?>(null)
    private var started by mutableStateOf(false)
    private var pitch by mutableStateOf(1f)

    private val requestRecordAudio =
        registerForActivityResult(ActivityResultContracts.RequestPermission()) { result ->
            if (result) {
                soundProcessor = SoundProcessor()
            }
        }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            if (soundProcessor != null) {
                val scope = rememberCoroutineScope()
                Main(
                    started = started,
                    pitch = pitch,
                    onStartedChanged = {
                        started = !started
                        scope.launch {
                            if (started) {
                                pitch = 1f
                                soundProcessor?.start(4.0f)
                            } else {
                                soundProcessor?.stop()
                            }
                        }
                    },
                    onPitchChanged = { newPitch ->
                        pitch = newPitch
                        scope.launch {
                            soundProcessor?.setPitch(newPitch)
                        }
                    }
                )
            }
        }
        lifecycle.addObserver(object : LifecycleEventObserver {
            override fun onStateChanged(source: LifecycleOwner, event: Lifecycle.Event) {
                when (event) {
                    Lifecycle.Event.ON_RESUME -> {
                        requestRecordAudio.launch(Manifest.permission.RECORD_AUDIO)
                    }
                    Lifecycle.Event.ON_PAUSE -> {
                        lifecycleScope.launch {
                            started = false
                            soundProcessor?.stop()
                        }
                    }
                }
            }
        })
    }
}

@Composable
fun Main(
    started: Boolean,
    pitch: Float,
    onStartedChanged: () -> Unit,
    onPitchChanged: (Float) -> Unit
) {
    Scaffold {
        Column {
            StartStopButton(
                started = started,
                onClick = onStartedChanged,
                modifier = Modifier
                    .align(CenterHorizontally)
                    .weight(1f)
            )
            PitchSlider(pitch = pitch, onPitchChanged = onPitchChanged)
        }
    }
}

@Preview
@Composable
fun MainPreview() {
    Main(started = false, pitch = 1f, onStartedChanged = {}, onPitchChanged = {})
}

@Composable
fun StartStopButton(started: Boolean, onClick: () -> Unit, modifier: Modifier = Modifier) {
    IconButton(onClick = onClick, modifier = modifier) {
        if (started) {
            Icon(AppIcons.Stop, contentDescription = "Stop")
        } else {
            Icon(AppIcons.PlayArrow, contentDescription = "Start")
        }
    }
}

@Composable
fun PitchSlider(pitch: Float, onPitchChanged: (Float) -> Unit) {
    Column {
        Text(text = "Pitch: $pitch", modifier = Modifier.align(CenterHorizontally))
        Slider(value = pitch, onValueChange = onPitchChanged, valueRange = 0.5f..2f)
    }
}