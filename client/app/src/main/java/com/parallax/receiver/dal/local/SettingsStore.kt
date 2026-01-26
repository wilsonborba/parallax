package com.parallax.receiver.dal.local

import android.content.SharedPreferences

interface SettingsStore {
    fun getScale(): Float
    fun setScale(scale: Float)
}

class SharedPreferencesSettingsStore(
    private val sharedPreferences: SharedPreferences,
) : SettingsStore {
    override fun getScale(): Float {
        return sharedPreferences.getFloat(KEY_SCALE, DEFAULT_SCALE)
    }

    override fun setScale(scale: Float) {
        sharedPreferences.edit()
            .putFloat(KEY_SCALE, scale)
            .apply()
    }

    private companion object {
        private const val KEY_SCALE = "settings.scale"
        private const val DEFAULT_SCALE = 1.0f
    }
}
