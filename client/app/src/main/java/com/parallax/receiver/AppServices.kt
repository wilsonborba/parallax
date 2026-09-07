package com.parallax.receiver

import android.content.Context
import com.parallax.receiver.dal.local.SettingsStore
import com.parallax.receiver.dal.local.SharedPreferencesSettingsStore
import com.parallax.receiver.domain.service.StreamSessionService

object AppServices {
    @Volatile
    private var settingsStoreRef: SettingsStore? = null

    @Volatile
    private var streamSessionServiceRef: StreamSessionService? = null

    fun settingsStore(context: Context): SettingsStore {
        val existing = settingsStoreRef
        if (existing != null) return existing
        return synchronized(this) {
            settingsStoreRef ?: SharedPreferencesSettingsStore(
                context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE),
            ).also { settingsStoreRef = it }
        }
    }

    fun streamSessionService(context: Context): StreamSessionService {
        val existing = streamSessionServiceRef
        if (existing != null) return existing
        return synchronized(this) {
            streamSessionServiceRef ?: StreamSessionService(
                settingsStore = settingsStore(context),
            ).also { streamSessionServiceRef = it }
        }
    }

    private const val PREFS_NAME = "receiver.settings"
}
