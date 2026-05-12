package com.servicesphere.mobile.ui.screens

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.servicesphere.mobile.data.model.MobileRequestDetailDto
import com.servicesphere.mobile.ui.RequestDetailUiState

@Composable
fun RequestDetailScreen(
    state: RequestDetailUiState,
    onRefresh: () -> Unit,
    canChangeStatus: Boolean,
    onChangeStatus: (String) -> Unit
) {
    Column(
        modifier = Modifier
            .padding(16.dp)
            .verticalScroll(rememberScrollState()),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        Text("Карточка заявки", style = MaterialTheme.typography.headlineSmall)

        if (state.error != null) {
            Text(state.error, color = MaterialTheme.colorScheme.error)
        }

        val item = state.item
        if (item == null) {
            Text("Загрузка заявки...")
            return@Column
        }

        RequestDetailCard(item)

        Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            Button(onClick = onRefresh, modifier = Modifier.weight(1f)) {
                Text("Обновить")
            }
            if (canChangeStatus) {
                StatusTransitionButtons(
                    modifier = Modifier.weight(1f),
                    status = item.status,
                    onChangeStatus = onChangeStatus
                )
            }
        }

        if (!canChangeStatus) {
            Text(
                "В мобильном приложении клиентам доступны просмотр и создание заявок. Операционные переходы статусов вынесены в web/desktop.",
                style = MaterialTheme.typography.bodyMedium
            )
        }
    }
}

@Composable
private fun RequestDetailCard(item: MobileRequestDetailDto) {
    Card(modifier = Modifier.fillMaxWidth()) {
        Column(modifier = Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Text(item.title, style = MaterialTheme.typography.titleLarge)
            Text("ID: ${item.ticketId}")
            Text("Статус: ${item.status}")
            Text("Приоритет: ${item.priority}")
            Text("Исполнитель: ${item.assignee ?: "не назначен"}")
            Text("Просрочено: ${if (item.overdue) "да" else "нет"}")
            Spacer(modifier = Modifier.height(4.dp))
            Text(item.description)
        }
    }
}

@Composable
private fun StatusTransitionButtons(
    modifier: Modifier = Modifier,
    status: String,
    onChangeStatus: (String) -> Unit
) {
    val nextStatuses = when (status.uppercase()) {
        "NEW" -> listOf("planned" to "В план")
        "PLANNED" -> listOf("in_progress" to "В работу")
        "IN_PROGRESS" -> listOf("resolved" to "Решена")
        "RESOLVED" -> listOf("closed" to "Закрыть")
        else -> emptyList()
    }

    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(8.dp)) {
        if (nextStatuses.isEmpty()) {
            Text("Доступных переходов нет", style = MaterialTheme.typography.bodyMedium)
        } else {
            nextStatuses.forEach { (statusValue, label) ->
                Button(
                    onClick = { onChangeStatus(statusValue) },
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Text(label)
                }
            }
        }
    }
}
