package com.parallax.receiver.domain.model

data class UiState(
    val config: StreamConfig,
    val streamState: StreamState,
    val pairingToken: String,
    val controlPort: Int,
)
