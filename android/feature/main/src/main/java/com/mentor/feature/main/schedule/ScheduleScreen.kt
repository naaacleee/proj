package com.mentor.feature.main.schedule

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import com.mentor.core.ui.components.PlaceholderScreen

@Composable
fun ScheduleScreen(modifier: Modifier = Modifier) {
    PlaceholderScreen(
        modifier = modifier,
        title = "Расписание",
        description = "Недельное расписание с постоянными занятиями и временными " +
            "переносами появится в Фазе 1.",
    )
}
