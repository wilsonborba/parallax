package com.parallax.receiver.domain.module

import com.parallax.receiver.dal.local.SettingsStore
import com.parallax.receiver.domain.service.StreamSessionService

class SetStreamEndpointUseCase(
    private val settingsStore: SettingsStore,
    private val streamSessionService: StreamSessionService,
) {
    fun setHost(host: String) {
        settingsStore.setHost(host)
        streamSessionService.setHost(host)
    }

    fun setPort(port: Int) {
        settingsStore.setPort(port)
        streamSessionService.setPort(port)
    }

    fun setAccessPin(accessPin: String) {
        settingsStore.setAccessPin(accessPin)
        streamSessionService.setAccessPin(accessPin)
    }
}
