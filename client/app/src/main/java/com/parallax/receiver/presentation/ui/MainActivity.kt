package com.parallax.receiver.presentation.ui

import android.content.Context
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.viewModels
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import com.parallax.receiver.dal.local.SharedPreferencesSettingsStore
import com.parallax.receiver.domain.module.SetScaleUseCase
import com.parallax.receiver.domain.module.SetViewModeUseCase
import com.parallax.receiver.domain.module.SetStreamEndpointUseCase
import com.parallax.receiver.domain.module.StartStreamUseCase
import com.parallax.receiver.domain.module.StopStreamUseCase
import com.parallax.receiver.domain.service.StreamSessionService
import com.parallax.receiver.presentation.theme.ReceiverTheme
import com.parallax.receiver.presentation.vm.StreamViewModel

class MainActivity : ComponentActivity() {
    private val streamViewModel: StreamViewModel by viewModels {
        StreamViewModelFactory(this)
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        handleIntentPayload(intent)
        setContent {
            ReceiverTheme {
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.background,
                    tonalElevation = 0.dp,
                ) {
                    val uiState by streamViewModel.uiState.collectAsState()
                    StreamScreen(
                        uiState = uiState,
                        uiEvents = streamViewModel.uiEvents,
                        onStartClicked = streamViewModel::onStartClicked,
                        onStopClicked = streamViewModel::onStopClicked,
                        onScaleChanged = streamViewModel::onScaleChanged,
                        onViewModeChanged = streamViewModel::onViewModeChanged,
                        onHostChanged = streamViewModel::onHostChanged,
                        onStreamPortChanged = streamViewModel::onStreamPortChanged,
                        onControlPortChanged = streamViewModel::onControlPortChanged,
                        onAccessPinChanged = streamViewModel::onAccessPinChanged,
                        onQrPayloadScanned = streamViewModel::onQrPayloadScanned,
                        onSurfaceAvailable = streamViewModel::onSurfaceAvailable,
                        onSurfaceDestroyed = streamViewModel::onSurfaceDestroyed,
                        onAddMonitorClicked = streamViewModel::onAddMonitorClicked,
                        onRemoveMonitorClicked = streamViewModel::onRemoveMonitorClicked,
                        onRefreshTopologyClicked = streamViewModel::onRefreshTopologyClicked,
                        onStartMonitorClicked = streamViewModel::onStartMonitorClicked,
                        onStopMonitorClicked = streamViewModel::onStopMonitorClicked,
                        onStatsOverlayVisibilityChanged = streamViewModel::onStatsOverlayVisibilityChanged,
                    )
                }
            }
        }
    }

    override fun onNewIntent(intent: android.content.Intent) {
        super.onNewIntent(intent)
        handleIntentPayload(intent)
    }

    private fun handleIntentPayload(intent: android.content.Intent?) {
        val payload = intent?.dataString ?: return
        streamViewModel.onQrPayloadScanned(payload)
    }
}

private class StreamViewModelFactory(
    private val context: Context,
) : ViewModelProvider.Factory {
    override fun <T : ViewModel> create(modelClass: Class<T>): T {
        if (modelClass.isAssignableFrom(StreamViewModel::class.java)) {
            val sharedPreferences = context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
            val settingsStore = SharedPreferencesSettingsStore(sharedPreferences)
            val streamSessionService = StreamSessionService(settingsStore = settingsStore)
            val startStreamUseCase = StartStreamUseCase(streamSessionService)
            val stopStreamUseCase = StopStreamUseCase(streamSessionService)
            val setScaleUseCase = SetScaleUseCase(settingsStore, streamSessionService)
            val setViewModeUseCase = SetViewModeUseCase(settingsStore, streamSessionService)
            val setStreamEndpointUseCase = SetStreamEndpointUseCase(settingsStore, streamSessionService)
            @Suppress("UNCHECKED_CAST")
            return StreamViewModel(
                streamSessionService = streamSessionService,
                startStream = startStreamUseCase,
                stopStream = stopStreamUseCase,
                setScale = setScaleUseCase,
                setViewMode = setViewModeUseCase,
                setStreamEndpoint = setStreamEndpointUseCase,
            ) as T
        }
        throw IllegalArgumentException("Unknown ViewModel class: ${modelClass.name}")
    }

    private companion object {
        private const val PREFS_NAME = "receiver.settings"
    }
}
