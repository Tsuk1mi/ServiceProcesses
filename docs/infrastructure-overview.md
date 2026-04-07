# Инфраструктура: как всё устроено

Документ описывает компоненты окружения, потоки данных и способы запуска (Docker, Kubernetes, Helm). Детали API — в `docs/api/backend-api.md`, запуск клиентов — в `docs/clients-run-guide.md`.

---

## 1. Компоненты и роли

| Компонент | Назначение |
|-----------|------------|
| **Backend (HTTP)** | Axum API на порту 8080: JWT, CRUD по домену, отчёты, постановка фоновых задач. |
| **PostgreSQL** | Основное хранилище при заданном `DATABASE_URL` (SeaORM, миграции из `backend/migrations/001_init.sql` при старте). |
| **Redis** | Ключи `job:status:{uuid}` — статусы задач из `/api/v1/jobs`; ключи `cache:api:v1:{sha256}` — кэш ответов GET `/api/v1/*` (кроме `/api/v1/jobs`). |
| **RabbitMQ** | Очередь `service_jobs` (по умолчанию) — сообщения для `queue_worker`; exchange **topic** `service_processes.events` — все доменные события из `EventPublisherPort`. |
| **SLA worker** | `APP_MODE=worker`: просроченные заявки → эскалации, аудит, обновление снимка аналитики. |
| **Queue worker** | `APP_MODE=queue_worker`: потребитель очереди, обновление Redis по задачам (`echo`, `simulate_slow`). |

Опционально в Docker (профиль `extras`): **MinIO**, **Nexus** — не требуются для работы API.

---

## 2. Режимы backend (`APP_MODE`)

| Значение | Процесс |
|----------|---------|
| `api` | Только HTTP-сервер. |
| `worker` | Цикл SLA + снимок аналитики (без HTTP). |
| `queue_worker` | Один долгоживущий consumer RabbitMQ. |

---

## 3. Переменные окружения (сводка)

| Переменная | Обязательность | Описание |
|------------|----------------|----------|
| `DATABASE_URL` | Опционально | Непустое значение → Postgres + **обязательны** `REDIS_URL` и `RABBITMQ_URL`. Пусто → in-memory хранилище. |
| `REDIS_URL` | С `DATABASE_URL` | Подключение Redis (`redis://host:6379`). |
| `RABBITMQ_URL` | С `DATABASE_URL` | AMQP URI, например `amqp://guest:guest@rabbitmq:5672/`. |
| `JOB_QUEUE_NAME` | Нет | Имя очереди (по умолчанию `service_jobs`). |
| `JWT_SECRET` | Рекомендуется | Подпись JWT. |
| `WORKER_INTERVAL_SEC` | Нет | Период SLA-воркера в секундах (по умолчанию 30). |
| `RUST_LOG` | Нет | Уровень логов (`info`, `debug`, …). |

Без `DATABASE_URL`, но с Redis+RabbitMQ: включаются кэш GET, публикация событий в RabbitMQ и эндпоинты `/api/v1/jobs`.

---

## 4. Docker Compose

**Путь:** `infra/docker/docker-compose.yml`.

**Запуск:**

```bash
cd infra/docker
docker compose up -d --build
```

**Сервисы:**

- `postgres` — БД, healthcheck `pg_isready`.
- `redis` — healthcheck `redis-cli PING`.
- `rabbitmq` — AMQP + management UI `:15672` (guest/guest в образе по умолчанию для compose).
- `backend` — API; `DATABASE_URL`, `REDIS_URL`, `RABBITMQ_URL` указывают на имена сервисов в сети compose.
- `sla-worker` — тот же образ, `APP_MODE=worker`.
- `queue-worker` — тот же образ, `APP_MODE=queue_worker`.

`backend` / воркеры стартуют после **healthy** состояния Postgres, Redis и RabbitMQ.

**Локальный `cargo run` против контейнеров:** пример переменных — `infra/docker/sample.env` (хост `localhost` и порты `5432`, `6379`, `5672`). Файлы `.env*` в git игнорируются — скопируйте вручную в `.env` при необходимости.

---

## 5. Потоки данных

### 5.1 HTTP → БД

Запросы с JWT обрабатываются хендлерами; чтение/запись идёт в репозитории (Postgres или in-memory). Ответы GET под `/api/v1/` (кроме jobs) могут браться из **Redis** при попадании в кэш (`X-Cache: HIT`). Любой **не-GET** под `/api/v1/` сбрасывает ключи `cache:api:v1:*`.

