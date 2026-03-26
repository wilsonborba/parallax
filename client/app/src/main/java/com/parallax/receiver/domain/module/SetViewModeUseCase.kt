package com.parallax.receiver.domain.module

import com.parallax.receiver.dal.local.SettingsStore
import com.parallax.receiver.domain.service.StreamSessionService

class SetViewModeUseCase(
    private val settingsStore: SettingsStore,
    private val streamSessionService: StreamSessionService,
) {
    operator fun invoke(viewMode: String) {
        settingsStore.setViewMode(viewMode)
        streamSessionService.setViewMode(viewMode)
    }
}
