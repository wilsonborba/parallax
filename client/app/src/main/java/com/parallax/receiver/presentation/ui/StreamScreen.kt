package com.parallax.receiver.presentation.ui

import android.view.Surface
import android.view.SurfaceHolder
import android.view.SurfaceView
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
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
import androidx.compose.ui.text.input.KeyboardOptions
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.input.KeyboardType
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
    onPortChanged: (Int) -> Unit,
    onSurfaceAvailable: (Surface) -> Unit,
    onSurfaceDestroyed: () -> Unit,
    remoteWidth: Int = DEFAULT_REMOTE_WIDTH,
    remoteHeight: Int = DEFAULT_REMOTE_HEIGHT,
    modifier: Modifier = Modifier,
) {
    val spacing = MaterialTheme.spacing
    val status = uiState.streamState.status
    Surface(
        modifier = modifier.fillMaxSize(),
        color = MaterialTheme.colorScheme.background,
        tonalElevation = 0.dp,
    ) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(spacing.extraLarge),
            verticalArrangement = Arrangement.spacedBy(spacing.large),
        ) {
            Text(
                text = "Parallax Stream",
                style = MaterialTheme.typography.headlineSmall,
                color = MaterialTheme.colorScheme.onBackground,
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
                    OutlinedButton(onClick = onStopClicked) {
                        Text("Cancel")
                    }
                }

                StreamState.Status.Streaming -> {
                    VideoArea(
                        remoteWidth = remoteWidth,
                        remoteHeight = remoteHeight,
                        scale = uiState.config.scale,
                        onScaleChanged = onScaleChanged,
                        onSurfaceAvailable = onSurfaceAvailable,
                        onSurfaceDestroyed = onSurfaceDestroyed,
                    )
                    OutlinedButton(onClick = onStopClicked) {
                        Text("Stop stream")
                    }
                }

                StreamState.Status.Error -> {
                    Text(
                        text = "Stream error: ${uiState.streamState.message ?: "Unknown"}",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                    FilledTonalButton(onClick = onStartClicked) {
                        Text("Retry")
                    }
                }
            }
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
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
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
fun VideoArea(
    remoteWidth: Int,
    remoteHeight: Int,
    scale: Float,
    onScaleChanged: (Float) -> Unit,
    onSurfaceAvailable: (Surface) -> Unit,
    onSurfaceDestroyed: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val spacing = MaterialTheme.spacing
    val currentOnSurfaceAvailable by rememberUpdatedState(onSurfaceAvailable)
    val currentOnSurfaceDestroyed by rememberUpdatedState(onSurfaceDestroyed)
    Column(
        modifier = modifier,
        verticalArrangement = Arrangement.spacedBy(spacing.medium),
    ) {
        Surface(
            modifier = Modifier
                .fillMaxWidth()
                .aspectRatio(remoteWidth.toFloat() / remoteHeight.toFloat())
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
        Text(
            text = "Scale: ${String.format("%.2f", scale)}x",
            style = MaterialTheme.typography.titleLarge,
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
        Spacer(modifier = Modifier.height(spacing.extraSmall))
        Text(
            text = "Adjust scale between ${SCALE_MIN}x and ${SCALE_MAX}x.",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Text(
            text = "Rendering via SurfaceView.",
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center,
        )
    }
}
