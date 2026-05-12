package com.servicesphere.mobile.ui.screens

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.servicesphere.mobile.ui.RequestListUiState

@Composable
fun RequestListScreen(
    state: RequestListUiState,
    onRefresh: () -> Unit,
    onOpenRequest: (String) -> Unit
) {
    LazyColumn(
        modifier = Modifier.padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        item {
            Text("Заявки", style = MaterialTheme.typography.headlineSmall)
            Text("Реальный список из mobile BFF.")
        }

        item {
            Button(onClick = onRefresh) {
                Text("Обновить список")
            }
        }

        if (state.error != null) {
            item {
                Text(state.error, color = MaterialTheme.colorScheme.error)
            }
        }

        if (state.items.isEmpty()) {
            item {
                Card {
                    Column(modifier = Modifier.padding(16.dp)) {
                        Text("Заявок пока нет", style = MaterialTheme.typography.titleMedium)
                        Text("После создания заявки она появится здесь и на главной.")
                    }
                }
            }
        } else {
            items(state.items) { item ->
                Card(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clickable { onOpenRequest(item.ticketId) }
                ) {
                    Column(modifier = Modifier.padding(16.dp)) {
                        Text(item.title, style = MaterialTheme.typography.titleMedium)
                        Text(item.description, style = MaterialTheme.typography.bodyMedium)
                        Text("Статус: ${item.status}")
                        Text("ID: ${item.ticketId}")
                    }
                }
            }
        }
    }
}
