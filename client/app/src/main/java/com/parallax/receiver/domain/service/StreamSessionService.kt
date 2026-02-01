package com.parallax.receiver.domain.service

import android.view.Surface
import com.parallax.receiver.core.control.ControlClient
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
        streamPort = settingsStore.getStreamPort(),
        controlPort = settingsStore.getControlPort(),
        scale = settingsStore.getScale(),
        accessPin = settingsStore.getAccessPin(),
        pairingToken = settingsStore.getPairingToken(),
    ),
) {
    private var controlClient: ControlClient = ControlClient(
        pairingToken = resolvePairingToken(initialConfig),
        logger = logger,
    )
    private val mutableState = MutableStateFlow(
        UiState(
            config = initialConfig,
            streamState = StreamState(StreamState.Status.Idle),
            pairingToken = initialConfig.pairingToken,
            controlPort = initialConfig.controlPort,
        ),
    )
    private var connectionJob: Job? = null
    private var renderSurface: Surface? = null
    private var pendingStartConfig: StreamConfig? = null
    private var controlSession: ControlClient.ControlSession? = null

    val uiState: StateFlow<UiState> = mutableState.asStateFlow()

    fun startStream(config: StreamConfig) {
        connectionJob?.cancel()
        pendingStartConfig = config
        logger.info(
            TAG,
            "Start stream requested",
            mapOf(
                "host" to config.host,
                "controlPort" to config.controlPort,
                "streamPort" to config.streamPort,
            ),
        )
        mutableState.value = UiState(
            config = config,
            streamState = StreamState(StreamState.Status.Connecting),
            pairingToken = config.pairingToken,
            controlPort = config.controlPort,
        )
        connectionJob = coroutineScope.launch {
            delay(connectionDelayMillis)
            val surface = renderSurface
            if (surface != null && mutableState.value.streamState.status == StreamState.Status.Connecting) {
                val sessionReady = openControlSession(config)
                if (!sessionReady) {
                    return@launch
                }
                startReceiver(config, surface)
                pendingStartConfig = null
                mutableState.update { current ->
                    if (current.streamState.status == StreamState.Status.Connecting) {
                        current.copy(streamState = StreamState(StreamState.Status.Streaming))
                    } else {
                        current
                    }
                }
            }
        }
    }

    fun stopStream() {
        connectionJob?.cancel()
        streamReceiver.stop()
        stopControlSession()
        pendingStartConfig = null
        mutableState.update { current ->
            current.copy(streamState = StreamState(StreamState.Status.Idle))
        }
    }

    fun setRenderSurface(surface: Surface) {
        renderSurface = surface
        if (streamReceiver.isRunning()) {
            return
        }
        val config = pendingStartConfig ?: mutableState.value.config
        when (mutableState.value.streamState.status) {
            StreamState.Status.Idle -> Unit
            StreamState.Status.Connecting -> {
                if (controlSession == null) {
                    val sessionReady = openControlSession(config)
                    if (!sessionReady) {
                        return
                    }
                }
                startReceiver(config, surface)
                pendingStartConfig = null
                mutableState.update { current ->
                    if (current.streamState.status == StreamState.Status.Connecting) {
                        current.copy(streamState = StreamState(StreamState.Status.Streaming))
                    } else {
                        current
                    }
                }
            }
            StreamState.Status.Streaming -> {
                if (controlSession == null) {
                    val sessionReady = openControlSession(config)
                    if (!sessionReady) {
                        return
                    }
                }
                startReceiver(config, surface)
            }
            StreamState.Status.Error -> {
                val sessionReady = openControlSession(config)
                if (!sessionReady) {
                    return
                }
                startReceiver(config, surface)
            }
        }
    }

    fun clearRenderSurface() {
        renderSurface = null
        val wasRunning = streamReceiver.isRunning()
        streamReceiver.stop()
        if (mutableState.value.streamState.status == StreamState.Status.Streaming && wasRunning) {
            return
        }
        stopControlSession()
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

    fun setStreamPort(port: Int) {
        mutableState.update { current ->
            current.copy(config = current.config.copy(streamPort = port))
        }
    }

    fun setControlPort(port: Int) {
        mutableState.update { current ->
            current.copy(config = current.config.copy(controlPort = port), controlPort = port)
        }
    }

    fun setAccessPin(accessPin: String) {
        mutableState.update { current ->
            current.copy(config = current.config.copy(accessPin = accessPin))
        }
    }

    fun setPairingToken(pairingToken: String) {
        settingsStore.setPairingToken(pairingToken)
        val updatedConfig = mutableState.value.config.copy(pairingToken = pairingToken)
        controlClient = ControlClient(pairingToken = resolvePairingToken(updatedConfig), logger = logger)
        mutableState.update { current ->
            current.copy(
                config = updatedConfig,
                pairingToken = updatedConfig.pairingToken,
                controlPort = updatedConfig.controlPort,
            )
        }
    }

    private fun startReceiver(config: StreamConfig, surface: Surface) {
        try {
            streamReceiver.start(config.streamPort, surface)
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

    private fun openControlSession(config: StreamConfig): Boolean {
        if (controlSession != null) {
            return true
        }
        return try {
            val pairingToken = resolvePairingToken(config)
            logger.info(
                TAG,
                "Opening control session",
                mapOf(
                    "host" to config.host,
                    "controlPort" to config.controlPort,
                    "streamPort" to config.streamPort,
                ),
            )
            controlClient = ControlClient(pairingToken = pairingToken, logger = logger)
            val session = controlClient.openSession(
                config.host,
                config.controlPort,
                config.streamPort,
            )
            session.startStream()
            controlSession = session
            true
        } catch (e: Exception) {
            logger.error(
                TAG,
                "Failed to open control session",
                mapOf("error" to e.message, "exception" to e),
            )
            mutableState.update { current ->
                current.copy(
                    streamState = StreamState(
                        StreamState.Status.Error,
                        e.message ?: "Failed to open control session.",
                    ),
                )
            }
            false
        }
    }

    private fun stopControlSession() {
        val session = controlSession ?: return
        controlSession = null
        try {
            session.stopStream()
        } catch (e: Exception) {
            logger.warn(TAG, "Failed to stop control session", mapOf("error" to e.message, "exception" to e))
        } finally {
            session.close()
        }
    }

    private companion object {
        private const val TAG = "StreamSessionService"
        private const val DEFAULT_PAIRING_TOKEN = "parallax"
    }

    private fun resolvePairingToken(config: StreamConfig): String {
        val accessPin = config.accessPin
        val pairingToken = config.pairingToken
        val shouldUseAccessPin = accessPin.isNotBlank() && (pairingToken.isBlank() || pairingToken == DEFAULT_PAIRING_TOKEN)
        val resolved = if (shouldUseAccessPin) accessPin else pairingToken
        logger.info(
            TAG,
            "Resolved pairing token",
            mapOf(
                "usingAccessPin" to shouldUseAccessPin,
                "pairingTokenBlank" to pairingToken.isBlank(),
                "accessPinBlank" to accessPin.isBlank(),
            ),
        )
        return resolved
    }
}
