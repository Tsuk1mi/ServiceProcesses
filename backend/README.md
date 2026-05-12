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
| `read_model_worker` | Потребитель domain events для обновления CQRS read-model |

### Переменные окружения (важные)

| Переменная | Описание |
|------------|----------|
| `DATABASE_URL` | PostgreSQL (SeaORM). Если задан — **обязательны** `REDIS_URL` и `RABBITMQ_URL`: кэш GET `/api/v1/*` в Redis, доменные события в RabbitMQ (exchange `service_processes.events`), задачи `/api/v1/jobs`, read-model worker. Миграции: `migrations/001_init.sql`, `migrations/002_read_model.sql`. Без `DATABASE_URL` — in-memory; Redis+Rabbit при этом опциональны (включают кэш, AMQP и задачи). |
| `JWT_SECRET` | Секрет подписи JWT (в проде задать явно). |
| `JWT_TTL_HOURS` | TTL access token в часах (по умолчанию `24`). |
| `REDIS_URL` + `RABBITMQ_URL` | См. выше. В Docker задаются в `infra/docker/docker-compose.yml`; для локального `cargo run` см. `infra/docker/sample.env`. |
| `JOB_QUEUE_NAME` | Очередь RabbitMQ (по умолчанию `service_jobs`). |
| `READ_MODEL_QUEUE_NAME` | Очередь RabbitMQ для проекций read-model (по умолчанию `service_read_model`). |
| `WORKER_INTERVAL_SEC` | Интервал SLA-воркера (секунды). |
| `CORS_ALLOWED_ORIGINS` | Список разрешенных origin через запятую. |
| `RUST_LOG` | Например `info` или `info,tower_http=debug`. |

Docker-стек: см. `infra/docker/docker-compose.yml` и `docs/server-stack.md`.

## Аутентификация

1. `POST /auth/login` — тело `{"username":"...","password":"..."}`.
2. `POST /auth/refresh` — выпустить новый access token по валидному текущему JWT.
3. Ответ: `access_token` (JWT), в запросах к `/api/v1/*`: заголовок  
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

## CQRS + BFF

Новые маршруты:

- Command API: `/api/v1/commands/*`
- Query API: `/api/v1/query/*`
- BFF API: `/api/v1/bff/web/*`, `/api/v1/bff/mobile/*`

Read-model:

- `read_request_list_item`
- `read_request_detail`
- `read_work_order_item`
- `analytics_snapshot`
- `read_projection_error`

Domain events публикуются в `service_processes.events`, а `APP_MODE=read_model_worker` обновляет денормализованные read-таблицы. Это делает чтение eventual consistent по отношению к commands.

Heatmap и базовые runtime-метрики доступны через `GET /api/v1/query/heatmap`.

Подробная инструкция по архитектуре и проверке: [`../docs/cqrs-bff-implementation.md`](../docs/cqrs-bff-implementation.md).

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
