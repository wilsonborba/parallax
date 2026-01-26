package com.parallax.receiver.domain.service

import android.view.Surface
import com.parallax.receiver.core.logging.Logger
import com.parallax.receiver.core.logging.LoggerProvider
import com.parallax.receiver.core.streaming.H264StreamReceiver
import com.parallax.receiver.dal.local.SettingsStore
import com.parallax.receiver.domain.model.StreamConfig
import com.parallax.receiver.domain.model.StreamState
import com.parallax.receiver.domain.model.UiState
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

class StreamSessionService(
    private val coroutineScope: CoroutineScope = CoroutineScope(SupervisorJob() + Dispatchers.Default),
    private val connectionDelayMillis: Long = 750L,
    private val settingsStore: SettingsStore,
    private val streamReceiver: H264StreamReceiver = H264StreamReceiver(),
    private val logger: Logger = LoggerProvider.logger,
    initialConfig: StreamConfig = StreamConfig(
        host = settingsStore.getHost(),
        port = settingsStore.getPort(),
        scale = settingsStore.getScale(),
    ),
) {
    private val mutableState = MutableStateFlow(
        UiState(
            config = initialConfig,
            streamState = StreamState(StreamState.Status.Idle),
        ),
    )
    private var connectionJob: Job? = null
    private var renderSurface: Surface? = null

    val uiState: StateFlow<UiState> = mutableState.asStateFlow()

    fun startStream(config: StreamConfig) {
        connectionJob?.cancel()
        mutableState.value = UiState(
            config = config,
            streamState = StreamState(StreamState.Status.Connecting),
        )
        connectionJob = coroutineScope.launch {
            delay(connectionDelayMillis)
            val surface = renderSurface
            mutableState.update { current ->
                if (current.streamState.status == StreamState.Status.Connecting) {
                    if (surface == null) {
                        current.copy(
                            streamState = StreamState(
                                StreamState.Status.Error,
                                "No render surface available.",
                            ),
                        )
                    } else {
                        current.copy(streamState = StreamState(StreamState.Status.Streaming))
                    }
                } else {
                    current
                }
            }
            if (surface != null && mutableState.value.streamState.status == StreamState.Status.Streaming) {
                startReceiver(config, surface)
            }
        }
    }

    fun stopStream() {
        connectionJob?.cancel()
        streamReceiver.stop()
        mutableState.update { current ->
            current.copy(streamState = StreamState(StreamState.Status.Idle))
        }
    }

    fun setRenderSurface(surface: Surface) {
        renderSurface = surface
        if (mutableState.value.streamState.status == StreamState.Status.Streaming && !streamReceiver.isRunning()) {
            startReceiver(mutableState.value.config, surface)
        }
    }

    fun clearRenderSurface() {
        renderSurface = null
        streamReceiver.stop()
    }

    fun setScale(scale: Float) {
        mutableState.update { current ->
            current.copy(config = current.config.copy(scale = scale))
        }
    }

    fun setHost(host: String) {
        mutableState.update { current ->
            current.copy(config = current.config.copy(host = host))
        }
    }

    fun setPort(port: Int) {
        mutableState.update { current ->
            current.copy(config = current.config.copy(port = port))
        }
    }

    private fun startReceiver(config: StreamConfig, surface: Surface) {
        try {
            streamReceiver.start(config.port, surface)
        } catch (e: Exception) {
            logger.error(
                TAG,
                "Failed to start stream receiver",
                mapOf("error" to e.message, "exception" to e),
            )
            mutableState.update { current ->
                current.copy(
                    streamState = StreamState(
                        StreamState.Status.Error,
                        e.message ?: "Failed to start stream receiver.",
                    ),
                )
            }
        }
    }

    private companion object {
        private const val TAG = "StreamSessionService"
    }
}
