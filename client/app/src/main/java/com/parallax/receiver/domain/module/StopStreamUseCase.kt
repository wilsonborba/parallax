package com.parallax.receiver.domain.module

import com.parallax.receiver.domain.service.StreamSessionService

class StopStreamUseCase(
    private val streamSessionService: StreamSessionService,
) {
    operator fun invoke() {
        streamSessionService.stopStream()
    }
}
