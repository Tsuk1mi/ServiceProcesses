# Backend Core (Rust, Hexagonal)

## Запуск локально

```bash
cargo run
```

Сервер слушает `http://0.0.0.0:8080` (локально: `http://localhost:8080`).

### Режимы (`APP_MODE`)

| Значение | Назначение |
|----------|------------|
| `api` (по умолчанию) | HTTP API |
| `worker` | SLA-воркер: просрочки, снимок аналитики |
| `queue_worker` | Потребитель RabbitMQ для фоновых задач (`/api/v1/jobs`) |

### Переменные окружения (важные)

| Переменная | Описание |
|------------|----------|
| `JWT_SECRET` | Секрет подписи JWT (в проде задать явно). |
| `REDIS_URL` + `RABBITMQ_URL` | Вместе включают `POST/GET /api/v1/jobs`; иначе эти маршруты отвечают 503. |
| `JOB_QUEUE_NAME` | Очередь RabbitMQ (по умолчанию `service_jobs`). |
| `WORKER_INTERVAL_SEC` | Интервал SLA-воркера (секунды). |
| `RUST_LOG` | Например `info` или `info,tower_http=debug`. |

Docker-стек: см. `infra/docker/docker-compose.yml` и `docs/server-stack.md`.

## Аутентификация

1. `POST /auth/login` — тело `{"username":"...","password":"..."}`.
2. Ответ: `access_token` (JWT), в запросах к `/api/v1/*`: заголовок  
   `Authorization: Bearer <token>`.

Демо-учётные записи (bcrypt): `admin`/`admin`, `user`/`user`, `dispatcher`/`dispatcher`, `technician`/`technician`.

Роли в токене: `admin`, `dispatcher`, `supervisor`, `technician`, `viewer`, `user`.  
**Администратор** видит все сущности; остальные — только со своим `owner_user_id`.

Устарело: заголовки `x-role` / `x-actor-id` для API **не используются** (заменены на JWT).

## Документация API

- OpenAPI JSON: `GET /api-docs/openapi.json`
- Swagger UI: `GET /swagger-ui/`

## Основные маршруты (`/api/v1/*` — нужен JWT)

- `GET /health` — без JWT, проверка живости (`{ "status": "ok" }`).
- **Задачи (очередь):** `POST /api/v1/jobs`, `GET /api/v1/jobs/{id}` — при настроенных Redis+RabbitMQ; виды `kind`: `echo`, `simulate_slow`.
- Активы: `POST/GET /api/v1/assets`, `GET /api/v1/assets/{id}`
- Заявки: `POST/GET /api/v1/requests`, `GET .../overdue`, `GET .../{id}`, `PUT .../{id}/status`
- Наряды: `POST/GET /api/v1/work-orders`, `PUT .../assign|start|complete`, `GET /api/v1/requests/{id}/work-orders`
- Эскалации: `POST/GET /api/v1/escalations`, `POST /api/v1/sla/escalate-overdue`, `PUT .../resolve`, `GET /api/v1/requests/{id}/escalations`
- Техники: `POST/GET /api/v1/technicians`
- Аудит: `GET /api/v1/requests/{id}/audit`
- Дашборд: `GET /api/v1/dashboard/summary`, `.../sla-compliance`, `.../sla-compliance-by-priority`, `.../technicians/workload`

Где уместно — query `limit`, `offset` (и для заявок `status`, `priority`).

### Примеры payload

`POST /api/v1/assets`

```json
{
  "kind": "building",
  "title": "Склад N2",
  "location": "Санкт-Петербург"
}
```

`POST /api/v1/requests`

```json
{
  "asset_id": "asset-1",
  "description": "Срочно: отказ системы питания"
}
```

`PUT /api/v1/requests/{id}/status`

```json
{
  "status": "in_progress"
}
```

`POST /api/v1/jobs` (при включённой очереди)

```json
{
  "kind": "echo",
  "payload": { "msg": "ping" }
}
```
