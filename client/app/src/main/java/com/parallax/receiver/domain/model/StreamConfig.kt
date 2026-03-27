package com.parallax.receiver.domain.model

data class StreamConfig(
    val host: String,
    val streamPort: Int,
    val controlPort: Int,
    val scale: Float,
    val viewMode: String,
    val accessPin: String,
    val pairingToken: String,
)
