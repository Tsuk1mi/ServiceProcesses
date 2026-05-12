package com.servicesphere.mobile.ui.screens

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.servicesphere.mobile.core.Config
import com.servicesphere.mobile.ui.SessionUiState

@Composable
fun ProfileScreen(
    state: SessionUiState,
    onRefresh: () -> Unit,
    onLogout: () -> Unit
) {
    Column(
        modifier = Modifier.padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        Text("Профиль", style = MaterialTheme.typography.headlineSmall)

        Card(modifier = Modifier.fillMaxWidth()) {
            Column(
                modifier = Modifier.padding(16.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                Text("Пользователь: ${state.userName}")
                Text("Email: ${state.userEmail}")
                Text("Роль: ${state.userRole}")
                Text(
                    if (state.userRole.equals("ADMIN", ignoreCase = true))
                        "Доступ: клиентская зона + админские функции"
                    else
                        "Доступ: только клиентская зона"
                )
                Text("Источник данных: реальный backend API")
                Text("API base URL: ${Config.baseUrl}")
            }
        }

        Button(onClick = onRefresh, modifier = Modifier.fillMaxWidth()) {
            Text("Обновить данные")
        }
        Button(onClick = onLogout, modifier = Modifier.fillMaxWidth()) {
            Text("Выйти")
        }
    }
}
