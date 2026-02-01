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
    private val setStreamEndpoint: SetStreamEndpointUseCase,
    private val logger: Logger = LoggerProvider.logger,
) : ViewModel() {
    val uiState: StateFlow<UiState> = streamSessionService.uiState
    private val _uiEvents = MutableSharedFlow<StreamUiEvent>(extraBufferCapacity = 1)
    val uiEvents: SharedFlow<StreamUiEvent> = _uiEvents.asSharedFlow()
    private var lastStatus: StreamState.Status? = null

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
        _uiEvents.tryEmit(StreamUiEvent.ShowMessage(buildQrSuccessMessage(endpoint)))
    }

    fun onSurfaceAvailable(surface: Surface) {
        streamSessionService.setRenderSurface(surface)
    }

    fun onSurfaceDestroyed() {
        streamSessionService.clearRenderSurface()
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
            return "QR scanned. Host set to ${endpoint.host}. $portSuffix"
        }
    }
}

sealed interface StreamUiEvent {
    data class ShowMessage(val message: String) : StreamUiEvent
}
