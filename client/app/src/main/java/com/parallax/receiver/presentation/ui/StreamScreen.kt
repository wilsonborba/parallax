package com.parallax.receiver.presentation.ui

import android.Manifest
import android.content.Context
import android.content.pm.PackageManager
import android.view.MotionEvent
import android.view.Surface
import android.view.SurfaceHolder
import android.view.SurfaceView
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.camera.core.CameraSelector
import androidx.camera.core.ImageAnalysis
import androidx.camera.core.Preview
import androidx.camera.lifecycle.ProcessCameraProvider
import androidx.camera.view.PreviewView
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.KeyboardArrowDown
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Surface
import androidx.compose.material3.Slider
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalLifecycleOwner
import androidx.compose.ui.input.pointer.pointerInteropFilter
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.core.content.ContextCompat
import com.google.mlkit.vision.barcode.BarcodeScanning
import com.google.mlkit.vision.common.InputImage
import com.parallax.receiver.core.config.DEFAULT_REMOTE_HEIGHT
import com.parallax.receiver.core.config.DEFAULT_REMOTE_WIDTH
import com.parallax.receiver.core.config.SCALE_MAX
import com.parallax.receiver.core.config.SCALE_MIN
import com.parallax.receiver.domain.model.MonitorPanelState
import com.parallax.receiver.domain.model.StreamState
import com.parallax.receiver.domain.model.UiState
import com.parallax.receiver.domain.model.VideoDimensions
import com.parallax.receiver.presentation.theme.spacing
import com.parallax.receiver.presentation.vm.StreamUiEvent
import java.util.concurrent.Executors
import kotlin.coroutines.resume
import kotlin.coroutines.suspendCoroutine
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.delay

private enum class ViewScaleMode(val storedValue: String) {
    Fit("fit"),
    Fill("fill"),
    Manual("manual");

    companion object {
        fun fromStored(value: String): ViewScaleMode = when (value.lowercase()) {
            "fill" -> Fill
            "manual" -> Manual
            else -> Fit
        }
    }
}

