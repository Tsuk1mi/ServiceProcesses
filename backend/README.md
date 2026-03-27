# Backend Core (Rust, Hexagonal)

## Запуск локально

```bash
cargo run
```

Сервер стартует на `http://localhost:8080`.

Режимы запуска:
- `APP_MODE=api` - HTTP API сервер (по умолчанию).
- `APP_MODE=worker` - SLA worker для фоновой автоэскалации.

## Входные точки для UI

Для операций изменения данных используйте заголовок роли:
- `x-role: dispatcher`
- `x-role: technician`
- `x-role: supervisor`
- `x-role: viewer`

Для `x-role: technician` на endpoint-ах:
- `PUT /api/v1/work-orders/{id}/start`
- `PUT /api/v1/work-orders/{id}/complete`
обязательно передавать `x-actor-id: tech-<id>`.

- `GET /health` - проверка доступности backend (JSON: `{ "status": "ok" }`).
- `POST /api/v1/assets` - создать объект обслуживания.
- `GET /api/v1/assets` - получить список объектов.
- `GET /api/v1/assets/{id}` - получить объект по идентификатору.
- `POST /api/v1/requests` - создать заявку.
- `GET /api/v1/requests` - получить список заявок.
  - поддерживает query: `limit`, `offset`, `status`, `priority`
- `GET /api/v1/requests/overdue` - получить просроченные по SLA заявки.
  - поддерживает query: `limit`, `offset`
- `GET /api/v1/requests/{id}` - получить заявку по идентификатору.
- `PUT /api/v1/requests/{id}/status` - обновить статус заявки.
- `POST /api/v1/work-orders` - создать наряд по заявке.
- `PUT /api/v1/work-orders/{id}/assign` - назначить исполнителя на наряд.
- `PUT /api/v1/work-orders/{id}/start` - перевести наряд в работу.
- `PUT /api/v1/work-orders/{id}/complete` - завершить наряд.
- `GET /api/v1/requests/{id}/work-orders` - получить наряды заявки.
  - поддерживает query: `limit`, `offset`
- `POST /api/v1/escalations` - создать эскалацию по заявке.
- `POST /api/v1/sla/escalate-overdue` - автоматически создать эскалации для просроченных заявок.
- `PUT /api/v1/escalations/{id}/resolve` - закрыть эскалацию.
- `GET /api/v1/requests/{id}/escalations` - получить эскалации заявки.
  - поддерживает query: `limit`, `offset`
- `POST /api/v1/technicians` - создать техника (исполнителя).
- `GET /api/v1/technicians` - получить список техников.
  - поддерживает query: `limit`, `offset`
- `GET /api/v1/requests/{id}/audit` - получить аудит действий по заявке.
  - поддерживает query: `limit`, `offset`
- `GET /api/v1/dashboard/summary` - получить агрегированную сводку для дашборда.
- `GET /api/v1/dashboard/sla-compliance` - получить SLA compliance по открытым заявкам.
- `GET /api/v1/dashboard/sla-compliance-by-priority` - получить SLA compliance по открытым заявкам с разбивкой по приоритетам.
- `GET /api/v1/dashboard/technicians/workload` - получить сводку загрузки техников.
  - поддерживает query: `limit`, `offset`

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
