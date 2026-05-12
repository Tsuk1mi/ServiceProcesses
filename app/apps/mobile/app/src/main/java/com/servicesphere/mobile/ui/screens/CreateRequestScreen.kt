package com.servicesphere.mobile.ui.screens

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.servicesphere.mobile.ui.CreateRequestUiState

@Composable
fun CreateRequestScreen(
    state: CreateRequestUiState,
    onCreateRequest: (String, String) -> Unit
) {
    var title by rememberSaveable { mutableStateOf("") }
    var description by rememberSaveable { mutableStateOf("") }

    Column(
        modifier = Modifier.padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        Text("Новая заявка", style = MaterialTheme.typography.headlineSmall)
        Text("Экран самообслуживания для клиентов и администратора. Заявка создается через mobile BFF. Если объектов в системе нет, backend вернет ошибку.")

        Card(modifier = Modifier.fillMaxWidth()) {
            Column(modifier = Modifier.padding(16.dp)) {
                OutlinedTextField(
                    value = title,
                    onValueChange = { title = it },
                    modifier = Modifier.fillMaxWidth(),
                    label = { Text("Заголовок") }
                )
                Spacer(modifier = Modifier.height(12.dp))
                OutlinedTextField(
                    value = description,
                    onValueChange = { description = it },
                    modifier = Modifier.fillMaxWidth(),
                    label = { Text("Описание") }
                )
                Spacer(modifier = Modifier.height(12.dp))

                if (state.error != null) {
                    Text(state.error, color = MaterialTheme.colorScheme.error)
                    Spacer(modifier = Modifier.height(8.dp))
                }

                if (state.successMessage != null) {
                    Text(state.successMessage, color = MaterialTheme.colorScheme.primary)
                    Spacer(modifier = Modifier.height(8.dp))
                }

                Button(
                    onClick = { onCreateRequest(title, description) },
                    enabled = !state.saving,
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Text(if (state.saving) "Отправка..." else "Создать заявку")
                }
            }
        }
    }
}
