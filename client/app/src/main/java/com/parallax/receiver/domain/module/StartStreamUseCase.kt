package com.parallax.receiver.domain.module

import com.parallax.receiver.domain.model.StreamConfig
import com.parallax.receiver.domain.service.StreamSessionService

class StartStreamUseCase(
    private val streamSessionService: StreamSessionService,
) {
    operator fun invoke(config: StreamConfig) {
        streamSessionService.startStream(config)
    }
}
