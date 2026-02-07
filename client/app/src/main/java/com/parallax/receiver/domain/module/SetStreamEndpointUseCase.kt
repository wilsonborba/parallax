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

    fun setStreamPort(port: Int) {
        settingsStore.setStreamPort(port)
        streamSessionService.setStreamPort(port)
    }

    fun setControlPort(port: Int) {
        settingsStore.setControlPort(port)
        streamSessionService.setControlPort(port)
    }

    fun setAccessPin(accessPin: String) {
        settingsStore.setAccessPin(accessPin)
        streamSessionService.setAccessPin(accessPin)
    }

    fun setPairingToken(pairingToken: String) {
        settingsStore.setPairingToken(pairingToken)
        streamSessionService.setPairingToken(pairingToken)
    }
}
