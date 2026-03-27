package com.parallax.receiver.domain.service

import android.view.Surface
import com.parallax.receiver.core.control.ControlClient
import com.parallax.receiver.core.logging.Logger
import com.parallax.receiver.core.logging.LoggerProvider
import com.parallax.receiver.core.streaming.H264StreamReceiver
import com.parallax.receiver.dal.local.SettingsStore
import com.parallax.receiver.domain.model.MonitorPanelState
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
        viewMode = settingsStore.getViewMode(),
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
            statsOverlayEnabled = settingsStore.getStatsOverlayEnabled(),
        ),
    )
    private var connectionJob: Job? = null
    private var renderSurface: Surface? = null
    private var pendingStartConfig: StreamConfig? = null
    private var controlSession: ControlClient.ControlSession? = null
    private var resumeOnSurfaceAvailable: Boolean = false

    val uiState: StateFlow<UiState> = mutableState.asStateFlow()

    init {
        streamReceiver.setOnVideoDimensionsDetected { dimensions ->
            mutableState.update { current ->
                if (current.videoDimensions == dimensions) {
                    current
                } else {
                    current.copy(videoDimensions = dimensions)
                }
            }
        }
    }

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
        mutableState.update { current ->
            current.copy(
                config = config,
                streamState = StreamState(StreamState.Status.Connecting),
                pairingToken = config.pairingToken,
                controlPort = config.controlPort,
                videoDimensions = null,
            )
        }
        connectionJob = coroutineScope.launch {
            delay(connectionDelayMillis)
            val surface = renderSurface
            if (surface != null && mutableState.value.streamState.status == StreamState.Status.Connecting) {
                val sessionReady = openControlSession(config)
                if (!sessionReady) {
                    return@launch
                }
                if (!ensureStreamStarted(1)) {
                    return@launch
                }
                startReceiver(config, surface)
                refreshTopologyIfPossible()
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
            current.copy(
                streamState = StreamState(StreamState.Status.Idle),
                videoDimensions = null,
            )
        }
    }

    fun setRenderSurface(surface: Surface) {
        renderSurface = surface
        if (streamReceiver.isRunning()) {
            return
        }
        val config = pendingStartConfig ?: mutableState.value.config
        if (resumeOnSurfaceAvailable) {
            resumeOnSurfaceAvailable = false
            if (controlSession == null) {
                val sessionReady = openControlSession(config)
                if (!sessionReady) {
                    return
                }
            }
            startReceiver(config, surface)
            pendingStartConfig = null
            mutableState.update { current ->
                if (current.streamState.status == StreamState.Status.Connecting ||
                    current.streamState.status == StreamState.Status.Streaming
                ) {
                    current.copy(streamState = StreamState(StreamState.Status.Streaming))
                } else {
                    current
                }
            }
            return
        }
        when (mutableState.value.streamState.status) {
            StreamState.Status.Idle -> Unit
            StreamState.Status.Connecting -> {
                if (controlSession == null) {
                    val sessionReady = openControlSession(config)
                    if (!sessionReady) {
                        return
                    }
                }
                if (!ensureStreamStarted(1)) {
                    return
                }
                startReceiver(config, surface)
                refreshTopologyIfPossible()
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
                if (!ensureStreamStarted(1)) {
                    return
                }
                startReceiver(config, surface)
                refreshTopologyIfPossible()
            }
            StreamState.Status.Error -> {
                val sessionReady = openControlSession(config)
                if (!sessionReady) {
                    return
                }
                if (!ensureStreamStarted(1)) {
                    return
                }
                startReceiver(config, surface)
                refreshTopologyIfPossible()
            }
        }
    }

    fun clearRenderSurface() {
        renderSurface = null
        val currentStatus = mutableState.value.streamState.status
        val shouldResumeOnSurface = currentStatus == StreamState.Status.Streaming ||
            currentStatus == StreamState.Status.Connecting
        resumeOnSurfaceAvailable = shouldResumeOnSurface
        streamReceiver.stop()
        if (shouldResumeOnSurface) {
            return
        }
        stopControlSession()
    }

    fun setScale(scale: Float) {
        mutableState.update { current ->
            current.copy(config = current.config.copy(scale = scale))
        }
    }

    fun setViewMode(viewMode: String) {
        mutableState.update { current ->
            current.copy(config = current.config.copy(viewMode = viewMode))
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

    fun setStatsOverlayEnabled(enabled: Boolean) {
        settingsStore.setStatsOverlayEnabled(enabled)
        mutableState.update { current ->
            current.copy(statsOverlayEnabled = enabled)
        }
    }

    fun refreshTopology() {
        runTopologyCommand("Topology refreshed.") { session ->
            syncTopology(session, "Topology refreshed.")
        }
    }

    fun requestAddMonitor() {
        runTopologyCommand("Monitor added.") { session ->
            val displays = session.listDisplays()
            val occupiedSlots = displays.virtual.mapNotNull { parseStreamIdFromDisplayId(it.id) }.toSet()
            val slot = (1..MAX_VIRTUAL_MONITORS).firstOrNull { it !in occupiedSlots }
                ?: throw IllegalStateException("Maximum of $MAX_VIRTUAL_MONITORS monitors reached.")
            val displayId = "prlx-v$slot"
            val rightMost = (
                displays.physical.maxOfOrNull { it.x + it.width } ?: 0
                ).coerceAtLeast(displays.virtual.maxOfOrNull { it.x + it.width } ?: 0)
            session.addVirtualDisplay(
                id = displayId,
                width = DEFAULT_MONITOR_WIDTH,
                height = DEFAULT_MONITOR_HEIGHT,
                x = rightMost,
                y = 0,
            )
            val startMessage = configureAndStartMonitor(session, displayId)
            syncTopology(session, startMessage ?: "Monitor $displayId added.")
        }
    }

    fun requestRemoveMonitor(displayId: String) {
        if (displayId.isBlank()) {
            return
        }
        runTopologyCommand("Monitor removed.") { session ->
            val streams = session.listStreams()
            val attached = streams.firstOrNull { it.displayId == displayId && it.running }
            if (attached != null) {
                session.stopStream(attached.streamId)
            }
            session.removeVirtualDisplay(displayId)
            syncTopology(session, "Monitor $displayId removed.")
        }
    }

    fun requestStartMonitor(displayId: String) {
        if (displayId.isBlank()) {
            return
        }
        runTopologyCommand("Monitor started.") { session ->
            val status = configureAndStartMonitor(session, displayId)
            syncTopology(session, status ?: "Monitor $displayId started.")
        }
    }

    fun requestStopMonitor(displayId: String) {
        if (displayId.isBlank()) {
            return
        }
        runTopologyCommand("Monitor stopped.") { session ->
            stopMonitorByDisplay(session, displayId)
            syncTopology(session, "Monitor $displayId stopped.")
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

    private fun ensureStreamStarted(streamId: Int): Boolean {
        val session = controlSession ?: return false
        return try {
            session.startStream(streamId)
            true
        } catch (e: Exception) {
            logger.error(
                TAG,
                "Failed to start stream on host",
                mapOf("streamId" to streamId, "error" to e.message, "exception" to e),
            )
            mutableState.update { current ->
                current.copy(
                    streamState = StreamState(
                        StreamState.Status.Error,
                        e.message ?: "Failed to start stream on host.",
                    ),
                )
            }
            false
        }
    }

    private fun runTopologyCommand(successStatus: String, command: (ControlClient.ControlSession) -> Unit) {
        if (mutableState.value.topologyBusy) {
            return
        }
        mutableState.update { current ->
            current.copy(topologyBusy = true, topologyStatus = "Applying monitor topology...")
        }
        coroutineScope.launch {
            val config = mutableState.value.config
            val sessionReady = openControlSession(config)
            if (!sessionReady) {
                mutableState.update { current ->
                    current.copy(topologyBusy = false, topologyStatus = "Failed to open control session.")
                }
                return@launch
            }
            val session = controlSession ?: run {
                mutableState.update { current ->
                    current.copy(topologyBusy = false, topologyStatus = "Control session not available.")
                }
                return@launch
            }
            try {
                command(session)
                mutableState.update { current ->
                    current.copy(topologyBusy = false, topologyStatus = successStatus)
                }
            } catch (e: Exception) {
                logger.warn(
                    TAG,
                    "Monitor topology command failed",
                    mapOf("error" to e.message, "exception" to e),
                )
                mutableState.update { current ->
                    current.copy(
                        topologyBusy = false,
                        topologyStatus = e.message ?: "Topology command failed.",
                    )
                }
            }
        }
    }

    private fun syncTopology(session: ControlClient.ControlSession, status: String?) {
        val displays = session.listDisplays()
        val streamsByDisplay = session.listStreams().associateBy { it.displayId }
        val panels = displays.virtual
            .filter { it.enabled }
            .sortedBy { it.x }
            .map { display ->
                val stream = streamsByDisplay[display.id]
                MonitorPanelState(
                    streamId = stream?.streamId ?: parseStreamIdFromDisplayId(display.id) ?: 1,
                    displayId = display.id,
                    width = stream?.width?.takeIf { it > 0 } ?: display.width,
                    height = stream?.height?.takeIf { it > 0 } ?: display.height,
                    x = display.x,
                    y = display.y,
                    running = stream?.running ?: false,
                    fps = stream?.fps ?: 0f,
                    bitrateKbps = stream?.bitrateKbps ?: 0,
                )
            }

        mutableState.update { current ->
            current.copy(
                monitorPanels = panels,
                topologyStatus = status,
            )
        }
    }

    private fun parseStreamIdFromDisplayId(displayId: String): Int? {
        val suffix = displayId.removePrefix("prlx-v")
        if (suffix == displayId) return null
        return suffix.toIntOrNull()
    }

    private fun refreshTopologyIfPossible() {
        val session = controlSession ?: return
        try {
            syncTopology(session, null)
        } catch (e: Exception) {
            logger.warn(TAG, "Failed to refresh topology", mapOf("error" to e.message, "exception" to e))
        }
    }

    private fun configureAndStartMonitor(
        session: ControlClient.ControlSession,
        displayId: String,
    ): String? {
        val streamId = parseStreamIdFromDisplayId(displayId) ?: 1
        return try {
            session.setStreamConfig(streamId = streamId, displayId = displayId)
            session.startStream(streamId)
            null
        } catch (e: Exception) {
            if (isStreamNotFoundError(e) && streamId != 1) {
                // Fallback for current single-stream host: bind to stream 1.
                session.setStreamConfig(streamId = 1, displayId = displayId)
                session.startStream(1)
                "Host single-stream fallback: $displayId bound to stream 1."
            } else {
                throw e
            }
        }
    }

    private fun stopMonitorByDisplay(session: ControlClient.ControlSession, displayId: String) {
        val streams = session.listStreams()
        val attached = streams.firstOrNull { it.displayId == displayId && it.running }
        if (attached != null) {
            session.stopStream(attached.streamId)
            return
        }

        val streamId = parseStreamIdFromDisplayId(displayId) ?: 1
        try {
            session.stopStream(streamId)
        } catch (e: Exception) {
            if (isStreamNotFoundError(e) && streamId != 1) {
                session.stopStream(1)
            } else {
                throw e
            }
        }
    }

    private fun isStreamNotFoundError(error: Exception): Boolean {
        val message = error.message?.lowercase() ?: return false
        return message.contains("stream_id not found")
    }

    private companion object {
        private const val TAG = "StreamSessionService"
        private const val DEFAULT_PAIRING_TOKEN = "parallax"
        private const val MAX_VIRTUAL_MONITORS = 3
        private const val DEFAULT_MONITOR_WIDTH = 1920
        private const val DEFAULT_MONITOR_HEIGHT = 1080
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
