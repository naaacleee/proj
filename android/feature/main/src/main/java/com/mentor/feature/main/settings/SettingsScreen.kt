package com.mentor.feature.main.settings

import android.widget.Toast
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import com.mentor.core.network.TokenStore

@Composable
fun SettingsScreen(
    modifier: Modifier = Modifier,
    tokenStore: TokenStore,
    currentBaseUrl: String,
    onSaveBaseUrl: (String) -> Unit,
    onLoggedOut: () -> Unit,
) {
    val context = LocalContext.current
    var baseUrl by rememberSaveable { mutableStateOf(currentBaseUrl) }

    Column(
        modifier = modifier
            .fillMaxSize()
            .padding(20.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        Text("Настройки", style = MaterialTheme.typography.headlineSmall)

        Text(
            "Аккаунт: ${tokenStore.username ?: "—"}",
            style = MaterialTheme.typography.bodyMedium,
        )

        OutlinedTextField(
            value = baseUrl,
            onValueChange = { baseUrl = it },
            label = { Text("Адрес бэкенда") },
            singleLine = true,
            modifier = Modifier.fillMaxWidth(),
        )
        Button(
            onClick = {
                onSaveBaseUrl(baseUrl)
                Toast.makeText(
                    context,
                    "Сохранено. Новый адрес применяется сразу.",
                    Toast.LENGTH_LONG,
                ).show()
            },
            modifier = Modifier.fillMaxWidth(),
        ) {
            Text("Сохранить адрес")
        }

        Spacer(Modifier.height(8.dp))

        OutlinedButton(
            onClick = {
                tokenStore.clear()
                onLoggedOut()
            },
            modifier = Modifier.fillMaxWidth(),
        ) {
            Text("Выйти из аккаунта")
        }

        Spacer(Modifier.height(8.dp))
        Text(
            "Ментор 0.1.0 (Фаза 0)",
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}
