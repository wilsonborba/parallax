package com.parallax.receiver.presentation.vm

import android.view.Surface
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.parallax.receiver.core.logging.Logger
import com.parallax.receiver.core.logging.LoggerProvider
import com.parallax.receiver.core.qr.PrlxQrParser
import com.parallax.receiver.domain.model.StreamState
import com.parallax.receiver.domain.model.UiState
import com.parallax.receiver.domain.service.StreamSessionService
import com.parallax.receiver.domain.module.SetScaleUseCase
import com.parallax.receiver.domain.module.SetStreamEndpointUseCase
import com.parallax.receiver.domain.module.StartStreamUseCase
import com.parallax.receiver.domain.module.StopStreamUseCase
import kotlinx.coroutines.flow.StateFlow
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
        val endpoint = PrlxQrParser.parse(payload) ?: return
        setStreamEndpoint.setHost(endpoint.host)
        setStreamEndpoint.setControlPort(endpoint.controlPort)
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
                            "controlPort" to state.config.controlPort,
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
    }
}
