# Backend Core (Rust, Hexagonal)

## Запуск локально

```bash
cargo run
```

Сервер стартует на `http://localhost:8080`.

## Входные точки для UI

- `GET /health` - проверка доступности backend.
- `POST /api/v1/assets` - создать объект обслуживания.
- `GET /api/v1/assets` - получить список объектов.
- `GET /api/v1/assets/{id}` - получить объект по идентификатору.
- `POST /api/v1/requests` - создать заявку.
- `GET /api/v1/requests` - получить список заявок.
- `GET /api/v1/requests/{id}` - получить заявку по идентификатору.
- `PUT /api/v1/requests/{id}/status` - обновить статус заявки.
- `POST /api/v1/work-orders` - создать наряд по заявке.
- `GET /api/v1/requests/{id}/work-orders` - получить наряды заявки.

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
