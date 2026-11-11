package com.mentor.app.navigation

import androidx.compose.runtime.Composable
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import com.mentor.app.AppContainer
import com.mentor.feature.auth.AuthScreen
import com.mentor.feature.main.MainScreen

private object Routes {
    const val AUTH = "auth"
    const val MAIN = "main"
}

@Composable
fun MentorNavHost(container: AppContainer) {
    val navController = rememberNavController()
    val startDestination = if (container.tokenStore.hasSession()) Routes.MAIN else Routes.AUTH

    NavHost(navController = navController, startDestination = startDestination) {
        composable(Routes.AUTH) {
            AuthScreen(
                api = container.api,
                tokenStore = container.tokenStore,
                onLoggedIn = {
                    navController.navigate(Routes.MAIN) {
                        popUpTo(Routes.AUTH) { inclusive = true }
                    }
                },
            )
        }
        composable(Routes.MAIN) {
            MainScreen(
                api = container.api,
                tokenStore = container.tokenStore,
                sseClient = container.sseClient,
                baseUrlProvider = { container.currentBaseUrl() },
                onSaveBaseUrl = { container.updateBaseUrl(it) },
                onLoggedOut = {
                    navController.navigate(Routes.AUTH) {
                        popUpTo(Routes.MAIN) { inclusive = true }
                    }
                },
            )
        }
    }
}
