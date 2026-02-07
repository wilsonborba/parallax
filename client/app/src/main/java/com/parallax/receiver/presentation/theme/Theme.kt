package com.parallax.receiver.presentation.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider

private val DarkColorScheme = darkColorScheme(
    primary = Neutral95,
    onPrimary = Neutral20,
    secondary = Neutral80,
    onSecondary = Neutral10,
    tertiary = AccentWarm,
    onTertiary = Neutral100,
    background = Neutral05,
    onBackground = Neutral95,
    surface = Neutral10,
    onSurface = Neutral95,
    surfaceVariant = Neutral20,
    onSurfaceVariant = Neutral80,
    outline = Neutral30,
)

private val LightColorScheme = lightColorScheme(
    primary = Neutral30,
    onPrimary = Neutral100,
    secondary = Neutral40,
    onSecondary = Neutral100,
    tertiary = AccentWarm,
    onTertiary = Neutral100,
    background = Neutral98,
    onBackground = Neutral10,
    surface = Neutral100,
    onSurface = Neutral10,
    surfaceVariant = Neutral95,
    onSurfaceVariant = Neutral40,
    outline = Neutral80,
)

@Composable
fun ReceiverTheme(
    darkTheme: Boolean = isSystemInDarkTheme(),
    content: @Composable () -> Unit
) {
    val colorScheme = if (darkTheme) DarkColorScheme else LightColorScheme

    CompositionLocalProvider(LocalSpacing provides Spacing()) {
        MaterialTheme(
            colorScheme = colorScheme,
            typography = Typography,
            content = content
        )
    }
}
