# Infrastructure

Краткий указатель; подробно — **[`docs/infrastructure-overview.md`](../docs/infrastructure-overview.md)**.

## Docker Compose (основной локальный/демо стек)

```bash
cd infra/docker
docker compose up -d --build
```

Поднимаются **Postgres**, **Redis**, **RabbitMQ**, **backend** (API :8080), **sla-worker**, **queue-worker**. Переменные URL заданы в `docker-compose.yml`; для `cargo run` с хоста см. **`infra/docker/sample.env`**.

Профиль `extras`: MinIO, Nexus — `docker compose --profile extras up -d`.

## Kubernetes

Манифесты в **`infra/k8s/`**. Для демо без External Secrets:

1. `namespace.yaml`
2. `postgres.yaml`, `redis.yaml`, `rabbitmq.yaml`
3. `backend-secrets.example.yaml` → как Secret `backend-secrets` (или свой Secret с `DATABASE_URL`, `JWT_SECRET`)
4. `backend-configmap.yaml`
5. `backend-deployment.yaml`, `backend-service.yaml`
6. `sla-worker-deployment.yaml`, `queue-worker-deployment.yaml`

Не применяйте одновременно **`backend-externalsecret.yaml`** и **`backend-secrets.example.yaml`** с одним именем Secret.

## Helm

```bash
helm upgrade --install service-processes infra/helm/service-processes -n service-processes --create-namespace
```

В chart включены опционально **Redis** и **queue-worker** (`values.yaml`: `redis.enabled`, `queueWorker.enabled`). Секреты — через `secrets.existingSecretName`.
