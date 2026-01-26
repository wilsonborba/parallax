package com.parallax.receiver.presentation.vm

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.parallax.receiver.core.logging.Logger
import com.parallax.receiver.core.logging.LoggerProvider
import com.parallax.receiver.domain.model.StreamConfig
import com.parallax.receiver.domain.model.StreamState
import com.parallax.receiver.domain.service.StreamSessionService
import com.parallax.receiver.domain.module.SetScaleUseCase
import com.parallax.receiver.domain.module.StartStreamUseCase
import com.parallax.receiver.domain.module.StopStreamUseCase
import kotlinx.coroutines.launch

class StreamViewModel(
    private val streamSessionService: StreamSessionService,
    private val startStream: StartStreamUseCase,
    private val stopStream: StopStreamUseCase,
    private val setScale: SetScaleUseCase,
    private val logger: Logger = LoggerProvider.logger,
) : ViewModel() {
    val uiState = streamSessionService.uiState
    private var lastStatus: StreamState.Status? = null

    init {
        observeStateTransitions()
    }

    fun startStream(config: StreamConfig) {
        startStream.invoke(config)
    }

    fun stopStream() {
        stopStream.invoke()
    }

    fun setScale(scale: Float) {
        setScale.invoke(scale)
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
                            "port" to state.config.port,
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
