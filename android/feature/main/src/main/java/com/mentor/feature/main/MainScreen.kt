package com.mentor.feature.main

import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Chat
import androidx.compose.material.icons.filled.Flag
import androidx.compose.material.icons.filled.Home
import androidx.compose.material.icons.filled.Schedule
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material3.Icon
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import com.mentor.core.network.ApiService
import com.mentor.core.network.SseChatClient
import com.mentor.core.network.TokenStore
import com.mentor.feature.main.chat.ChatScreen
import com.mentor.feature.main.goals.GoalsScreen
import com.mentor.feature.main.home.HomeScreen
import com.mentor.feature.main.schedule.ScheduleScreen
import com.mentor.feature.main.settings.SettingsScreen

private data class Tab(val route: String, val label: String, val icon: ImageVector)

private val tabs = listOf(
    Tab("home", "Главная", Icons.Default.Home),
    Tab("chat", "Чат", Icons.Default.Chat),
    Tab("goals", "Цели", Icons.Default.Flag),
    Tab("schedule", "Расписание", Icons.Default.Schedule),
    Tab("settings", "Настройки", Icons.Default.Settings),
)

@Composable
fun MainScreen(
    api: ApiService,
    tokenStore: TokenStore,
    sseClient: SseChatClient,
    baseUrlProvider: () -> String,
    onSaveBaseUrl: (String) -> Unit,
    onLoggedOut: () -> Unit,
) {
    var currentTab by rememberSaveable { mutableStateOf(tabs.first().route) }

    Scaffold(
        bottomBar = {
            NavigationBar {
                tabs.forEach { tab ->
                    NavigationBarItem(
                        selected = currentTab == tab.route,
                        onClick = { currentTab = tab.route },
                        icon = { Icon(tab.icon, contentDescription = tab.label) },
                        label = { Text(tab.label) },
                    )
                }
            }
        },
    ) { padding ->
        val contentModifier = Modifier.padding(padding)
        when (currentTab) {
            "home" -> HomeScreen(contentModifier, tokenStore.username ?: "друг")
            "chat" -> ChatScreen(
                modifier = contentModifier,
                api = api,
                tokenStore = tokenStore,
                sseClient = sseClient,
                baseUrlProvider = baseUrlProvider,
            )
            "goals" -> GoalsScreen(contentModifier)
            "schedule" -> ScheduleScreen(contentModifier)
            "settings" -> SettingsScreen(
                modifier = contentModifier,
                tokenStore = tokenStore,
                currentBaseUrl = baseUrlProvider(),
                onSaveBaseUrl = onSaveBaseUrl,
                onLoggedOut = onLoggedOut,
            )
        }
    }
}
