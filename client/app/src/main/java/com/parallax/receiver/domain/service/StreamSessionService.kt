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
    private val extraRenderSurfaces = mutableMapOf<Int, Surface>()
    private val extraReceivers = mutableMapOf<Int, H264StreamReceiver>()
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
        appendDebugLog("info", "Start requested: ${config.host}:${config.controlPort} (udp ${config.streamPort})")
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
        stopAllExtraReceivers()
        stopControlSession()
        pendingStartConfig = null
        mutableState.update { current ->
            current.copy(
                streamState = StreamState(StreamState.Status.Idle),
                videoDimensions = null,
                receiverRunning = false,
            )
        }
        appendDebugLog("info", "Stream stopped by user.")
    }

    fun setRenderSurface(streamId: Int, surface: Surface) {
        if (streamId != 1) {
            extraRenderSurfaces[streamId] = surface
            val status = mutableState.value.streamState.status
            if (status == StreamState.Status.Streaming || status == StreamState.Status.Connecting) {
                val config = pendingStartConfig ?: mutableState.value.config
                if (controlSession == null && !openControlSession(config)) {
                    return
                }
                if (ensureStreamStarted(streamId)) {
                    startExtraReceiver(streamId, config, surface)
                    updateReceiverRunningState()
                }
            }
            return
        }
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

    fun clearRenderSurface(streamId: Int) {
        if (streamId != 1) {
            extraRenderSurfaces.remove(streamId)
            extraReceivers.remove(streamId)?.stop()
            updateReceiverRunningState()
            return
        }
        renderSurface = null
        val currentStatus = mutableState.value.streamState.status
        val shouldResumeOnSurface = currentStatus == StreamState.Status.Streaming ||
            currentStatus == StreamState.Status.Connecting
        resumeOnSurfaceAvailable = shouldResumeOnSurface
        streamReceiver.stop()
        stopAllExtraReceivers()
        updateReceiverRunningState()
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
            val displays = topologyStep("listDisplays", session) { session.listDisplays() }
            val occupiedSlots = displays.virtual.mapNotNull { parseStreamIdFromDisplayId(it.id) }.toSet()
            // stream 1 is reserved for the primary panel; virtual additions start at stream 2.
            val slot = (2..MAX_STREAMS).firstOrNull { it !in occupiedSlots }
                ?: throw IllegalStateException("Maximum of ${MAX_STREAMS - 1} extra monitors reached.")
            val displayId = "prlx-v$slot"
            val rightMost = (
                displays.physical.maxOfOrNull { it.x + it.width } ?: 0
                ).coerceAtLeast(displays.virtual.maxOfOrNull { it.x + it.width } ?: 0)
            topologyStep("addVirtualDisplay($displayId)", session) {
                session.addVirtualDisplay(
                    id = displayId,
                    width = DEFAULT_MONITOR_WIDTH,
                    height = DEFAULT_MONITOR_HEIGHT,
                    x = rightMost,
                    y = 0,
                )
            }
            val startMessage = topologyStep("configureAndStartMonitor($displayId)", session) {
                configureAndStartMonitor(session, displayId)
            }
            topologyStep("syncTopology", session) {
                syncTopology(session, startMessage ?: "Monitor $displayId added.")
            }
        }
    }

    fun requestRemoveMonitor(displayId: String) {
        if (displayId.isBlank()) {
            return
        }
        runTopologyCommand("Monitor removed.") { session ->
            val streams = topologyStep("listStreams", session) { session.listStreams() }
            val attached = streams.firstOrNull { it.displayId == displayId && it.running }
            if (attached != null) {
                topologyStep("stopStream(${attached.streamId})", session) { session.stopStream(attached.streamId) }
            }
            topologyStep("removeVirtualDisplay($displayId)", session) { session.removeVirtualDisplay(displayId) }
            topologyStep("syncTopology", session) { syncTopology(session, "Monitor $displayId removed.") }
        }
    }

    fun requestStartMonitor(displayId: String) {
        if (displayId.isBlank()) {
            return
        }
        runTopologyCommand("Monitor started.") { session ->
            val status = topologyStep("configureAndStartMonitor($displayId)", session) {
                configureAndStartMonitor(session, displayId)
            }
            topologyStep("syncTopology", session) { syncTopology(session, status ?: "Monitor $displayId started.") }
        }
    }

    fun requestStopMonitor(displayId: String) {
        if (displayId.isBlank()) {
            return
        }
        runTopologyCommand("Monitor stopped.") { session ->
            topologyStep("stopMonitorByDisplay($displayId)", session) { stopMonitorByDisplay(session, displayId) }
            topologyStep("syncTopology", session) { syncTopology(session, "Monitor $displayId stopped.") }
        }
    }

    private fun startReceiver(config: StreamConfig, surface: Surface) {
        try {
            streamReceiver.start(config.streamPort, surface, streamId = 1L)
            reconcileExtraReceivers(config)
            updateReceiverRunningState()
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
                    receiverRunning = false,
                )
            }
        }
    }

    private fun startExtraReceiver(streamId: Int, config: StreamConfig, surface: Surface) {
        val receiver = extraReceivers.getOrPut(streamId) { H264StreamReceiver() }
        val port = streamPortFor(streamId, config.streamPort)
        receiver.start(port, surface, streamId = streamId.toLong())
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
            appendDebugLog("info", "Control session connected: ${config.host}:${config.controlPort}", session)
            true
        } catch (e: Exception) {
            logger.error(
                TAG,
                "Failed to open control session",
                mapOf("error" to e.message, "exception" to e),
            )
            mutableState.update { current ->
                if (current.receiverRunning) {
                    current.copy(topologyStatus = "Control session lost: ${e.message ?: "unknown"}")
                } else {
                    current.copy(
                        streamState = StreamState(
                            StreamState.Status.Error,
                            e.message ?: "Failed to open control session.",
                        ),
                    )
                }
            }
            appendDebugLog("error", "Control session error: ${e.message ?: "unknown"}")
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
            stopAllExtraReceivers()
            updateReceiverRunningState()
        }
    }

    private fun resetControlSession() {
        val session = controlSession ?: return
        controlSession = null
        try {
            session.close()
        } catch (_: Exception) {
            // Best effort cleanup for stale sockets.
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
        appendDebugLog("info", "Topology: applying command...")
        coroutineScope.launch {
            val config = mutableState.value.config
            var recoveredOnce = false
            while (true) {
                val session = try {
                    openTransientControlSession(config)
                } catch (e: Exception) {
                    logger.warn(
                        TAG,
                        "Failed to open transient control session",
                        mapOf("error" to e.message, "exception" to e),
                    )
                    mutableState.update { current ->
                        current.copy(
                            topologyBusy = false,
                            topologyStatus = e.message ?: "Failed to open control session.",
                        )
                    }
                    appendDebugLog("error", "Topology: open session failed: ${e.message ?: "unknown"}")
                    return@launch
                }
                try {
                    appendDebugLog("debug", "Topology: transient control session connected", session)
                    command(session)
                    reconcileExtraReceivers(config)
                    mutableState.update { current ->
                        current.copy(topologyBusy = false, topologyStatus = successStatus)
                    }
                    appendDebugLog("info", "Topology: success ($successStatus)")
                    return@launch
                } catch (e: Exception) {
                    val recoverable = isRecoverableControlError(e)
                    if (recoverable && !recoveredOnce) {
                        recoveredOnce = true
                        logger.warn(
                            TAG,
                            "Topology command failed on stale control session; retrying",
                            mapOf("error" to e.message, "exception" to e),
                        )
                        mutableState.update { current ->
                            current.copy(topologyBusy = true, topologyStatus = "Reconnecting to host...")
                        }
                        appendDebugLog("warn", "Topology: recoverable error (${e.message}); retrying...", session)
                        continue
                    }
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
                    appendDebugLog("error", "Topology failed: ${e.message ?: "unknown"}", session)
                    return@launch
                } finally {
                    try {
                        session.close()
                    } catch (_: Exception) {
                        // best effort
                    }
                }
            }
        }
    }

    private fun syncTopology(session: ControlClient.ControlSession, status: String?) {
        val displays = session.listDisplays()
        val streams = session.listStreams()
        val streamsByDisplay = streams.associateBy { it.displayId }
        val streamsById = streams.associateBy { it.streamId }
        val panels = displays.virtual
            .filter { it.enabled }
            .sortedBy { it.x }
            .mapNotNull { display ->
                val inferredStreamId = parseStreamIdFromDisplayId(display.id) ?: 1
                if (inferredStreamId == 1) {
                    // stream 1 is the primary panel; skip legacy virtual mapping on v1
                    return@mapNotNull null
                }
                // Prefer stream-id mapping (v1 -> stream 1, v2 -> stream 2, ...),
                // fallback to display-id mapping for hosts that already report it.
                val stream = streamsById[inferredStreamId] ?: streamsByDisplay[display.id]
                MonitorPanelState(
                    streamId = stream?.streamId ?: inferredStreamId,
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
        reconcileExtraReceivers(mutableState.value.config)
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
            reconcileExtraReceivers(mutableState.value.config)
        } catch (e: Exception) {
            logger.warn(TAG, "Failed to refresh topology", mapOf("error" to e.message, "exception" to e))
        }
    }

    private fun configureAndStartMonitor(
        session: ControlClient.ControlSession,
        displayId: String,
    ): String? {
        val streamId = parseStreamIdFromDisplayId(displayId) ?: 1
        val attached = session.listStreams().firstOrNull { it.displayId == displayId }
        if (attached?.running == true) {
            return "Monitor $displayId already running."
        }
        return try {
            // Host currently expects X11 DISPLAY (e.g. :0.0) for capture.
            // Do not send virtual monitor id as display selector until host supports monitor-region capture.
            session.setStreamConfig(streamId = streamId)
            session.startStream(streamId)
            null
        } catch (e: Exception) {
            if (isStreamNotFoundError(e) && streamId != 1) {
                // Fallback for current single-stream host: bind to stream 1.
                session.setStreamConfig(streamId = 1)
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

    private fun isRecoverableControlError(error: Exception): Boolean {
        val message = error.message?.lowercase() ?: return false
        return message.contains("broken pipe") ||
            message.contains("connection reset") ||
            message.contains("socket closed") ||
            message.contains("eof") ||
            message.contains("unexpected end of stream")
    }

    private fun openTransientControlSession(config: StreamConfig): ControlClient.ControlSession {
        val pairingToken = resolvePairingToken(config)
        val transientClient = ControlClient(pairingToken = pairingToken, logger = logger)
        appendDebugLog(
            "debug",
            "Topology: opening transient session ${config.host}:${config.controlPort} (no streamPort)",
            forwardToHost = false,
        )
        return transientClient.openSession(
            config.host,
            config.controlPort,
            null,
        )
    }

    private fun appendDebugLog(
        level: String,
        message: String,
        session: ControlClient.ControlSession? = null,
        forwardToHost: Boolean = true,
    ) {
        val line = "[${level.uppercase()}] $message"
        mutableState.update { current ->
            val next = (current.debugLogs + line).takeLast(MAX_DEBUG_LOGS)
            current.copy(debugLogs = next)
        }
        if (FORWARD_CLIENT_LOGS_TO_HOST && forwardToHost && session != null) {
            emitClientLogToHost(session, level, message)
        }
    }

    private inline fun <T> topologyStep(
        name: String,
        session: ControlClient.ControlSession,
        block: () -> T,
    ): T {
        appendDebugLog("debug", "Topology step: $name", session)
        return try {
            val result = block()
            appendDebugLog("debug", "Topology step OK: $name", session)
            result
        } catch (e: Exception) {
            appendDebugLog("error", "Topology step FAILED: $name -> ${e.message ?: "unknown"}", session)
            throw e
        }
    }

    private fun emitClientLogToHost(
        session: ControlClient.ControlSession,
        level: String,
        message: String,
    ) {
        try {
            session.sendClientLog(level, message)
        } catch (e: Exception) {
            logger.warn(
                TAG,
                "Failed to forward client log to host",
                mapOf("error" to e.message, "level" to level, "message" to message),
            )
        }
    }

    private fun reconcileExtraReceivers(config: StreamConfig) {
        val status = mutableState.value.streamState.status
        if (status != StreamState.Status.Streaming && status != StreamState.Status.Connecting) {
            stopAllExtraReceivers()
            updateReceiverRunningState()
            return
        }

        val desiredIds = mutableState.value.monitorPanels
            .asSequence()
            .filter { it.running && it.streamId != 1 }
            .map { it.streamId }
            .toSet()

        val staleIds = extraReceivers.keys.filter { it !in desiredIds || extraRenderSurfaces[it] == null }
        staleIds.forEach { streamId ->
            extraReceivers.remove(streamId)?.stop()
        }

        desiredIds.forEach { streamId ->
            val surface = extraRenderSurfaces[streamId] ?: return@forEach
            if (controlSession == null && !openControlSession(config)) {
                return
            }
            if (!ensureStreamStarted(streamId)) {
                return@forEach
            }
            startExtraReceiver(streamId, config, surface)
        }

        updateReceiverRunningState()
    }

    private fun stopAllExtraReceivers() {
        extraReceivers.values.forEach { it.stop() }
        extraReceivers.clear()
    }

    private fun updateReceiverRunningState() {
        val running = streamReceiver.isRunning() || extraReceivers.values.any { it.isRunning() }
        mutableState.update { current -> current.copy(receiverRunning = running) }
    }

    private fun streamPortFor(streamId: Int, basePort: Int): Int {
        return basePort + (streamId - 1).coerceAtLeast(0)
    }

    private companion object {
        private const val TAG = "StreamSessionService"
        private const val DEFAULT_PAIRING_TOKEN = "parallax"
        private const val MAX_STREAMS = 3
        private const val DEFAULT_MONITOR_WIDTH = 1920
        private const val DEFAULT_MONITOR_HEIGHT = 1080
        private const val MAX_DEBUG_LOGS = 200
        private const val FORWARD_CLIENT_LOGS_TO_HOST = true
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
