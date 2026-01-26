package com.parallax.receiver.presentation.ui

import android.view.Surface
import android.view.SurfaceHolder
import android.view.SurfaceView
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
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
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Slider
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberSaveable
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.viewinterop.AndroidView
import com.parallax.receiver.core.config.DEFAULT_REMOTE_HEIGHT
import com.parallax.receiver.core.config.DEFAULT_REMOTE_WIDTH
import com.parallax.receiver.core.config.SCALE_MAX
import com.parallax.receiver.core.config.SCALE_MIN
import com.parallax.receiver.domain.model.StreamState
import com.parallax.receiver.domain.model.UiState
import com.parallax.receiver.presentation.theme.spacing
import kotlin.math.min

@Composable
fun StreamScreen(
    uiState: UiState,
    onStartClicked: () -> Unit,
    onStopClicked: () -> Unit,
    onScaleChanged: (Float) -> Unit,
    onHostChanged: (String) -> Unit,
    onPortChanged: (Int) -> Unit,
    onSurfaceAvailable: (Surface) -> Unit,
    onSurfaceDestroyed: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val spacing = MaterialTheme.spacing
    val status = uiState.streamState.status
    var controlsVisible by rememberSaveable { mutableStateOf(false) }
    var autoFitApplied by rememberSaveable { mutableStateOf(false) }
    Surface(
        modifier = modifier.fillMaxSize(),
        color = MaterialTheme.colorScheme.background,
        tonalElevation = 0.dp,
    ) {
        BoxWithConstraints(modifier = Modifier.fillMaxSize()) {
            val aspectRatio = remoteWidth.toFloat() / remoteHeight.toFloat()
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
                        onPortChanged = onPortChanged,
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
    onPortChanged: (Int) -> Unit,
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
                port = uiState.config.port,
                enabled = status == StreamState.Status.Idle || status == StreamState.Status.Error,
                onHostChanged = onHostChanged,
                onPortChanged = onPortChanged,
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
    port: Int,
    enabled: Boolean,
    onHostChanged: (String) -> Unit,
    onPortChanged: (Int) -> Unit,
) {
    val spacing = MaterialTheme.spacing
    var hostText by remember(host) { mutableStateOf(host) }
    var portText by remember(port) { mutableStateOf(port.toString()) }
    val portValue = portText.toIntOrNull()
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
            value = portText,
            onValueChange = { value ->
                portText = value
                value.toIntOrNull()?.let(onPortChanged)
            },
            label = { Text("Port") },
            singleLine = true,
            enabled = enabled,
            isError = portValue == null,
            modifier = Modifier.fillMaxWidth(),
        )
        if (portValue == null) {
            Text(
                text = "Port must be a number.",
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
    scale: Float,
    modifier: Modifier = Modifier,
) {
    val currentOnSurfaceAvailable by rememberUpdatedState(onSurfaceAvailable)
    val currentOnSurfaceDestroyed by rememberUpdatedState(onSurfaceDestroyed)
    Box(
        modifier = modifier,
        contentAlignment = Alignment.Center,
    ) {
        Text(
            text = label,
            style = MaterialTheme.typography.labelMedium.copy(fontWeight = FontWeight.SemiBold),
            color = color,
            modifier = Modifier.padding(horizontal = 8.dp, vertical = 4.dp),
        )
    }
}

@Composable
private fun DebugPanel(
    uiState: UiState,
    onClose: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val spacing = MaterialTheme.spacing
    Surface(
        modifier = modifier,
        color = MaterialTheme.colorScheme.surface.copy(alpha = 0.92f),
        shape = MaterialTheme.shapes.medium,
        tonalElevation = 2.dp,
        shadowElevation = 4.dp,
    ) {
        Column(
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
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween,
                modifier = Modifier.fillMaxWidth(),
            ) {
                Text(
                    text = "Debug panel",
                    style = MaterialTheme.typography.titleSmall,
                    color = MaterialTheme.colorScheme.onSurface,
                )
                IconButton(onClick = onClose, modifier = Modifier.size(24.dp)) {
                    Icon(
                        imageVector = Icons.Default.Close,
                        contentDescription = "Close debug panel",
                    )
                }
            }
            Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {
                Text(
                    text = "Status: ${uiState.streamState.status}",
                    style = MaterialTheme.typography.bodySmall,
                )
                Text(
                    text = "Endpoint: ${uiState.config.host}:${uiState.config.port}",
                    style = MaterialTheme.typography.bodySmall,
                )
                Text(
                    text = "Scale: ${String.format("%.2f", uiState.config.scale)}x",
                    style = MaterialTheme.typography.bodySmall,
                )
                if (uiState.streamState.message != null) {
                    Text(
                        text = "Message: ${uiState.streamState.message}",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
        }
    }
}

@Composable
private fun ControlPanel(
    uiState: UiState,
    status: StreamState.Status,
    onStartClicked: () -> Unit,
    onStopClicked: () -> Unit,
    onScaleChanged: (Float) -> Unit,
    onHostChanged: (String) -> Unit,
    onPortChanged: (Int) -> Unit,
    modifier: Modifier = Modifier,
) {
    val spacing = MaterialTheme.spacing
    Surface(
        modifier = modifier.fillMaxHeight(),
        color = MaterialTheme.colorScheme.surface,
        tonalElevation = 4.dp,
        shadowElevation = 6.dp,
        shape = MaterialTheme.shapes.large,
    ) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(spacing.large),
            verticalArrangement = Arrangement.spacedBy(spacing.large),
        ) {
            Text(
                text = "Stream controls",
                style = MaterialTheme.typography.titleLarge,
                color = MaterialTheme.colorScheme.onSurface,
            )
            ConnectionSettings(
                host = uiState.config.host,
                port = uiState.config.port,
                enabled = status == StreamState.Status.Idle || status == StreamState.Status.Error,
                onHostChanged = onHostChanged,
                onPortChanged = onPortChanged,
            )
            when (status) {
                StreamState.Status.Idle -> {
                    Text(
                        text = "Ready to connect to ${uiState.config.host}:${uiState.config.port}.",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                    FilledTonalButton(onClick = onStartClicked) {
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
                            text = "Connecting... (simulated delay)",
                            style = MaterialTheme.typography.bodyMedium,
                        )
                    }
                },
                modifier = Modifier.fillMaxSize(),
            )
        }
    }
}
