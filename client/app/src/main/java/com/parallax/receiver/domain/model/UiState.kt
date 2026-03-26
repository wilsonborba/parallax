package com.parallax.receiver.domain.model

data class UiState(
    val config: StreamConfig,
    val streamState: StreamState,
    val pairingToken: String,
    val controlPort: Int,
    val videoDimensions: VideoDimensions? = null,
    val monitorPanels: List<MonitorPanelState> = emptyList(),
    val topologyBusy: Boolean = false,
    val topologyStatus: String? = null,
    val statsOverlayEnabled: Boolean = false,
)