### 5.2 Доменные события → RabbitMQ

Сервисы приложения вызывают `EventPublisherPort::publish`. При подключённом `JobClient` (Redis+Rabbit) сообщения уходят в exchange **`service_processes.events`** (topic), routing key = строка topic (например `service_request.created`).

### 5.3 Фоновые задачи (`POST /api/v1/jobs`)

1. API создаёт запись в Redis (`queued`) и публикует JSON в очередь RabbitMQ.
2. `queue_worker` читает сообщение, ставит `processing`, выполняет `kind` (`echo`, `simulate_slow`), пишет `completed` / `failed` в Redis.
3. Клиент опрашивает `GET /api/v1/jobs/{id}`.

### 5.4 SLA worker

По таймеру: выборка просроченных заявок → создание эскалаций → аудит → `analytics_snapshot_service.refresh` (кэш дашборда для admin scope в БД или памяти в зависимости от репозитория снимков).

---

## 6. Kubernetes (`infra/k8s`)

Базовые манифесты в каталоге `infra/k8s`:

- `namespace.yaml` — пространство имён `service-processes`.
- `backend-configmap.yaml` — неконфиденциальные переменные (`RUST_LOG`, `REDIS_URL`, `RABBITMQ_URL`, `JOB_QUEUE_NAME`, …).
- `backend-deployment.yaml` / `backend-service.yaml` — API.
- `sla-worker-deployment.yaml` — SLA worker.
- `queue-worker-deployment.yaml` — consumer очереди.
- `redis.yaml` — Redis для кэша и статусов задач.
- `postgres.yaml` — Postgres для демо-кластера (можно заменить на управляемую БД).
- `rabbitmq.yaml` — RabbitMQ с пользователем `app` / `app` (для подключения с других подов).
- `backend-externalsecret.yaml` — пример интеграции с External Secrets (AWS); для локального кластера используйте `backend-secrets.example.yaml`.
- `backend-secrets.example.yaml` — пример Secret с `DATABASE_URL`, `JWT_SECRET` (скопируйте и примените как `backend-secrets`).

Порядок применения для демо:

```bash
kubectl apply -f infra/k8s/namespace.yaml
kubectl apply -f infra/k8s/postgres.yaml
kubectl apply -f infra/k8s/redis.yaml
kubectl apply -f infra/k8s/rabbitmq.yaml
# создать секрет из примера и переименовать/применить как backend-secrets
kubectl apply -f infra/k8s/backend-secrets.example.yaml  # при необходимости поправить имя
kubectl apply -f infra/k8s/backend-configmap.yaml
kubectl apply -f infra/k8s/backend-deployment.yaml
kubectl apply -f infra/k8s/backend-service.yaml
kubectl apply -f infra/k8s/sla-worker-deployment.yaml
kubectl apply -f infra/k8s/queue-worker-deployment.yaml
```

`ExternalSecret` подключайте только если установлен External Secrets Operator и настроен `ClusterSecretStore`.

---

## 7. Helm (`infra/helm/service-processes`)

Chart задаёт ConfigMap, Deployments API и SLA worker, опционально queue worker и Redis (включается в `values.yaml`). Установка:

```bash
helm upgrade --install service-processes infra/helm/service-processes -n service-processes --create-namespace
```

Секреты (`DATABASE_URL`, `JWT_SECRET`, …) ожидаются в объекте Secret с именем из `values.yaml` → `secrets.existingSecretName`.

---

## 8. Схема взаимодействия (кратко)

```mermaid
flowchart LR
  subgraph clients [Клиенты]
    WPF[WPF]
    And[Android]
  end
  subgraph api [Backend API]
    HTTP[Axum]
  end
  subgraph data [Данные]
    PG[(Postgres)]
    RD[(Redis)]
  end
  subgraph mq [Очереди]
    RMQ[RabbitMQ]
  end
  WPF --> HTTP
  And --> HTTP
  HTTP --> PG
  HTTP --> RD
  HTTP --> RMQ
  QW[queue_worker] --> RMQ
  QW --> RD
  SW[sla_worker] --> PG
  SW --> RD
  SW --> RMQ
```

---

## 9. Полезные ссылки в репозитории

| Документ / путь | Содержание |
|-----------------|------------|
| `infra/README.md` | Быстрый старт Docker / K8s / Helm. |
| `docs/server-stack.md` | Стек и переменные (кратко). |
| `backend/README.md` | Backend, маршруты, переменные. |
| `backend/migrations/001_init.sql` | SQL-схема Postgres. |
