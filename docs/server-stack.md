# Серверный стек и фоновые задачи

Полное описание инфраструктуры: **[infrastructure-overview.md](./infrastructure-overview.md)**.

## Сервисы в Docker

Файл: `infra/docker/docker-compose.yml`.

| Сервис        | Назначение |
|---------------|------------|
| `postgres`    | БД: `DATABASE_URL=postgres://app:app@postgres:5432/service_processes` (миграции при старте API). |
| `redis`       | Статусы задач `job:status:{uuid}`; кэш ответов GET `/api/v1/*` (`cache:api:v1:*`). |
| `rabbitmq`    | Очередь `service_jobs`; topic exchange **`service_processes.events`** — все доменные события из приложения. UI: http://localhost:15672 (guest/guest). |
| `backend`     | HTTP API :8080; при `DATABASE_URL` требует Redis+Rabbit. |
| `sla-worker`  | `APP_MODE=worker` — SLA, аудит, снимок аналитики (те же URL к БД и брокерам). |
| `queue-worker`| `APP_MODE=queue_worker` — потребитель очереди, обновляет Redis. |

Запуск:

```bash
cd infra/docker
docker compose up -d --build
```

Опционально MinIO и Nexus: `docker compose --profile extras up -d`.

## Переменные окружения (backend)

| Переменная       | Описание |
|------------------|----------|
| `APP_MODE`       | `api` (по умолчанию), `worker`, `queue_worker`. |
| `JWT_SECRET`     | Секрет подписи JWT (HS256). |
| `REDIS_URL`      | Например `redis://127.0.0.1:6379`. Без него и без RabbitMQ эндпоинты `/api/v1/jobs` отключены (503). |
| `RABBITMQ_URL`   | Например `amqp://guest:guest@127.0.0.1:5672/`. |
| `JOB_QUEUE_NAME` | Имя очереди (по умолчанию `service_jobs`). |
| `WORKER_INTERVAL_SEC` | Интервал SLA-воркера в секундах. |
| `DATABASE_URL`   | Postgres (SeaORM). Если задан — обязательны `REDIS_URL` и `RABBITMQ_URL`. |
| `RUST_LOG`       | Уровень логов, например `info,tower_http=debug` для трассировки HTTP. |

## Поток фоновой задачи

1. Клиент получает JWT: `POST /auth/login`.
2. `POST /api/v1/jobs` с телом, например `{"kind":"echo","payload":{"msg":"hi"}}` и заголовком `Authorization: Bearer …`.
3. API сразу отвечает **202** с `job_id` и статусом `queued`: запись в Redis и сообщение в RabbitMQ.
4. Процесс `queue_worker` забирает сообщение, выставляет в Redis `processing` → `completed` или `failed`.
5. Клиент опрашивает `GET /api/v1/jobs/{id}` (видит только свои задачи; `admin` — любые).

Типы `kind` (демо): `echo`, `simulate_slow` (имитация долгой работы ~3 с).

## Локальный запуск без Docker

Нужны Redis и RabbitMQ (или только API без очереди). Пример:

```bash
export REDIS_URL=redis://127.0.0.1:6379
export RABBITMQ_URL=amqp://guest:guest@127.0.0.1:5672/
cd backend && cargo run
```

Второй терминал для воркера очереди:

```bash
set APP_MODE=queue_worker
cargo run
```

## Примечание по сборке (Windows)

Клиент AMQP `lapin` собран с `native-tls`, чтобы не тянуть `rustls`/`ring` (часто проблемы с MSVC при сборке `ring`).
