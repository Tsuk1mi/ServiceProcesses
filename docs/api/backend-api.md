# Backend API Guide

Документ описывает API ядра системы для подключения клиентских приложений:
- Windows desktop
- Android mobile
- Web client

Базовый URL:
- локально: `http://localhost:8080`
- в k8s (пример): `http://backend.service-processes.svc.cluster.local`

## 1. Общие правила интеграции

- Формат данных: JSON.
- Все запросы отправляются с `Content-Type: application/json`.
- Для операций изменения данных требуется заголовок `x-role`.
- Поддерживаемые роли:
  - `dispatcher`
  - `technician`
  - `supervisor`
  - `viewer`
- Ошибки возвращаются в формате:

```json
{
  "message": "описание ошибки"
}
```

Пример:

```http
x-role: dispatcher
```

Для роли `technician` в операциях `start/complete` наряда обязателен заголовок:

```http
x-actor-id: tech-<id>
```

Техник может начать/завершить только наряд, где он назначен исполнителем.

## 2. Endpoints

### 2.1 Health check

`GET /health`

Ответ `200 OK`:

```text
ok
```

---

### 2.2 Создать объект обслуживания

`POST /api/v1/assets`

Тело запроса:

```json
{
  "kind": "building",
  "title": "Склад N2",
  "location": "Санкт-Петербург"
}
```

Успешный ответ `201 Created`:

```json
{
  "id": "asset-<uuid>",
  "kind": "building",
  "title": "Склад N2",
  "location": "Санкт-Петербург",
  "state": "Active"
}
```

---

### 2.3 Получить список объектов обслуживания

`GET /api/v1/assets`

Успешный ответ `200 OK`:

```json
[
  {
    "id": "asset-<uuid>",
    "kind": "building",
    "title": "Склад N2",
    "location": "Санкт-Петербург",
    "state": "Active"
  }
]
```

---

### 2.4 Создать заявку

`POST /api/v1/requests`

Тело запроса:

```json
{
  "asset_id": "asset-<uuid>",
  "description": "Срочно: отказ системы питания"
}
```

Успешный ответ `201 Created`:

```json
{
  "result": "created"
}
```

Примечание:
- Приоритет и SLA рассчитываются в ядре автоматически.
- Событие `service_request.created` публикуется через `EventPublisherPort`.

---

### 2.5 Получить список заявок

`GET /api/v1/requests`

Query-параметры:
- `limit` (опционально)
- `offset` (опционально)
- `status` (опционально): `New`, `Planned`, `InProgress`, `Resolved`, `Closed`, `Escalated`
- `priority` (опционально): `Low`, `Medium`, `High`, `Critical`

Успешный ответ `200 OK`:

```json
[
  {
    "id": "req-<uuid>",
    "asset_id": "asset-<uuid>",
    "description": "Срочно: отказ системы питания",
    "priority": "High",
    "status": "New",
    "sla_minutes": 240
  }
]
```

---

### 2.5.1 Получить просроченные заявки

`GET /api/v1/requests/overdue`

Query-параметры:
- `limit` (опционально)
- `offset` (опционально)

Успешный ответ `200 OK`:

```json
[
  {
    "id": "req-<uuid>",
    "asset_id": "asset-<uuid>",
    "description": "Срочно: отказ системы питания",
    "priority": "High",
    "status": "New",
    "sla_minutes": 240,
    "created_at_epoch_sec": 1742380000
  }
]
```

---

### 2.6 Обновить статус заявки

`PUT /api/v1/requests/{id}/status`

Тело запроса:

```json
{
  "status": "in_progress"
}
```

Поддерживаемые статусы:
- `new`
- `planned`
- `in_progress`
- `resolved`
- `closed`
- `escalated`

Успешный ответ `200 OK`:

```json
{
  "result": "updated"
}
```

---

### 2.7 Создать наряд (Work Order)

`POST /api/v1/work-orders`

Тело запроса:

```json
{
  "request_id": "req-<uuid>"
}
```

Успешный ответ `201 Created`:

```json
{
  "id": "wo-<uuid>",
  "request_id": "req-<uuid>",
  "assignee": null,
  "status": "Created"
}
```

---

### 2.8 Получить наряды по заявке

`GET /api/v1/requests/{id}/work-orders`

Query-параметры:
- `limit` (опционально)
- `offset` (опционально)

Успешный ответ `200 OK`:

```json
[
  {
    "id": "wo-<uuid>",
    "request_id": "req-<uuid>",
    "assignee": null,
    "status": "Created"
  }
]
```

---

### 2.9 Назначить исполнителя на наряд

`PUT /api/v1/work-orders/{id}/assign`

Тело запроса:

```json
{
  "assignee": "tech-1"
}
```

Успешный ответ `200 OK`:

