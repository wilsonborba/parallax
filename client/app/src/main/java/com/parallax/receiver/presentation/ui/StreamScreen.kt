package com.parallax.receiver.presentation.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Button
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Slider
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import com.parallax.receiver.core.config.DEFAULT_REMOTE_HEIGHT
import com.parallax.receiver.core.config.DEFAULT_REMOTE_WIDTH
import com.parallax.receiver.core.config.SCALE_MAX
import com.parallax.receiver.core.config.SCALE_MIN
import com.parallax.receiver.domain.model.StreamState
import com.parallax.receiver.domain.model.UiState

@Composable
fun StreamScreen(
    uiState: UiState,
    onStartClicked: () -> Unit,
    onStopClicked: () -> Unit,
    onScaleChanged: (Float) -> Unit,
    remoteWidth: Int = DEFAULT_REMOTE_WIDTH,
    remoteHeight: Int = DEFAULT_REMOTE_HEIGHT,
    modifier: Modifier = Modifier,
) {
    val status = uiState.streamState.status
    Column(
        modifier = modifier
            .fillMaxSize()
            .padding(24.dp),
        verticalArrangement = Arrangement.spacedBy(20.dp),
    ) {
        Text(
            text = "Parallax Stream",
            style = MaterialTheme.typography.headlineSmall,
        )
        when (status) {
            StreamState.Status.Idle -> {
                Text("Ready to connect to ${uiState.config.host}:${uiState.config.port}.")
                Button(onClick = onStartClicked) {
                    Text("Start stream")
                }
            }

            StreamState.Status.Connecting -> {
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(12.dp),
                ) {
                    CircularProgressIndicator()
                    Text("Connecting... (simulated delay)")
                }
                Button(onClick = onStopClicked) {
                    Text("Cancel")
                }
            }

            StreamState.Status.Streaming -> {
                VideoArea(
                    remoteWidth = remoteWidth,
                    remoteHeight = remoteHeight,
                    scale = uiState.config.scale,
                    onScaleChanged = onScaleChanged,
                )
                Button(onClick = onStopClicked) {
                    Text("Stop stream")
                }
            }

            StreamState.Status.Error -> {
                Text("Stream error: ${uiState.streamState.message ?: "Unknown"}")
                Button(onClick = onStartClicked) {
                    Text("Retry")
                }
            }
        }
    }
}

@Composable
fun VideoArea(
    remoteWidth: Int,
    remoteHeight: Int,
    scale: Float,
    onScaleChanged: (Float) -> Unit,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier = modifier,
        verticalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .aspectRatio(remoteWidth.toFloat() / remoteHeight.toFloat())
                .background(Color.DarkGray),
            contentAlignment = Alignment.Center,
        ) {
            Text("Video feed", color = Color.White)
        }
        Text("Scale: ${String.format("%.2f", scale)}x")
        Row(
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Button(
                onClick = {
                    onScaleChanged((scale - 0.1f).coerceAtLeast(SCALE_MIN))
                },
            ) {
                Text("-")
            }
            Spacer(modifier = Modifier.width(12.dp))
            Slider(
                value = scale,
                onValueChange = onScaleChanged,
                valueRange = SCALE_MIN..SCALE_MAX,
                modifier = Modifier.weight(1f),
            )
            Spacer(modifier = Modifier.width(12.dp))
            Button(
                onClick = {
                    onScaleChanged((scale + 0.1f).coerceAtMost(SCALE_MAX))
                },
            ) {
                Text("+")
            }
        }
        Spacer(modifier = Modifier.height(8.dp))
        Text("Adjust scale between ${SCALE_MIN}x and ${SCALE_MAX}x.")
    }
}
