package com.parallax.receiver.presentation.ui

import android.view.Surface
import android.view.SurfaceHolder
import android.view.SurfaceView
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Slider
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.viewinterop.AndroidView
import com.parallax.receiver.core.config.DEFAULT_REMOTE_HEIGHT
import com.parallax.receiver.core.config.DEFAULT_REMOTE_WIDTH
import com.parallax.receiver.core.config.SCALE_MAX
import com.parallax.receiver.core.config.SCALE_MIN
import com.parallax.receiver.domain.model.StreamState
import com.parallax.receiver.domain.model.UiState
import com.parallax.receiver.presentation.theme.spacing

@Composable
fun StreamScreen(
    uiState: UiState,
    onStartClicked: () -> Unit,
    onStopClicked: () -> Unit,
    onScaleChanged: (Float) -> Unit,
    onHostChanged: (String) -> Unit,
    onStreamPortChanged: (Int) -> Unit,
    onControlPortChanged: (Int) -> Unit,
    onAccessPinChanged: (String) -> Unit,
    onPairingTokenChanged: (String) -> Unit,
    onSurfaceAvailable: (Surface) -> Unit,
    onSurfaceDestroyed: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val spacing = MaterialTheme.spacing
    val status = uiState.streamState.status
    var controlsVisible by remember { mutableStateOf(false) }
    var autoFitApplied by remember { mutableStateOf(false) }
    Surface(
        modifier = modifier.fillMaxSize(),
        color = MaterialTheme.colorScheme.background,
        tonalElevation = 0.dp,
    ) {
        BoxWithConstraints(modifier = Modifier.fillMaxSize()) {
            val aspectRatio = DEFAULT_REMOTE_WIDTH.toFloat() / DEFAULT_REMOTE_HEIGHT.toFloat()
            var baseWidth = maxWidth
            var baseHeight = maxWidth / aspectRatio
            if (baseHeight > maxHeight) {
                baseHeight = maxHeight
                baseWidth = baseHeight * aspectRatio
            }
            if (!autoFitApplied) {
                onScaleChanged(1f)
                autoFitApplied = true
            }
            Box(modifier = Modifier.fillMaxSize()) {
                VideoArea(
                    baseWidth = baseWidth,
                    baseHeight = baseHeight,
                    scale = uiState.config.scale,
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
                )
                ControlsToggle(
                    expanded = controlsVisible,
                    onToggle = { controlsVisible = !controlsVisible },
                    modifier = Modifier
                        .align(Alignment.TopEnd)
                        .padding(spacing.medium),
                )
                if (controlsVisible) {
                    ControlsPanel(
                        uiState = uiState,
                        onStartClicked = onStartClicked,
                        onStopClicked = onStopClicked,
                        onScaleChanged = onScaleChanged,
                        onHostChanged = onHostChanged,
                        onStreamPortChanged = onStreamPortChanged,
                        onControlPortChanged = onControlPortChanged,
                        onAccessPinChanged = onAccessPinChanged,
                        onPairingTokenChanged = onPairingTokenChanged,
                        status = status,
                        modifier = Modifier
                            .align(Alignment.BottomEnd)
                            .padding(spacing.large)
                            .widthIn(max = 360.dp)
                            .fillMaxWidth(),
                    )
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
    onScaleChanged: (Float) -> Unit,
    onHostChanged: (String) -> Unit,
    onStreamPortChanged: (Int) -> Unit,
    onControlPortChanged: (Int) -> Unit,
    onAccessPinChanged: (String) -> Unit,
    onPairingTokenChanged: (String) -> Unit,
    status: StreamState.Status,
    modifier: Modifier = Modifier,
) {
    val spacing = MaterialTheme.spacing
    Surface(
        modifier = modifier,
        shape = MaterialTheme.shapes.large,
        color = MaterialTheme.colorScheme.surface.copy(alpha = 0.95f),
        tonalElevation = 4.dp,
        shadowElevation = 8.dp,
    ) {
        Column(
            modifier = Modifier.padding(spacing.large),
            verticalArrangement = Arrangement.spacedBy(spacing.medium),
        ) {
            Text(
                text = "Stream controls",
                style = MaterialTheme.typography.titleLarge,
                color = MaterialTheme.colorScheme.onSurface,
            )
            ConnectionSettings(
                host = uiState.config.host,
                streamPort = uiState.config.streamPort,
                controlPort = uiState.config.controlPort,
                accessPin = uiState.config.accessPin,
                pairingToken = uiState.pairingToken,
                enabled = status == StreamState.Status.Idle || status == StreamState.Status.Error,
                onHostChanged = onHostChanged,
                onStreamPortChanged = onStreamPortChanged,
                onControlPortChanged = onControlPortChanged,
                onAccessPinChanged = onAccessPinChanged,
                onPairingTokenChanged = onPairingTokenChanged,
            )
            StreamActions(
                status = status,
                errorMessage = uiState.streamState.message,
                onStartClicked = onStartClicked,
                onStopClicked = onStopClicked,
            )
            ScaleControls(
                scale = uiState.config.scale,
                onScaleChanged = onScaleChanged,
            )
        }
    }
}

@Composable
private fun ConnectionSettings(
    host: String,
    streamPort: Int,
    controlPort: Int,
    accessPin: String,
    pairingToken: String,
    enabled: Boolean,
    onHostChanged: (String) -> Unit,
    onStreamPortChanged: (Int) -> Unit,
    onControlPortChanged: (Int) -> Unit,
    onAccessPinChanged: (String) -> Unit,
    onPairingTokenChanged: (String) -> Unit,
) {
    val spacing = MaterialTheme.spacing
    var hostText by remember(host) { mutableStateOf(host) }
    var streamPortText by remember(streamPort) { mutableStateOf(streamPort.toString()) }
    var controlPortText by remember(controlPort) { mutableStateOf(controlPort.toString()) }
    var accessPinText by remember(accessPin) { mutableStateOf(accessPin) }
    var pairingTokenText by remember(pairingToken) { mutableStateOf(pairingToken) }
    val streamPortValue = streamPortText.toIntOrNull()
    val controlPortValue = controlPortText.toIntOrNull()
    Column(verticalArrangement = Arrangement.spacedBy(spacing.small)) {
        Text(
            text = "Sender connection",
            style = MaterialTheme.typography.titleMedium,
            color = MaterialTheme.colorScheme.onBackground,
        )
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
            label = { Text("Stream port") },
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
            label = { Text("Control port") },
            singleLine = true,
            enabled = enabled,
            isError = controlPortValue == null,
            modifier = Modifier.fillMaxWidth(),
        )
        OutlinedTextField(
            value = accessPinText,
            onValueChange = { value ->
                accessPinText = value
                onAccessPinChanged(value)
            },
            label = { Text("Access PIN") },
            singleLine = true,
            enabled = enabled,
            modifier = Modifier.fillMaxWidth(),
        )
        OutlinedTextField(
            value = pairingTokenText,
            onValueChange = { value ->
                pairingTokenText = value
                onPairingTokenChanged(value)
            },
            label = { Text("Pairing token") },
            singleLine = true,
            enabled = enabled,
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

@Composable
private fun StreamActions(
    status: StreamState.Status,
    errorMessage: String?,
    onStartClicked: () -> Unit,
    onStopClicked: () -> Unit,
) {
    val spacing = MaterialTheme.spacing
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
    onScaleChanged: (Float) -> Unit,
    modifier: Modifier = Modifier,
) {
    val spacing = MaterialTheme.spacing
    Column(
        modifier = modifier,
        verticalArrangement = Arrangement.spacedBy(spacing.small),
    ) {
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
) {
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
    modifier: Modifier = Modifier,
) {
    Surface(
        modifier = modifier.clickable(onClick = onToggle),
        shape = MaterialTheme.shapes.small,
        color = MaterialTheme.colorScheme.surface.copy(alpha = 0.9f),
        tonalElevation = 2.dp,
        shadowElevation = 6.dp,
    ) {
        Text(
            text = if (expanded) "Hide" else "Menu",
            style = MaterialTheme.typography.labelLarge,
            color = MaterialTheme.colorScheme.onSurface,
            modifier = Modifier.padding(horizontal = 12.dp, vertical = 8.dp),
        )
    }
}

@Composable
fun VideoArea(
    baseWidth: Dp,
    baseHeight: Dp,
    scale: Float,
    onSurfaceAvailable: (Surface) -> Unit,
    onSurfaceDestroyed: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val currentOnSurfaceAvailable by rememberUpdatedState(onSurfaceAvailable)
    val currentOnSurfaceDestroyed by rememberUpdatedState(onSurfaceDestroyed)
    Box(
        modifier = modifier,
        contentAlignment = Alignment.Center,
    ) {
        Surface(
            modifier = Modifier
                .width(baseWidth)
                .height(baseHeight)
                .graphicsLayer(scaleX = scale, scaleY = scale)
                .border(
                    width = 1.dp,
                    color = MaterialTheme.colorScheme.outline,
                    shape = MaterialTheme.shapes.medium,
                ),
            color = MaterialTheme.colorScheme.surfaceVariant,
            shape = MaterialTheme.shapes.medium,
            tonalElevation = 0.dp,
            shadowElevation = 0.dp,
        ) {
            AndroidView(
                factory = { context ->
                    SurfaceView(context).apply {
                        holder.addCallback(
                            object : SurfaceHolder.Callback {
                                override fun surfaceCreated(holder: SurfaceHolder) {
                                    currentOnSurfaceAvailable(holder.surface)
                                }

                                override fun surfaceChanged(
                                    holder: SurfaceHolder,
                                    format: Int,
                                    width: Int,
                                    height: Int,
                                ) = Unit

                                override fun surfaceDestroyed(holder: SurfaceHolder) {
                                    currentOnSurfaceDestroyed()
                                }
                            },
                        )
                    }
                },
                modifier = Modifier.fillMaxSize(),
            )
        }
    }
}
