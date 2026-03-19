# Backend Core (Rust, Hexagonal)

## Запуск локально

```bash
cargo run
```

Сервер стартует на `http://localhost:8080`.

## Входные точки для UI

Для операций изменения данных используйте заголовок роли:
- `x-role: dispatcher`
- `x-role: technician`
- `x-role: supervisor`
- `x-role: viewer`

- `GET /health` - проверка доступности backend.
- `POST /api/v1/assets` - создать объект обслуживания.
- `GET /api/v1/assets` - получить список объектов.
- `GET /api/v1/assets/{id}` - получить объект по идентификатору.
- `POST /api/v1/requests` - создать заявку.
- `GET /api/v1/requests` - получить список заявок.
- `GET /api/v1/requests/{id}` - получить заявку по идентификатору.
- `PUT /api/v1/requests/{id}/status` - обновить статус заявки.
- `POST /api/v1/work-orders` - создать наряд по заявке.
- `PUT /api/v1/work-orders/{id}/assign` - назначить исполнителя на наряд.
- `PUT /api/v1/work-orders/{id}/start` - перевести наряд в работу.
- `PUT /api/v1/work-orders/{id}/complete` - завершить наряд.
- `GET /api/v1/requests/{id}/work-orders` - получить наряды заявки.
- `POST /api/v1/escalations` - создать эскалацию по заявке.
- `PUT /api/v1/escalations/{id}/resolve` - закрыть эскалацию.
- `GET /api/v1/requests/{id}/escalations` - получить эскалации заявки.
- `POST /api/v1/technicians` - создать техника (исполнителя).
- `GET /api/v1/technicians` - получить список техников.

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
