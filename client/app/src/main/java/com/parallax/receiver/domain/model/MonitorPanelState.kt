package com.parallax.receiver.domain.model

data class MonitorPanelState(
    val streamId: Int,
    val displayId: String,
    val width: Int,
    val height: Int,
    val x: Int,
    val y: Int,
    val running: Boolean,
    val fps: Float = 0f,
    val bitrateKbps: Int = 0,
)
