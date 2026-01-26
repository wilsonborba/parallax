package com.parallax.receiver.domain.service

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
    initialConfig: StreamConfig = StreamConfig(
        host = "0.0.0.0",
        port = 7777,
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

    val uiState: StateFlow<UiState> = mutableState.asStateFlow()

    fun startStream(config: StreamConfig) {
        connectionJob?.cancel()
        mutableState.value = UiState(
            config = config,
            streamState = StreamState(StreamState.Status.Connecting),
        )
        connectionJob = coroutineScope.launch {
            delay(connectionDelayMillis)
            mutableState.update { current ->
                if (current.streamState.status == StreamState.Status.Connecting) {
                    current.copy(streamState = StreamState(StreamState.Status.Streaming))
                } else {
                    current
                }
            }
        }
    }

    fun stopStream() {
        connectionJob?.cancel()
        mutableState.update { current ->
            current.copy(streamState = StreamState(StreamState.Status.Idle))
        }
    }

    fun setScale(scale: Float) {
        mutableState.update { current ->
            current.copy(config = current.config.copy(scale = scale))
        }
    }
}
