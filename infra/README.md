# Infrastructure Quick Start

## Локальный запуск (Docker)

```bash
cd infra/docker
docker compose up -d --build
```

Поднимаются:
- backend (`:8080`)
- RabbitMQ (`:5672`, management `:15672`)
- S3-совместимое хранилище MinIO (`:9000`, console `:9001`)
- Nexus (`:8081`)

## Kubernetes

Базовые манифесты находятся в `infra/k8s`:
- `namespace.yaml`
- `backend-configmap.yaml`
- `backend-externalsecret.yaml`
- `backend-deployment.yaml`
- `backend-service.yaml`
- `rabbitmq.yaml`
- `s3-minio.yaml`
- `nexus.yaml`

Пример применения:

```bash
kubectl apply -f infra/k8s/namespace.yaml
kubectl apply -f infra/k8s/
```