```json
{
  "id": "wo-<uuid>",
  "request_id": "req-<uuid>",
  "assignee": "tech-1",
  "status": "Assigned"
}
```

---

### 2.10 Начать выполнение наряда

`PUT /api/v1/work-orders/{id}/start`

Успешный ответ `200 OK`:

```json
{
  "id": "wo-<uuid>",
  "request_id": "req-<uuid>",
  "assignee": "tech-1",
  "status": "InProgress"
}
```

---

### 2.11 Завершить наряд

`PUT /api/v1/work-orders/{id}/complete`

Успешный ответ `200 OK`:

```json
{
  "id": "wo-<uuid>",
  "request_id": "req-<uuid>",
  "assignee": "tech-1",
  "status": "Completed"
}
```

---

### 2.12 Создать эскалацию

`POST /api/v1/escalations`

Тело запроса:

```json
{
  "request_id": "req-<uuid>",
  "reason": "Нарушение времени реакции SLA"
}
```

Успешный ответ `201 Created`:

```json
{
  "id": "esc-<uuid>",
  "request_id": "req-<uuid>",
  "reason": "Нарушение времени реакции SLA",
  "state": "Open"
}
```

---

### 2.13 Закрыть эскалацию

`PUT /api/v1/escalations/{id}/resolve`

Успешный ответ `200 OK`:

```json
{
  "id": "esc-<uuid>",
  "request_id": "req-<uuid>",
  "reason": "Нарушение времени реакции SLA",
  "state": "Resolved"
}
```

---

### 2.13.1 Автоэскалация просроченных заявок

`POST /api/v1/sla/escalate-overdue`

Требуемая роль: `dispatcher` или `supervisor`.

Успешный ответ `200 OK`:

```json
{
  "created": 2
}
```

---

### 2.14 Получить эскалации по заявке

`GET /api/v1/requests/{id}/escalations`

Query-параметры:
- `limit` (опционально)
- `offset` (опционально)

Успешный ответ `200 OK`:

```json
[
  {
    "id": "esc-<uuid>",
    "request_id": "req-<uuid>",
    "reason": "Нарушение времени реакции SLA",
    "state": "Open"
  }
]
```

---

### 2.15 Создать техника

`POST /api/v1/technicians`

Тело запроса:

```json
{
  "full_name": "Иван Иванов",
  "skills": ["electrical", "inspection"]
}
```

Успешный ответ `201 Created`:

```json
{
  "id": "tech-<uuid>",
  "full_name": "Иван Иванов",
  "skills": ["electrical", "inspection"],
  "is_active": true
}
```

---

### 2.16 Получить список техников

`GET /api/v1/technicians`

Query-параметры:
- `limit` (опционально)
- `offset` (опционально)

Успешный ответ `200 OK`:

```json
[
  {
    "id": "tech-<uuid>",
    "full_name": "Иван Иванов",
    "skills": ["electrical", "inspection"],
    "is_active": true
  }
]
```

---

### 2.17 Получить аудит по заявке

`GET /api/v1/requests/{id}/audit`

Query-параметры:
- `limit` (опционально)
- `offset` (опционально)

Успешный ответ `200 OK`:

```json
[
  {
    "id": "aud-<id>",
    "request_id": "req-<uuid>",
    "entity": "work_order",
    "action": "assign",
    "actor_role": "dispatcher",
    "actor_id": "disp-1",
    "details": "work_order_id=wo-<uuid>,assignee=tech-1",
    "created_at_utc": "1742380000"
  }
]
```

## 3. Бизнес-правила статусов

Разрешенные переходы:
- `New -> Planned`
- `Planned -> InProgress`
- `InProgress -> Resolved`
- `Resolved -> Closed`
- `* -> Escalated`

При недопустимом переходе возвращается `409 Conflict`.

## 4. Коды ответов

- `200 OK` - успешное чтение/обновление.
- `201 Created` - сущность создана.
- `400 Bad Request` - некорректные входные данные.
- `404 Not Found` - сущность не найдена.
- `403 Forbidden` - недостаточно прав или действие запрещено правилами.
- `409 Conflict` - конфликт бизнес-правил (например, переход статуса).

## 5. Рекомендации для UI-команд

- Перед созданием заявок кэшировать список объектов (`GET /api/v1/assets`) на стороне клиента.
- После `POST /api/v1/requests` делать обновление списка заявок (`GET /api/v1/requests`).
- Для карточек заявок отображать `priority`, `status`, `sla_minutes`.
- Использовать единый клиент API SDK для Windows, Android и Web, чтобы унифицировать обработку ошибок.

## 6. Режимы контейнеров backend

- `APP_MODE=api` - запуск HTTP API.
- `APP_MODE=worker` - запуск SLA worker для автоэскалации просроченных заявок.

Параметры worker:
- `WORKER_INTERVAL_SEC` - интервал проверки просрочки в секундах.