@Composable
fun StreamScreen(
    uiState: UiState,
    uiEvents: Flow<StreamUiEvent>,
    onStartClicked: () -> Unit,
    onStopClicked: () -> Unit,
    onScaleChanged: (Float) -> Unit,
    onViewModeChanged: (String) -> Unit,
    onHostChanged: (String) -> Unit,
    onStreamPortChanged: (Int) -> Unit,
    onControlPortChanged: (Int) -> Unit,
    onAccessPinChanged: (String) -> Unit,
    onQrPayloadScanned: (String) -> Unit,
    onSurfaceAvailable: (Int, Surface) -> Unit,
    onSurfaceDestroyed: (Int) -> Unit,
    onAddMonitorClicked: () -> Unit,
    onRemoveMonitorClicked: (String) -> Unit,
    onRefreshTopologyClicked: () -> Unit,
    onStartMonitorClicked: (String) -> Unit,
    onStopMonitorClicked: (String) -> Unit,
    onStatsOverlayVisibilityChanged: (Boolean) -> Unit,
    modifier: Modifier = Modifier,
) {
    val spacing = MaterialTheme.spacing
    val status = uiState.streamState.status
    var controlsVisible by remember { mutableStateOf(false) }
    var settingsHandleVisible by remember { mutableStateOf(false) }
    var statusHandleVisible by remember { mutableStateOf(false) }
    var statsEnabled by remember(uiState.statsOverlayEnabled) { mutableStateOf(uiState.statsOverlayEnabled) }
    val scaleMode = remember(uiState.config.viewMode) {
        ViewScaleMode.fromStored(uiState.config.viewMode)
    }
    var autoFitApplied by remember { mutableStateOf(false) }
    val snackbarHostState = remember { SnackbarHostState() }
    LaunchedEffect(uiEvents) {
        uiEvents.collect { event ->
            if (event is StreamUiEvent.ShowMessage) {
                snackbarHostState.showSnackbar(event.message)
            }
        }
    }
    Scaffold(
        modifier = modifier.fillMaxSize(),
        snackbarHost = { SnackbarHost(hostState = snackbarHostState) },
        contentWindowInsets = WindowInsets(0, 0, 0, 0),
    ) { innerPadding ->
        Surface(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding),
            color = MaterialTheme.colorScheme.background,
            tonalElevation = 0.dp,
        ) {
            BoxWithConstraints(modifier = Modifier.fillMaxSize()) {
                val videoDimensions = uiState.videoDimensions
                    ?: VideoDimensions(DEFAULT_REMOTE_WIDTH, DEFAULT_REMOTE_HEIGHT)
                val aspectRatio = videoDimensions.width.toFloat() / videoDimensions.height.toFloat()
                var baseWidth = maxWidth
                var baseHeight = maxWidth / aspectRatio
                if (baseHeight > maxHeight) {
                    baseHeight = maxHeight
                    baseWidth = baseHeight * aspectRatio
                }
                val fillScale = maxOf(
                    maxWidth.value / baseWidth.value.coerceAtLeast(1f),
                    maxHeight.value / baseHeight.value.coerceAtLeast(1f),
                )
                val effectiveScale = when (scaleMode) {
                    ViewScaleMode.Fit -> 1f
                    ViewScaleMode.Fill -> fillScale
                    ViewScaleMode.Manual -> uiState.config.scale
                }
                val renderedPanels = remember(uiState.monitorPanels, videoDimensions) {
                    val virtualPanels = uiState.monitorPanels
                        .filter { it.running }
                        .sortedBy { it.x }
                        .filter { it.streamId != 1 }
                        .map {
                            RenderedPanel(
                                streamId = it.streamId,
                                width = it.width.coerceAtLeast(1),
                                height = it.height.coerceAtLeast(1),
                            )
                        }
                    listOf(
                        RenderedPanel(
                            streamId = 1,
                            width = videoDimensions.width.coerceAtLeast(1),
                            height = videoDimensions.height.coerceAtLeast(1),
                        ),
                    ) + virtualPanels
                }
                if (!autoFitApplied) {
                    onScaleChanged(1f)
                    autoFitApplied = true
                }
                LaunchedEffect(settingsHandleVisible, controlsVisible) {
                    if (settingsHandleVisible && !controlsVisible) {
                        delay(1800)
                        settingsHandleVisible = false
                    }
                }
                LaunchedEffect(statusHandleVisible) {
                    if (statusHandleVisible) {
                        delay(1800)
                        statusHandleVisible = false
                    }
                }
                val onboardingHintVisible = !controlsVisible &&
                    !settingsHandleVisible &&
                    status != StreamState.Status.Streaming
                Box(modifier = Modifier.fillMaxSize()) {
                    VideoGridArea(
                        panels = renderedPanels,
                        baseWidth = baseWidth,
                        baseHeight = baseHeight,
                        scale = effectiveScale,
                        onSurfaceAvailable = onSurfaceAvailable,
                        onSurfaceDestroyed = onSurfaceDestroyed,
                        modifier = Modifier.fillMaxSize(),
                    )
                    StreamStatusBadge(
                        status = status,
                        message = uiState.streamState.message,
                        modifier = Modifier
                            .align(Alignment.TopStart)
                            .padding(spacing.medium),
                        visible = statsEnabled && statusHandleVisible && !controlsVisible,
                    )
                    CornerRevealArea(
                        modifier = Modifier
                            .align(Alignment.TopEnd)
                            .padding(spacing.small),
                        onReveal = { settingsHandleVisible = true },
                    )
                    CornerRevealArea(
                        modifier = Modifier
                            .align(Alignment.TopStart)
                            .padding(spacing.small),
                        onReveal = {
                            if (statsEnabled) {
                                statusHandleVisible = true
                            }
                        },
                    )
                    ControlsToggle(
                        expanded = controlsVisible,
                        onToggle = { controlsVisible = !controlsVisible },
                        visible = settingsHandleVisible || controlsVisible,
                        modifier = Modifier
                            .align(Alignment.TopEnd)
                            .padding(spacing.medium),
                    )
                    AnimatedVisibility(
                        visible = onboardingHintVisible,
                        enter = fadeIn() + slideInVertically(initialOffsetY = { -it / 3 }),
                        exit = fadeOut() + slideOutVertically(targetOffsetY = { -it / 4 }),
                        modifier = Modifier
                            .align(Alignment.TopCenter)
                            .padding(top = spacing.large),
                    ) {
                        FirstRunHint()
                    }
                    if (controlsVisible) {
                        ControlsPanel(
                            uiState = uiState,
                            onStartClicked = onStartClicked,
                            onStopClicked = onStopClicked,
                            scaleMode = scaleMode,
                            onScaleModeChanged = { mode -> onViewModeChanged(mode.storedValue) },
                            onScaleChanged = onScaleChanged,
                            onHostChanged = onHostChanged,
                            onStreamPortChanged = onStreamPortChanged,
                            onControlPortChanged = onControlPortChanged,
                            onAccessPinChanged = onAccessPinChanged,
                            onQrPayloadScanned = onQrPayloadScanned,
                            onAddMonitorClicked = onAddMonitorClicked,
                            onRemoveMonitorClicked = onRemoveMonitorClicked,
                            onRefreshTopologyClicked = onRefreshTopologyClicked,
                            onStartMonitorClicked = onStartMonitorClicked,
                            onStopMonitorClicked = onStopMonitorClicked,
                            statsEnabled = statsEnabled,
                            onStatsVisibilityChanged = { enabled ->
                                statsEnabled = enabled
                                onStatsOverlayVisibilityChanged(enabled)
                                if (!enabled) {
                                    statusHandleVisible = false
                                }
                            },
                            onClose = { controlsVisible = false },
                            status = status,
                            modifier = Modifier
                                .fillMaxSize()
                                .padding(spacing.large)
                                .align(Alignment.Center),
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun ControlsPanel(
    uiState: UiState,
    onStartClicked: () -> Unit,
    onStopClicked: () -> Unit,
    scaleMode: ViewScaleMode,
    onScaleModeChanged: (ViewScaleMode) -> Unit,
    onScaleChanged: (Float) -> Unit,
    onHostChanged: (String) -> Unit,
    onStreamPortChanged: (Int) -> Unit,
    onControlPortChanged: (Int) -> Unit,
    onAccessPinChanged: (String) -> Unit,
    onQrPayloadScanned: (String) -> Unit,
    onAddMonitorClicked: () -> Unit,
    onRemoveMonitorClicked: (String) -> Unit,
    onRefreshTopologyClicked: () -> Unit,
    onStartMonitorClicked: (String) -> Unit,
    onStopMonitorClicked: (String) -> Unit,
    statsEnabled: Boolean,
    onStatsVisibilityChanged: (Boolean) -> Unit,
    onClose: () -> Unit,
    status: StreamState.Status,
    modifier: Modifier = Modifier,
) {
    val spacing = MaterialTheme.spacing
    val context = LocalContext.current
    val scrollState = rememberScrollState()
    var showScanner by remember { mutableStateOf(false) }
    val cameraPermissionGranted = remember {
        mutableStateOf(
            ContextCompat.checkSelfPermission(
                context,
                Manifest.permission.CAMERA,
            ) == PackageManager.PERMISSION_GRANTED,
        )
    }
    val cameraPermissionLauncher = rememberLauncherForActivityResult(
        contract = ActivityResultContracts.RequestPermission(),
    ) { granted ->
        cameraPermissionGranted.value = granted
        if (granted) {
            showScanner = true
        }
    }
    Surface(
        modifier = modifier,
        shape = MaterialTheme.shapes.extraLarge,
        color = MaterialTheme.colorScheme.surface.copy(alpha = 0.88f),
        tonalElevation = 4.dp,
        shadowElevation = 14.dp,
    ) {
        Column(
            modifier = Modifier
                .clip(MaterialTheme.shapes.extraLarge)
                .border(
                    width = 0.5.dp,
                    color = MaterialTheme.colorScheme.outline.copy(alpha = 0.18f),
                    shape = MaterialTheme.shapes.extraLarge,
                )
                .verticalScroll(scrollState)
                .padding(spacing.large),
            verticalArrangement = Arrangement.spacedBy(spacing.medium),
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    text = "Stream controls",
                    style = MaterialTheme.typography.titleLarge,
                    color = MaterialTheme.colorScheme.onSurface,
                )
                FilledTonalButton(
                    onClick = onClose,
                    modifier = Modifier.defaultMinSize(minHeight = 44.dp),
                ) {
                    Icon(
                        imageVector = Icons.Default.KeyboardArrowDown,
                        contentDescription = "Hide controls",
                    )
                    Spacer(modifier = Modifier.width(spacing.extraSmall))
                    Text("Hide")
                }
            }
            ConnectionSettings(
                host = uiState.config.host,
                streamPort = uiState.config.streamPort,
                controlPort = uiState.controlPort,
                accessPin = uiState.config.accessPin,
                enabled = status == StreamState.Status.Idle || status == StreamState.Status.Error,
                onHostChanged = onHostChanged,
                onStreamPortChanged = onStreamPortChanged,
                onControlPortChanged = onControlPortChanged,
                onAccessPinChanged = onAccessPinChanged,
                onScanQrClicked = {
                    if (cameraPermissionGranted.value) {
                        showScanner = true
                    } else {
                        cameraPermissionLauncher.launch(Manifest.permission.CAMERA)
                    }
                },
            )
            StreamActions(
                status = status,
                receiverRunning = uiState.receiverRunning,
                errorMessage = uiState.streamState.message,
                onStartClicked = onStartClicked,
                onStopClicked = onStopClicked,
            )
            Row(
                horizontalArrangement = Arrangement.spacedBy(spacing.small),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    text = "Stats overlay",
                    style = MaterialTheme.typography.titleMedium,
                    color = MaterialTheme.colorScheme.onSurface,
                )
                OutlinedButton(onClick = { onStatsVisibilityChanged(!statsEnabled) }) {
                    Text(if (statsEnabled) "On" else "Off")
                }
            }
            MonitorControls(
                monitorPanels = uiState.monitorPanels,
                topologyBusy = uiState.topologyBusy,
                onAddMonitorClicked = onAddMonitorClicked,
                onRemoveMonitorClicked = onRemoveMonitorClicked,
                onRefreshTopologyClicked = onRefreshTopologyClicked,
                onStartMonitorClicked = onStartMonitorClicked,
                onStopMonitorClicked = onStopMonitorClicked,
            )
            ScaleControls(
                scale = uiState.config.scale,
                mode = scaleMode,
                onModeChange = onScaleModeChanged,
                onScaleChanged = onScaleChanged,
            )
            DebugConsole(logs = uiState.debugLogs)
        }
    }
    if (showScanner) {
        QrScannerSheet(
            onDismiss = { showScanner = false },
            onPayloadScanned = onQrPayloadScanned,
        )
    }
}

@Composable
private fun DebugConsole(
    logs: List<String>,
    modifier: Modifier = Modifier,
) {
    val spacing = MaterialTheme.spacing
    val scrollState = rememberScrollState()
    Surface(
        modifier = modifier.fillMaxWidth(),
        shape = MaterialTheme.shapes.large,
        color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.5f),
        tonalElevation = 2.dp,
        border = BorderStroke(0.5.dp, MaterialTheme.colorScheme.outline.copy(alpha = 0.2f)),
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(spacing.medium),
            verticalArrangement = Arrangement.spacedBy(spacing.small),
        ) {
            Text(
                text = "Debug log",
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onSurface,
            )
            Surface(
                shape = MaterialTheme.shapes.medium,
                color = MaterialTheme.colorScheme.surface.copy(alpha = 0.82f),
                border = BorderStroke(0.5.dp, MaterialTheme.colorScheme.outline.copy(alpha = 0.2f)),
            ) {
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .heightIn(min = 100.dp, max = 220.dp)
                        .verticalScroll(scrollState)
                        .padding(horizontal = spacing.small, vertical = spacing.small),
                    verticalArrangement = Arrangement.spacedBy(4.dp),
                ) {
                    if (logs.isEmpty()) {
                        Text(
                            text = "No logs yet.",
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    } else {
                        logs.takeLast(80).forEach { line ->
                            Text(
                                text = line,
                                style = MaterialTheme.typography.bodySmall,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun FirstRunHint(
    modifier: Modifier = Modifier,
) {
    Surface(
        modifier = modifier.widthIn(max = 460.dp),
        shape = MaterialTheme.shapes.large,
        color = MaterialTheme.colorScheme.surface.copy(alpha = 0.9f),
        tonalElevation = 6.dp,
        shadowElevation = 12.dp,
        border = BorderStroke(0.75.dp, MaterialTheme.colorScheme.outline.copy(alpha = 0.24f)),
    ) {
        Column(
            modifier = Modifier.padding(horizontal = 20.dp, vertical = 16.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Text(
                text = "Getting started",
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onSurface,
            )
            Text(
                text = "Hover or tap the top-right corner to open controls.",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            Text(
                text = "1. Scan host QR   2. Enter PIN   3. Start stream",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}

@Composable
private fun MonitorControls(
    monitorPanels: List<MonitorPanelState>,
    topologyBusy: Boolean,
    onAddMonitorClicked: () -> Unit,
    onRemoveMonitorClicked: (String) -> Unit,
    onRefreshTopologyClicked: () -> Unit,
    onStartMonitorClicked: (String) -> Unit,
    onStopMonitorClicked: (String) -> Unit,
) {
    val spacing = MaterialTheme.spacing
    val canAdd = monitorPanels.size < 3 && !topologyBusy
    Column(verticalArrangement = Arrangement.spacedBy(spacing.small)) {
        Surface(
            shape = MaterialTheme.shapes.large,
            color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.55f),
            tonalElevation = 2.dp,
            border = BorderStroke(0.5.dp, MaterialTheme.colorScheme.outline.copy(alpha = 0.2f)),
        ) {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(spacing.medium),
                verticalArrangement = Arrangement.spacedBy(spacing.small),
            ) {
                Text(
                    text = "Virtual monitors",
                    style = MaterialTheme.typography.titleMedium,
                    color = MaterialTheme.colorScheme.onSurface,
                )
                Text(
                    text = "${monitorPanels.size}/3 active slots",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                Row(horizontalArrangement = Arrangement.spacedBy(spacing.small)) {
                    FilledTonalButton(onClick = onAddMonitorClicked, enabled = canAdd) {
                        Text("+1 monitor")
                    }
                    OutlinedButton(onClick = onRefreshTopologyClicked, enabled = !topologyBusy) {
                        Text("Refresh")
                    }
                }
            }
        }
        if (monitorPanels.isEmpty()) {
            Surface(
                shape = MaterialTheme.shapes.medium,
                color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.35f),
                border = BorderStroke(0.5.dp, MaterialTheme.colorScheme.outline.copy(alpha = 0.16f)),
            ) {
                Text(
                    text = "No virtual monitors configured yet. Tap +1 monitor to add one.",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(spacing.medium),
                )
            }
        } else {
            monitorPanels.forEach { panel ->
                Surface(
                    shape = MaterialTheme.shapes.medium,
                    color = MaterialTheme.colorScheme.surface.copy(alpha = 0.75f),
                    tonalElevation = 1.dp,
                    border = BorderStroke(0.5.dp, MaterialTheme.colorScheme.outline.copy(alpha = 0.18f)),
                ) {
                    Column(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(spacing.medium),
                        verticalArrangement = Arrangement.spacedBy(spacing.small),
                    ) {
                        Text(
                            text = panel.displayId,
                            style = MaterialTheme.typography.titleSmall,
                            color = MaterialTheme.colorScheme.onSurface,
                        )
                        Text(
                            text = buildMonitorLine(panel),
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                        Row(
                            horizontalArrangement = Arrangement.spacedBy(spacing.small),
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            OutlinedButton(
                                onClick = {
                                    if (panel.running) onStopMonitorClicked(panel.displayId)
                                    else onStartMonitorClicked(panel.displayId)
                                },
                                enabled = !topologyBusy,
                            ) {
                                Text(if (panel.running) "Stop" else "Start")
                            }
                            OutlinedButton(
                                onClick = { onRemoveMonitorClicked(panel.displayId) },
                                enabled = !topologyBusy,
                            ) {
                                Text("Remove")
                            }
                        }
                    }
                }
            }
        }
        if (topologyBusy) {
            Text(
                text = "Applying topology...",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}

private fun buildMonitorLine(panel: MonitorPanelState): String {
    val state = if (panel.running) "running" else "stopped"
    val base = "${panel.displayId} • ${panel.width}x${panel.height} @ ${panel.x},${panel.y} • $state"
    if (!panel.running) return base
    val fps = String.format("%.2f", panel.fps)
    val bitrate = panel.bitrateKbps
    return "$base • ${fps}fps • ${bitrate}kbps"
}

@Composable
private fun ConnectionSettings(
    host: String,
    streamPort: Int,
    controlPort: Int,
    accessPin: String,
    enabled: Boolean,
    onHostChanged: (String) -> Unit,
    onStreamPortChanged: (Int) -> Unit,
    onControlPortChanged: (Int) -> Unit,
    onAccessPinChanged: (String) -> Unit,
    onScanQrClicked: () -> Unit,
) {
    val spacing = MaterialTheme.spacing
    var hostText by remember(host) { mutableStateOf(host) }
    var streamPortText by remember(streamPort) { mutableStateOf(streamPort.toString()) }
    var controlPortText by remember(controlPort) { mutableStateOf(controlPort.toString()) }
    var accessPinText by remember(accessPin) { mutableStateOf(accessPin) }
    var advancedVisible by remember { mutableStateOf(false) }
    val streamPortValue = streamPortText.toIntOrNull()
    val controlPortValue = controlPortText.toIntOrNull()
    Column(verticalArrangement = Arrangement.spacedBy(spacing.small)) {
        Text(
            text = "Sender connection",
            style = MaterialTheme.typography.titleMedium,
            color = MaterialTheme.colorScheme.onBackground,
        )
        Text(
            text = "Step 1: Scan the QR code shown on the host.",
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.End,
        ) {
            OutlinedButton(
                onClick = onScanQrClicked,
                enabled = enabled,
            ) {
                Text("Scan QR")
            }
        }
        Text(
            text = "Step 2: Enter the PIN shown on the host.",
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        OutlinedTextField(
            value = accessPinText,
            onValueChange = { value ->
                accessPinText = value
                onAccessPinChanged(value)
            },
            label = { Text("PIN / token") },
            singleLine = true,
            enabled = enabled,
            modifier = Modifier.fillMaxWidth(),
        )
        Text(
            text = "Step 3: Start the stream.",
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Text(
            text = "Sender: $host • Control: $controlPort • Stream: $streamPort",
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        OutlinedButton(
            onClick = { advancedVisible = !advancedVisible },
            enabled = enabled,
            modifier = Modifier.fillMaxWidth(),
        ) {
            Text(if (advancedVisible) "Hide advanced settings" else "Show advanced settings")
        }
        if (advancedVisible) {
            OutlinedTextField(
                value = hostText,
                onValueChange = { value ->
                    hostText = value
                    onHostChanged(value)
                },
                label = { Text("Sender IP / host") },
                singleLine = true,
                enabled = enabled,
                modifier = Modifier.fillMaxWidth(),
            )
            OutlinedTextField(
                value = streamPortText,
                onValueChange = { value ->
                    streamPortText = value
                    value.toIntOrNull()?.let(onStreamPortChanged)
                },
                label = { Text("Stream port (client receiver)") },
                singleLine = true,
                enabled = enabled,
                isError = streamPortValue == null,
                modifier = Modifier.fillMaxWidth(),
            )
            OutlinedTextField(
                value = controlPortText,
                onValueChange = { value ->
                    controlPortText = value
                    value.toIntOrNull()?.let(onControlPortChanged)
                },
                label = { Text("Control port (host)") },
                singleLine = true,
                enabled = enabled,
                isError = controlPortValue == null,
                modifier = Modifier.fillMaxWidth(),
            )
            if (streamPortValue == null) {
                Text(
                    text = "Stream port must be a number.",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.error,
                )
            }
            if (controlPortValue == null) {
                Text(
                    text = "Control port must be a number.",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.error,
                )
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun QrScannerSheet(
    onDismiss: () -> Unit,
    onPayloadScanned: (String) -> Unit,
) {
    val context = LocalContext.current
    val lifecycleOwner = LocalLifecycleOwner.current
    val scanner = remember { BarcodeScanning.getClient() }
    val previewView = remember { PreviewView(context) }
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)
    val executor = remember { Executors.newSingleThreadExecutor() }
    val mainExecutor = remember { ContextCompat.getMainExecutor(context) }
    var cameraProvider by remember { mutableStateOf<ProcessCameraProvider?>(null) }
    var hasScanned by remember { mutableStateOf(false) }

    DisposableEffect(Unit) {
        onDispose {
            cameraProvider?.unbindAll()
            scanner.close()
            executor.shutdown()
        }
    }

    LaunchedEffect(previewView, lifecycleOwner) {
        val provider = context.getCameraProvider()
        cameraProvider = provider
        val preview = Preview.Builder().build().also {
            it.setSurfaceProvider(previewView.surfaceProvider)
        }
        val analysis = ImageAnalysis.Builder()
            .setBackpressureStrategy(ImageAnalysis.STRATEGY_KEEP_ONLY_LATEST)
            .build()
        analysis.setAnalyzer(executor) { imageProxy ->
            if (hasScanned) {
                imageProxy.close()
                return@setAnalyzer
            }
            val mediaImage = imageProxy.image
            if (mediaImage == null) {
                imageProxy.close()
                return@setAnalyzer
            }
            val inputImage =
                InputImage.fromMediaImage(mediaImage, imageProxy.imageInfo.rotationDegrees)
            scanner.process(inputImage)
                .addOnSuccessListener(mainExecutor) { barcodes ->
                    if (hasScanned) {
                        return@addOnSuccessListener
                    }
                    val payload = barcodes.firstOrNull { !it.rawValue.isNullOrBlank() }?.rawValue
                    if (payload != null) {
                        hasScanned = true
                        onPayloadScanned(payload)
                        onDismiss()
                    }
                }
                .addOnCompleteListener(mainExecutor) {
                    imageProxy.close()
                }
        }
        provider.unbindAll()
        provider.bindToLifecycle(
            lifecycleOwner,
            CameraSelector.DEFAULT_BACK_CAMERA,
            preview,
            analysis,
        )
    }

    ModalBottomSheet(
        onDismissRequest = onDismiss,
        sheetState = sheetState,
        dragHandle = null,
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 16.dp, vertical = 12.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Text(
                text = "Scan QR code",
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onSurface,
            )
            Text(
                text = "Align the QR code within the frame to autofill the host, control port, and any available stream port or PIN.",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            AndroidView(
                factory = { previewView },
                modifier = Modifier
                    .fillMaxWidth()
                    .height(320.dp),
            )
            OutlinedButton(
                onClick = onDismiss,
                modifier = Modifier.fillMaxWidth(),
            ) {
                Text("Close")
            }
        }
    }
}

@Composable
private fun StreamActions(
    status: StreamState.Status,
    receiverRunning: Boolean,
    errorMessage: String?,
    onStartClicked: () -> Unit,
    onStopClicked: () -> Unit,
) {
    val spacing = MaterialTheme.spacing
    if (receiverRunning) {
        OutlinedButton(onClick = onStopClicked, modifier = Modifier.fillMaxWidth()) {
            Text("Stop stream")
        }
        return
    }
    when (status) {
        StreamState.Status.Idle -> {
            FilledTonalButton(onClick = onStartClicked, modifier = Modifier.fillMaxWidth()) {
                Text("Start stream")
            }
        }

        StreamState.Status.Connecting -> {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(spacing.small),
            ) {
                CircularProgressIndicator(
                    color = MaterialTheme.colorScheme.primary,
                    strokeWidth = 2.dp,
                )
                Text(
                    text = "Connecting...",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurface,
                )
            }
            OutlinedButton(onClick = onStopClicked, modifier = Modifier.fillMaxWidth()) {
                Text("Cancel")
            }
        }

        StreamState.Status.Streaming -> {
            OutlinedButton(onClick = onStopClicked, modifier = Modifier.fillMaxWidth()) {
                Text("Stop stream")
            }
        }

        StreamState.Status.Error -> {
            Text(
                text = "Stream error: ${errorMessage ?: "Unknown"}",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.error,
            )
            FilledTonalButton(onClick = onStartClicked, modifier = Modifier.fillMaxWidth()) {
                Text("Retry")
            }
        }
    }
}

@Composable
private fun ScaleControls(
    scale: Float,
    mode: ViewScaleMode,
    onModeChange: (ViewScaleMode) -> Unit,
    onScaleChanged: (Float) -> Unit,
    modifier: Modifier = Modifier,
) {
    val spacing = MaterialTheme.spacing
    Column(
        modifier = modifier,
        verticalArrangement = Arrangement.spacedBy(spacing.small),
    ) {
        Text(
            text = "View mode",
            style = MaterialTheme.typography.titleMedium,
            color = MaterialTheme.colorScheme.onSurface,
        )
        Row(
            horizontalArrangement = Arrangement.spacedBy(spacing.small),
        ) {
            OutlinedButton(
                onClick = { onModeChange(ViewScaleMode.Fit) },
                enabled = mode != ViewScaleMode.Fit,
            ) {
                Text("Fit")
            }
            OutlinedButton(
                onClick = { onModeChange(ViewScaleMode.Fill) },
                enabled = mode != ViewScaleMode.Fill,
            ) {
                Text("Fill")
            }
            OutlinedButton(
                onClick = { onModeChange(ViewScaleMode.Manual) },
                enabled = mode != ViewScaleMode.Manual,
            ) {
                Text("Manual")
            }
        }
        if (mode != ViewScaleMode.Manual) {
            Text(
                text = if (mode == ViewScaleMode.Fit) {
                    "Fit shows the full desktop without cropping."
                } else {
                    "Fill uses all available area and may crop edges."
                },
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            return
        }
        Text(
            text = "Scale: ${String.format("%.2f", scale)}x",
            style = MaterialTheme.typography.titleMedium,
            color = MaterialTheme.colorScheme.onSurface,
        )
        Row(
            verticalAlignment = Alignment.CenterVertically,
        ) {
            OutlinedButton(
                onClick = {
                    onScaleChanged((scale - 0.1f).coerceAtLeast(SCALE_MIN))
                },
            ) {
                Text("-")
            }
            Spacer(modifier = Modifier.width(spacing.small))
            Slider(
                value = scale,
                onValueChange = onScaleChanged,
                valueRange = SCALE_MIN..SCALE_MAX,
                modifier = Modifier.weight(1f),
            )
            Spacer(modifier = Modifier.width(spacing.small))
            OutlinedButton(
                onClick = {
                    onScaleChanged((scale + 0.1f).coerceAtMost(SCALE_MAX))
                },
            ) {
                Text("+")
            }
        }
        Text(
            text = "Adjust scale between ${SCALE_MIN}x and ${SCALE_MAX}x.",
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}

@Composable
private fun StreamStatusBadge(
    status: StreamState.Status,
    message: String?,
    modifier: Modifier = Modifier,
    visible: Boolean = true,
) {
    if (!visible) return
    val label = when (status) {
        StreamState.Status.Idle -> "Idle"
        StreamState.Status.Connecting -> "Connecting"
        StreamState.Status.Streaming -> "Streaming"
        StreamState.Status.Error -> "Error"
    }
    Surface(
        modifier = modifier,
        shape = MaterialTheme.shapes.small,
        color = MaterialTheme.colorScheme.surface.copy(alpha = 0.9f),
        tonalElevation = 2.dp,
        shadowElevation = 4.dp,
    ) {
        Column(modifier = Modifier.padding(horizontal = 12.dp, vertical = 6.dp)) {
            Text(
                text = label,
                style = MaterialTheme.typography.labelLarge,
                color = MaterialTheme.colorScheme.onSurface,
            )
            if (!message.isNullOrBlank()) {
                Text(
                    text = message,
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
    }
}

@Composable
private fun ControlsToggle(
    expanded: Boolean,
    onToggle: () -> Unit,
    visible: Boolean,
    modifier: Modifier = Modifier,
) {
    if (!visible) return
    Surface(
        modifier = modifier
            .clickable(onClick = onToggle),
        shape = MaterialTheme.shapes.extraLarge,
        color = MaterialTheme.colorScheme.surface.copy(alpha = 0.7f),
        tonalElevation = 1.dp,
        shadowElevation = 6.dp,
        border = BorderStroke(
            0.5.dp,
            MaterialTheme.colorScheme.outline.copy(alpha = 0.12f),
        ),
    ) {
        Icon(
            imageVector = Icons.Default.Settings,
            contentDescription = if (expanded) "Hide controls" else "Show controls",
            tint = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier
                .defaultMinSize(minWidth = 40.dp, minHeight = 40.dp)
                .padding(8.dp),
        )
    }
}

@Composable
private fun CornerRevealArea(
    modifier: Modifier = Modifier,
    onReveal: () -> Unit,
) {
    Box(
        modifier = modifier
            .width(56.dp)
            .height(56.dp)
            .pointerInteropFilter { event ->
                when (event.actionMasked) {
                    MotionEvent.ACTION_HOVER_ENTER,
                    MotionEvent.ACTION_HOVER_MOVE,
                    MotionEvent.ACTION_DOWN -> {
                        onReveal()
                        false
                    }

                    else -> false
                }
            },
    )
}

private data class RenderedPanel(
    val streamId: Int,
    val width: Int,
    val height: Int,
)

@Composable
private fun VideoGridArea(
    panels: List<RenderedPanel>,
    baseWidth: Dp,
    baseHeight: Dp,
    scale: Float,
    onSurfaceAvailable: (Int, Surface) -> Unit,
    onSurfaceDestroyed: (Int) -> Unit,
    modifier: Modifier = Modifier,
) {
    if (panels.size <= 1) {
        val panel = panels.firstOrNull() ?: RenderedPanel(1, DEFAULT_REMOTE_WIDTH, DEFAULT_REMOTE_HEIGHT)
        VideoArea(
            streamId = panel.streamId,
            baseWidth = baseWidth,
            baseHeight = baseHeight,
            width = panel.width,
            height = panel.height,
            scale = scale,
            onSurfaceAvailable = onSurfaceAvailable,
            onSurfaceDestroyed = onSurfaceDestroyed,
            modifier = modifier,
        )
        return
    }

    Box(
        modifier = modifier,
        contentAlignment = Alignment.Center,
    ) {
        Column(
            modifier = Modifier
                .width(baseWidth)
                .height(baseHeight)
                .graphicsLayer(scaleX = scale, scaleY = scale),
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            val topPanels = panels.take(2)
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .weight(1f),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                topPanels.forEach { panel ->
                    VideoArea(
                        streamId = panel.streamId,
                        baseWidth = 1.dp,
                        baseHeight = 1.dp,
                        width = panel.width,
                        height = panel.height,
                        scale = 1f,
                        useBaseSize = false,
                        onSurfaceAvailable = onSurfaceAvailable,
                        onSurfaceDestroyed = onSurfaceDestroyed,
                        modifier = Modifier
                            .weight(1f)
                            .fillMaxSize(),
                    )
                }
            }
            if (panels.size >= 3) {
                val panel = panels[2]
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .weight(1f),
                    horizontalArrangement = Arrangement.Center,
                ) {
                    VideoArea(
                        streamId = panel.streamId,
                        baseWidth = 1.dp,
                        baseHeight = 1.dp,
                        width = panel.width,
                        height = panel.height,
                        scale = 1f,
                        useBaseSize = false,
                        onSurfaceAvailable = onSurfaceAvailable,
                        onSurfaceDestroyed = onSurfaceDestroyed,
                        modifier = Modifier
                            .weight(0.8f)
                            .fillMaxSize(),
                    )
                }
            }
        }
    }
}

@Composable
fun VideoArea(
    streamId: Int,
    baseWidth: Dp,
    baseHeight: Dp,
    width: Int,
    height: Int,
    scale: Float,
    useBaseSize: Boolean = true,
    onSurfaceAvailable: (Int, Surface) -> Unit,
    onSurfaceDestroyed: (Int) -> Unit,
    modifier: Modifier = Modifier,
) {
    val currentOnSurfaceAvailable by rememberUpdatedState(onSurfaceAvailable)
    val currentOnSurfaceDestroyed by rememberUpdatedState(onSurfaceDestroyed)
    Box(
        modifier = modifier,
        contentAlignment = Alignment.Center,
    ) {
        val surfaceModifier = if (useBaseSize) {
            Modifier
                .width(baseWidth)
                .height(baseHeight)
        } else {
            Modifier.fillMaxSize()
        }
        Surface(
            modifier = surfaceModifier.graphicsLayer(scaleX = scale, scaleY = scale),
            color = MaterialTheme.colorScheme.surfaceVariant,
            tonalElevation = 0.dp,
            shadowElevation = 0.dp,
        ) {
            AndroidView(
                factory = { context ->
                    SurfaceView(context).apply {
                        holder.addCallback(
                            object : SurfaceHolder.Callback {
                                override fun surfaceCreated(holder: SurfaceHolder) {
                                    currentOnSurfaceAvailable(streamId, holder.surface)
                                }

                                override fun surfaceChanged(
                                    holder: SurfaceHolder,
                                    format: Int,
                                    width: Int,
                                    height: Int,
                                ) = Unit

                                override fun surfaceDestroyed(holder: SurfaceHolder) {
                                    currentOnSurfaceDestroyed(streamId)
                                }
                            },
                        )
                        holder.setFixedSize(width, height)
                    }
                },
                update = { view ->
                    view.holder.setFixedSize(width, height)
                },
                modifier = Modifier.fillMaxSize(),
            )
        }
    }
}

private suspend fun Context.getCameraProvider(): ProcessCameraProvider {
    return suspendCoroutine { continuation ->
        val future = ProcessCameraProvider.getInstance(this)
        future.addListener(
            { continuation.resume(future.get()) },
            ContextCompat.getMainExecutor(this),
        )
    }
}
