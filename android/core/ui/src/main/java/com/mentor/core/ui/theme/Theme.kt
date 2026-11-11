package com.mentor.core.ui.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

// Спокойная минималистичная палитра: нейтральный фон, один акцент.
private val LightColors = lightColorScheme(
    primary = Color(0xFF3D5AFE),
    onPrimary = Color.White,
    background = Color(0xFFF6F6F8),
    surface = Color.White,
    onBackground = Color(0xFF16161A),
    onSurface = Color(0xFF16161A),
    surfaceVariant = Color(0xFFEDEDF2),
    onSurfaceVariant = Color(0xFF5C5C66),
)

private val DarkColors = darkColorScheme(
    primary = Color(0xFF9DB0FF),
    onPrimary = Color(0xFF0B1B4D),
    background = Color(0xFF101014),
    surface = Color(0xFF1A1A20),
    onBackground = Color(0xFFECECF1),
    onSurface = Color(0xFFECECF1),
    surfaceVariant = Color(0xFF232329),
    onSurfaceVariant = Color(0xFFB4B4BE),
)

@Composable
fun MentorTheme(
    darkTheme: Boolean = isSystemInDarkTheme(),
    content: @Composable () -> Unit,
) {
    MaterialTheme(
        colorScheme = if (darkTheme) DarkColors else LightColors,
        content = content,
    )
}
