package me.tatarka.voicechanger

import android.app.Application

class MyApp : Application() {
    init {
        System.loadLibrary("voice_changer")
    }

}