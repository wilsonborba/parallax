package com.parallax.receiver.dal.local

import android.content.SharedPreferences

interface SettingsStore {
    fun getScale(): Float
    fun setScale(scale: Float)
    fun getHost(): String
    fun setHost(host: String)
    fun getStreamPort(): Int
    fun setStreamPort(port: Int)
    fun getControlPort(): Int
    fun setControlPort(port: Int)
    fun getAccessPin(): String
    fun setAccessPin(accessPin: String)
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

    override fun getStreamPort(): Int {
        if (sharedPreferences.contains(KEY_STREAM_PORT)) {
            return sharedPreferences.getInt(KEY_STREAM_PORT, DEFAULT_STREAM_PORT)
        }
        return sharedPreferences.getInt(KEY_LEGACY_PORT, DEFAULT_STREAM_PORT)
    }

    override fun setStreamPort(port: Int) {
        sharedPreferences.edit()
            .putInt(KEY_STREAM_PORT, port)
            .apply()
    }

    override fun getControlPort(): Int {
        return sharedPreferences.getInt(KEY_CONTROL_PORT, DEFAULT_CONTROL_PORT)
    }

    override fun setControlPort(port: Int) {
        sharedPreferences.edit()
            .putInt(KEY_CONTROL_PORT, port)
            .apply()
    }

    override fun getAccessPin(): String {
        return sharedPreferences.getString(KEY_ACCESS_PIN, DEFAULT_ACCESS_PIN) ?: DEFAULT_ACCESS_PIN
    }

    override fun setAccessPin(accessPin: String) {
        sharedPreferences.edit()
            .putString(KEY_ACCESS_PIN, accessPin)
            .apply()
    }

    private companion object {
        private const val KEY_SCALE = "settings.scale"
        private const val DEFAULT_SCALE = 1.0f
        private const val KEY_HOST = "settings.host"
        private const val KEY_STREAM_PORT = "settings.stream_port"
        private const val KEY_CONTROL_PORT = "settings.control_port"
        private const val KEY_LEGACY_PORT = "settings.port"
        private const val KEY_ACCESS_PIN = "settings.access_pin"
        private const val DEFAULT_HOST = "127.0.0.1"
        private const val DEFAULT_STREAM_PORT = 5000
        private const val DEFAULT_CONTROL_PORT = 7000
        private const val DEFAULT_ACCESS_PIN = "parallax"
    }
}
