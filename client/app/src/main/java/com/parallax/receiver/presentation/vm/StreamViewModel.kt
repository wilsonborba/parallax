package com.parallax.receiver.presentation.vm

import android.view.Surface
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.parallax.receiver.core.logging.Logger
import com.parallax.receiver.core.logging.LoggerProvider
import com.parallax.receiver.core.qr.QrParser
import com.parallax.receiver.domain.model.StreamState
import com.parallax.receiver.domain.model.UiState
import com.parallax.receiver.domain.service.StreamSessionService
import com.parallax.receiver.domain.module.SetScaleUseCase
import com.parallax.receiver.domain.module.SetViewModeUseCase
import com.parallax.receiver.domain.module.SetStreamEndpointUseCase
import com.parallax.receiver.domain.module.StartStreamUseCase
import com.parallax.receiver.domain.module.StopStreamUseCase
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.launch

class StreamViewModel(
    private val streamSessionService: StreamSessionService,
    private val startStream: StartStreamUseCase,
    private val stopStream: StopStreamUseCase,
    private val setScale: SetScaleUseCase,
    private val setViewMode: SetViewModeUseCase,
    private val setStreamEndpoint: SetStreamEndpointUseCase,
    private val logger: Logger = LoggerProvider.logger,
) : ViewModel() {
    val uiState: StateFlow<UiState> = streamSessionService.uiState
    private val _uiEvents = MutableSharedFlow<StreamUiEvent>(extraBufferCapacity = 1)
    val uiEvents: SharedFlow<StreamUiEvent> = _uiEvents.asSharedFlow()
    private var lastStatus: StreamState.Status? = null
    private var lastTopologyStatus: String? = null

    init {
        observeStateTransitions()
    }

    fun onStartClicked() {
        startStream.invoke(uiState.value.config)
    }

    fun onStopClicked() {
        stopStream.invoke()
    }

    fun onScaleChanged(scale: Float) {
        setScale.invoke(scale)
    }

    fun onViewModeChanged(viewMode: String) {
        setViewMode.invoke(viewMode)
    }

    fun onHostChanged(host: String) {
        setStreamEndpoint.setHost(host)
    }

    fun onStreamPortChanged(port: Int) {
        setStreamEndpoint.setStreamPort(port)
    }

    fun onControlPortChanged(port: Int) {
        setStreamEndpoint.setControlPort(port)
    }

    fun onAccessPinChanged(accessPin: String) {
        setStreamEndpoint.setAccessPin(accessPin)
    }

    fun onQrPayloadScanned(payload: String) {
        val endpoint = QrParser.parse(payload)
        if (endpoint == null) {
            logger.warn(TAG, "Unsupported QR payload", mapOf("payload" to payload))
            _uiEvents.tryEmit(StreamUiEvent.ShowMessage(UNSUPPORTED_QR_MESSAGE))
            return
        }
        setStreamEndpoint.setHost(endpoint.host)
        setStreamEndpoint.setControlPort(endpoint.controlPort)
        endpoint.streamPort?.let { setStreamEndpoint.setStreamPort(it) }
        endpoint.accessPin?.let { setStreamEndpoint.setAccessPin(it) }
        _uiEvents.tryEmit(StreamUiEvent.ShowMessage(buildQrSuccessMessage(endpoint)))
    }

    fun onSurfaceAvailable(streamId: Int, surface: Surface) {
        streamSessionService.setRenderSurface(streamId, surface)
    }

    fun onSurfaceDestroyed(streamId: Int) {
        streamSessionService.clearRenderSurface(streamId)
    }

    fun onAddMonitorClicked() {
        streamSessionService.requestAddMonitor()
    }

    fun onRemoveMonitorClicked(displayId: String) {
        streamSessionService.requestRemoveMonitor(displayId)
    }

    fun onRefreshTopologyClicked() {
        streamSessionService.refreshTopology()
    }

    fun onStartMonitorClicked(displayId: String) {
        streamSessionService.requestStartMonitor(displayId)
    }

    fun onStopMonitorClicked(displayId: String) {
        streamSessionService.requestStopMonitor(displayId)
    }

    fun onStatsOverlayVisibilityChanged(enabled: Boolean) {
        streamSessionService.setStatsOverlayEnabled(enabled)
    }

    private fun observeStateTransitions() {
        viewModelScope.launch {
            uiState.collect { state ->
                val currentStatus = state.streamState.status
                if (lastStatus != currentStatus) {
                    logger.info(
                        TAG,
                        "Stream state transition",
                        mapOf(
                            "from" to lastStatus?.name,
                            "to" to currentStatus.name,
                            "host" to state.config.host,
                            "streamPort" to state.config.streamPort,
                            "controlPort" to state.controlPort,
                            "scale" to state.config.scale,
                        ),
                    )
                    lastStatus = currentStatus
                }
                if (!state.topologyStatus.isNullOrBlank() && state.topologyStatus != lastTopologyStatus) {
                    _uiEvents.tryEmit(StreamUiEvent.ShowMessage(state.topologyStatus))
                    lastTopologyStatus = state.topologyStatus
                }
            }
        }
    }

    companion object {
        private const val TAG = "StreamViewModel"
        private const val UNSUPPORTED_QR_MESSAGE = "Unsupported QR code. Expected prlx://host:port."
        private fun buildQrSuccessMessage(endpoint: com.parallax.receiver.core.qr.QrEndpoint): String {
            val portSuffix = if (endpoint.streamPort == null) {
                "Control port set to ${endpoint.controlPort}."
            } else {
                "Control ${endpoint.controlPort}, stream ${endpoint.streamPort}."
            }
            val pinSuffix = if (endpoint.accessPin == null) {
                ""
            } else {
                " PIN set."
            }
            return "QR scanned. Host set to ${endpoint.host}. $portSuffix$pinSuffix"
        }
    }
}

sealed interface StreamUiEvent {
    data class ShowMessage(val message: String) : StreamUiEvent
}
