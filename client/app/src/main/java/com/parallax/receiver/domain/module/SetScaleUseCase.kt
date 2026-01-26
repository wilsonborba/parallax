package com.parallax.receiver.domain.module

import com.parallax.receiver.domain.service.StreamSessionService

class SetScaleUseCase(
    private val streamSessionService: StreamSessionService,
) {
    operator fun invoke(scale: Float) {
        streamSessionService.setScale(scale)
    }
}
