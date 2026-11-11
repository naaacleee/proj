package com.mentor.feature.main.goals

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import com.mentor.core.ui.components.PlaceholderScreen

@Composable
fun GoalsScreen(modifier: Modifier = Modifier) {
    PlaceholderScreen(
        modifier = modifier,
        title = "Цели",
        description = "Здесь появятся ваши цели, их декомпозиция на шаги и прогресс.\n" +
            "Раздел входит в Фазу 1 (MVP).",
    )
}
