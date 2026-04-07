# Серверный стек и фоновые задачи

## Сервисы в Docker

Файл: `infra/docker/docker-compose.yml`.

| Сервис        | Назначение |
|---------------|------------|
| `postgres`    | БД (подключение через `DATABASE_URL`; миграции SeaORM можно добавить отдельно). |
| `redis`       | Статусы фоновых задач, ключ `job:status:{uuid}`. |
| `rabbitmq`    | Очередь сообщений; UI управления: http://localhost:15672 (guest/guest). |
| `backend`     | HTTP API на порту 8080. |
| `sla-worker`  | `APP_MODE=worker` — периодическое SLA и снимок аналитики. |
| `queue-worker`| `APP_MODE=queue_worker` — потребитель RabbitMQ, обновляет Redis. |

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
| `DATABASE_URL`   | Postgres (для будущего слоя SeaORM). |
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
