package com.parallax.receiver.domain.module

import com.parallax.receiver.dal.local.SettingsStore
import com.parallax.receiver.domain.service.StreamSessionService

class SetScaleUseCase(
    private val settingsStore: SettingsStore,
    private val streamSessionService: StreamSessionService,
) {
    operator fun invoke(scale: Float) {
        settingsStore.setScale(scale)
        streamSessionService.setScale(scale)
    }
}
