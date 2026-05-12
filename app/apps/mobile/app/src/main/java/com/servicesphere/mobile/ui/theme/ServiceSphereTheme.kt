package com.servicesphere.mobile.ui.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

private val LightColors = lightColorScheme(
    primary = Color(0xFF0F766E),
    onPrimary = Color.White,
    secondary = Color(0xFF10202D),
    onSecondary = Color.White,
    tertiary = Color(0xFF0369A1),
    background = Color(0xFFF2F6F9),
    onBackground = Color(0xFF12202D),
    surface = Color.White,
    onSurface = Color(0xFF12202D),
    surfaceVariant = Color(0xFFECF2F6),
    outline = Color(0xFFD8E2EA),
    error = Color(0xFFBE123C)
)

private val DarkColors = darkColorScheme(
    primary = Color(0xFF34D399),
    onPrimary = Color(0xFF082A28),
    secondary = Color(0xFFB6C7D3),
    onSecondary = Color(0xFF10202D),
    tertiary = Color(0xFF7DD3FC),
    background = Color(0xFF0D1720),
    onBackground = Color(0xFFEEF6FB),
    surface = Color(0xFF142331),
    onSurface = Color(0xFFEEF6FB),
    surfaceVariant = Color(0xFF1B3041),
    outline = Color(0xFF2C4254),
    error = Color(0xFFFDA4AF)
)

@Composable
fun ServiceSphereTheme(content: @Composable () -> Unit) {
    MaterialTheme(
        colorScheme = if (isSystemInDarkTheme()) DarkColors else LightColors,
        content = content
    )
}
