package com.parallax.receiver.dal.local

import android.content.SharedPreferences

interface SettingsStore {
    fun getScale(): Float
    fun setScale(scale: Float)
    fun getHost(): String
    fun setHost(host: String)
    fun getPort(): Int
    fun setPort(port: Int)
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

    override fun getHost(): String {
        return sharedPreferences.getString(KEY_HOST, DEFAULT_HOST) ?: DEFAULT_HOST
    }

    override fun setHost(host: String) {
        sharedPreferences.edit()
            .putString(KEY_HOST, host)
            .apply()
    }

    override fun getPort(): Int {
        return sharedPreferences.getInt(KEY_PORT, DEFAULT_PORT)
    }

    override fun setPort(port: Int) {
        sharedPreferences.edit()
            .putInt(KEY_PORT, port)
            .apply()
    }

    private companion object {
        private const val KEY_SCALE = "settings.scale"
        private const val DEFAULT_SCALE = 1.0f
        private const val KEY_HOST = "settings.host"
        private const val KEY_PORT = "settings.port"
        private const val DEFAULT_HOST = "127.0.0.1"
        private const val DEFAULT_PORT = 5000
    }
}
