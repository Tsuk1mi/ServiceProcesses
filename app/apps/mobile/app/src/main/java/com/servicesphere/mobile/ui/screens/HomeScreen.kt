package com.servicesphere.mobile.ui.screens

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
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
import com.servicesphere.mobile.data.model.MobileRequestCardDto
import com.servicesphere.mobile.ui.HomeUiState

@Composable
fun HomeScreen(
    state: HomeUiState,
    userRole: String,
    onRefresh: () -> Unit,
    onOpenRequest: (String) -> Unit,
    onOpenRequests: () -> Unit,
    onOpenCreate: () -> Unit
) {
    LazyColumn(modifier = Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
        item {
            Text("Главная", style = MaterialTheme.typography.headlineSmall)
            Text(
                if (userRole.equals("ADMIN", ignoreCase = true))
                    "Режим администратора: клиентский кабинет плюс базовые админские функции."
                else
                    "Режим клиента: самообслуживание, заявки и уведомления."
            )
        }

        item {
            Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                CounterCard("Открытые", state.counters.openRequests, Modifier.weight(1f))
                CounterCard("Просрочено", state.counters.overdueRequests, Modifier.weight(1f))
                CounterCard("Активные наряды", state.counters.activeWorkOrders, Modifier.weight(1f))
            }
        }

        item {
            Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                Button(onClick = onRefresh, modifier = Modifier.weight(1f)) {
                    Text("Обновить")
                }
                Button(onClick = onOpenCreate, modifier = Modifier.weight(1f)) {
                    Text("Новая заявка")
                }
            }
        }

        item {
            Text("Последние заявки", style = MaterialTheme.typography.titleMedium)
        }

        if (state.error != null) {
            item {
                Text(state.error, color = MaterialTheme.colorScheme.error)
            }
        }

        if (state.recentRequests.isEmpty()) {
            item {
                Card {
                    Column(modifier = Modifier.padding(16.dp)) {
                        Text("Данных пока нет", style = MaterialTheme.typography.titleMedium)
                        Spacer(modifier = Modifier.height(6.dp))
                        Text("Создайте объект и первую заявку в системе. После этого mobile home начнет показывать counters и recent requests.")
                        Spacer(modifier = Modifier.height(12.dp))
                        Button(onClick = onOpenRequests) {
                            Text("Открыть список заявок")
                        }
                    }
                }
            }
        } else {
            items(state.recentRequests) { item ->
                RequestCard(item = item, onOpenRequest = onOpenRequest)
            }
        }
    }
}

@Composable
private fun CounterCard(label: String, value: Int, modifier: Modifier = Modifier) {
    Card(modifier = modifier) {
        Column(modifier = Modifier.padding(16.dp)) {
            Text(label, style = MaterialTheme.typography.bodyMedium)
            Spacer(modifier = Modifier.height(8.dp))
            Text(value.toString(), style = MaterialTheme.typography.headlineMedium)
        }
    }
}

@Composable
private fun RequestCard(item: MobileRequestCardDto, onOpenRequest: (String) -> Unit) {
    Card(
        modifier = Modifier
            .fillMaxWidth()
            .clickable { onOpenRequest(item.ticketId) }
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            Text(item.title, style = MaterialTheme.typography.titleMedium)
            Spacer(modifier = Modifier.height(4.dp))
            Text(item.description, style = MaterialTheme.typography.bodyMedium)
            Spacer(modifier = Modifier.height(8.dp))
            Text("Статус: ${item.status}")
        }
    }
}
