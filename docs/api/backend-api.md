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
- Текущее API без авторизации (MVP), в следующих итерациях будет добавлен JWT/RBAC.
- Ошибки возвращаются в формате:

```json
{
  "message": "описание ошибки"
}
```

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
- `409 Conflict` - конфликт бизнес-правил (например, переход статуса).

## 5. Рекомендации для UI-команд

- Перед созданием заявок кэшировать список объектов (`GET /api/v1/assets`) на стороне клиента.
- После `POST /api/v1/requests` делать обновление списка заявок (`GET /api/v1/requests`).
- Для карточек заявок отображать `priority`, `status`, `sla_minutes`.
- Использовать единый клиент API SDK для Windows, Android и Web, чтобы унифицировать обработку ошибок.
