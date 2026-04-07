# RabbitMQ, Redis и Docker: где что сделано

Краткая карта относительно требований «опора API на RabbitMQ + Redis» и «настройка Docker». Ниже — **как устроено сейчас** в коде и в `infra/docker`.

## 1. RabbitMQ

| Назначение | Где в коде |
|------------|------------|
| Очередь фоновых **jobs** (`echo`, `simulate_slow`): публикация из API, потребление в `queue_worker` | `backend/src/infrastructure/jobs.rs` — `JobClient::enqueue`, `basic_publish` в очередь `JOB_QUEUE_NAME` (по умолчанию `service_jobs`) |
| Потребитель очереди | `run_worker` в том же файле; запуск: `APP_MODE=queue_worker` в `backend/src/main.rs` (`run_queue_worker`) |
| **Доменные события** (topic exchange) | `DOMAIN_EVENTS_EXCHANGE`, `JobClient::publish_domain_event`; адаптер `JobClientEventPublisher` реализует `EventPublisherPort` и подключается в `build_state_pg` / in-memory ветке в `main.rs` |

Итог: RabbitMQ используется для **отдельного API задач** (`POST /api/v1/jobs`) и для **публикации событий** из application-сервисов, а не как транспорт для каждого REST-эндпоинта мутаций (создание заявок, нарядов и т.д.).

## 2. Redis

| Назначение | Где в коде |
|------------|------------|
| Статусы jobs (`job:status:{uuid}`), TTL | `backend/src/infrastructure/jobs.rs` — `write_record`, `get_status`, обновления в `run_worker` |
| HTTP-кэш ответов **GET** `/api/v1/*` (кроме `/api/v1/jobs`) | `backend/src/infrastructure/redis_cache.rs`; middleware в `backend/src/interfaces/http.rs` (`redis_http_cache_middleware`), инвалидация при не-GET |
| Подключение к Redis для API | Общий `ConnectionManager` из `JobClient` в режиме PostgreSQL; в in-memory — опционально при заданных `REDIS_URL` и `RABBITMQ_URL` |

Итог: Redis — **статусы асинхронных jobs** и **кэш чтений**; сами мутации (`POST`/`PUT` по домену) выполняются **синхронно** в обработчиках HTTP и пишут в Postgres.

## 3. Обязательность Redis + RabbitMQ

- При **`DATABASE_URL` задан** (`build_state_pg` в `main.rs`) подключение к Redis и RabbitMQ **обязательно**: без них API не поднимется (единый `JobClient::connect`).
- В режиме **только in-memory** (без `DATABASE_URL`) Redis/RabbitMQ **опциональны**: при отсутствии — нет кэша, нет AMQP-событий и недоступны `/api/v1/jobs` (503).

## 4. Docker Compose

Файл: `infra/docker/docker-compose.yml`.

| Сервис | Роль |
|--------|------|
| `postgres` | БД, healthcheck `pg_isready`, volume `pgdata` |
| `redis` | Кэш + статусы jobs, healthcheck `redis-cli ping` |
| `rabbitmq` | Брокер; пользователь `app`/`app` (не `guest` — иначе нет доступа с других контейнеров); URI с vhost `%2F` |
| `backend` | `APP_MODE=api`, все URL через имена сервисов; `depends_on` с `condition: service_healthy` |
| `sla-worker` | `APP_MODE=worker`, SLA + снимки аналитики |
| `queue-worker` | `APP_MODE=queue_worker`, consumer очереди `service_jobs` |

Сборка образа: `backend/Dockerfile` (в т.ч. копирование `migrations` и базовый образ Rust актуальной ветки).

## 5. Соответствие формулировке «все API на RabbitMQ + Redis»

**Сделано:** интеграция стека **RabbitMQ + Redis + Postgres** в Docker; фоновые **jobs** полностью на паре **очередь + Redis**; **кэш GET** в Redis; **события** в RabbitMQ; в PG-режиме брокер и Redis **обязательны** для работы API.

**Не сделано (если подразумевалось буквально):** перевод **всех** доменных операций (`/api/v1/assets`, `/api/v1/requests`, наряды, эскалации и т.д.) в модель «только постановка в RabbitMQ + опрос результата в Redis». Эти маршруты по-прежнему обрабатываются **синхронно** в `backend/src/interfaces/http.rs` с записью в PostgreSQL.

---

*Документ отражает состояние репозитория на момент составления; при появлении очередей для доменных команд его стоит обновить.*
