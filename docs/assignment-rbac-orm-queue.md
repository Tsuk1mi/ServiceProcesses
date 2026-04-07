# Соответствие заданию: RBAC, владение данными, ORM, логи, очередь

Краткая карта по пунктам методички и где это в репозитории.

## 1. Авторизация RBAC (роли в БД, разделение, админ видит всё)

| Механизм | Реализация |
|----------|------------|
| Хранение ролей | Таблицы `app_user`, `app_user_role` (`backend/migrations/001_init.sql`); при логине роли читаются из БД |
| Выдача в токене | `backend/src/auth/jwt.rs` — в JWT попадают `roles` из `AuthUser` |
| Админ видит все данные | `DataScope::from_auth` в `backend/src/ports/data_scope.rs`: при роли `admin` — `DataScope::All`, иначе `Owner(sub)` |
| Проверка в сервисах | `backend/src/application/rbac.rs` — `require_any_role`; вызывается в `service_request_service`, `work_order_service`, `escalation_service`, `technician_service`, `sla_service` |
| Добавление роли | `UserStore::add_role_for_subject` (`backend/src/auth/users.rs`), реализация в `backend/src/infrastructure/postgres/user_store.rs`; API: **`POST /api/v1/admin/roles`** (только `admin`), тело `{ "subject_id": "<uuid>", "role": "viewer" }` — см. `backend/src/interfaces/http.rs` (`admin_add_role`) |

Проверки ролей на уровне HTTP (`require_roles` в `http.rs`) остаются для раннего отказа; повторная проверка в application-слое защищает и потенциальные вызовы не через HTTP.

## 2. Владение данными (ownership)

| Механизм | Реализация |
|----------|------------|
| Поле владельца | У доменных сущностей `owner_user_id` (и в SQL, и в entity) |
| Фильтрация чтения | Репозитории Postgres/in-memory: `get_by_id` / `list` с `DataScope` — см. `backend/src/infrastructure/postgres/repos.rs`, `in_memory.rs` |
| Защита записи | Методы `save` / `update` репозиториев принимают `actor_scope: DataScope`: для не-админа запрещено создавать/менять строки с чужим `owner_user_id` или перехватывать чужие id (`enforce_entity_save` в `repos.rs` и аналог в `in_memory.rs`) |
| Аудит | `owner_user_id` в записи аудита задаёт область видимости при чтении; `AuditRepository::save` без лишней проверки владельца-актора (владелец записи = владелец бизнес-сущности) |

## 3. Оптимизация ORM (eager, без N+1)

| Что сделано | Где |
|-------------|-----|
| **Eager**: пользователь + роли одним запросом | `PgUserStore::verify`: `find_with_related(app_user_role::Entity)` после объявления связей в `entity/app_user.rs` и `entity/app_user_role.rs` |
| **Eager**: заявки + актив | `ServiceRequestRepository::list_with_assets` — `find_also_related(asset::Entity)` в `repos.rs` (комментарий в `ports/outbound.rs`) |
| **Lazy** | Обычные `find().one()` / `all()` без жадной загрузки связей там, где не нужен join (явное «ленивое» чтение по месту использования) |

Дашборд и отчёты используют несколько отдельных запросов по типам сущностей (не цикл по строкам с запросом на каждую) — отдельного N+1 по одной заявке нет.

## 4. Логирование

| Событие | Где |
|---------|-----|
| Входящий HTTP-запрос | `redis_http_cache_middleware` в `http.rs`: `tracing::info!(method, path, "incoming http request")` |
| Ошибки домена → HTTP | `domain_error_to_response`: `tracing::warn!(error, ...)` |
| Ключевое действие | Пример: создание заявки — `tracing::info!` в `service_request_service.rs` |
| Общий трейсинг запросов | `TraceLayer::new_for_http()` на корневом роутере в `http.rs` |

## 5. Фоновые задачи и RabbitMQ + Redis

| Требование | Реализация |
|------------|------------|
| Очередь | RabbitMQ, очередь `JOB_QUEUE_NAME` (по умолчанию `service_jobs`), см. `backend/src/infrastructure/jobs.rs` |
| Статус задачи | Redis, ключи `job:status:{uuid}`, TTL; опрос **`GET /api/v1/jobs/{id}`** |
| Воркер | `APP_MODE=queue_worker`, `main.rs` → `run_worker` |
| Демо-задачи | `echo`, `simulate_slow` через **`POST /api/v1/jobs`** (внешним клиентам разрешены только эти `kind`, см. `enqueue_job`) |

### Перевод «всех» мутаций REST в очередь

Полная схема «каждый POST/PUT только ставит `api.*` в RabbitMQ, воркер пишет в Postgres» **в этом коммите не доведена до конца**: доменные маршруты по-прежнему выполняют бизнес-логику **синхронно** в HTTP после проверок RBAC/ownership. Инфраструктура очереди + Redis для статусов готова; чтобы закрыть формулировку задания полностью, нужно вынести `AppState` в отдельный модуль, добавить обработчик `api.*` в воркере (аналог текущих хендлеров) и заменить тела мутаций в `http.rs` на `enqueue` + **202 Accepted** + тот же `job_id`.

## 6. Docker

Стек Postgres + Redis + RabbitMQ + `backend` + `sla-worker` + `queue-worker`: `infra/docker/docker-compose.yml` (учётная запись RabbitMQ `app`, vhost `%2F` в URI).

---

*При нехватке места на диске сборка может падать на incremental; имеет смысл `cargo clean` или `CARGO_INCREMENTAL=0 cargo check`.*
