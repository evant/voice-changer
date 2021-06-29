package me.tatarka.voicechanger

import kotlinx.coroutines.asCoroutineDispatcher
import kotlinx.coroutines.withContext
import java.util.concurrent.Executors

class SoundProcessor {
    private var ref: Long? = null
    private val processingThread = Executors.newSingleThreadExecutor().asCoroutineDispatcher()

    suspend fun start(wavelength: Float) {
        withContext(processingThread) {
            if (ref == null) {
                val res = me.tatarka.voicechanger.start(wavelength)
                if (res == 0L) {
                    throw RuntimeException("Failed to start")
                } else {
                    ref = res
                }
            }
        }
    }

    suspend fun setPitch(pitch: Float) {
        withContext(processingThread) {
            if (ref != null) {
                setPitch(ref!!, pitch)
            }
        }
    }

    suspend fun stop() {
        withContext(processingThread) {
            if (ref != null) {
                val res = stop(ref!!)
                if (res == 0L) {
                    throw RuntimeException("Failed to stop")
                } else {
                    ref = null
                }
            }
        }
    }
}

private external fun start(wavelength: Float): Long

private external fun stop(ref: Long): Long

private external fun setPitch(ref: Long, pitch: Float);