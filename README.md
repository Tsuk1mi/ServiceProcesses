Ниже — **полная, оформленная архитектурная документация**, которую можно **поместить в GitHub /docs/architecture.md**.
Документ соответствует требованиям:

* единый UI
* backend на Rust
* hexagonal architecture
* GitOps / IaC / Terraform / Flux
* k8s / Docker / Helm
* Observability
* RabbitMQ
* S3 / AWS Secrets
* CI/CD на GitLab

Вставлены **image_group** для визуальной части и **entity-ссылки** на технологии.

---

# # Архитектурная документация

## Проект: Информационная система управления процессами обслуживания

![Image](https://miro.medium.com/1%2AaD3zDFzcF5Y2_27dvU213Q.png)

![Image](https://learn.microsoft.com/en-us/azure/architecture/reference-architectures/containers/aks-microservices/images/microservices-architecture.svg)

![Image](https://media.licdn.com/dms/image/v2/D4D12AQHaKCwjlMiT5g/article-cover_image-shrink_720_1280/article-cover_image-shrink_720_1280/0/1677225533131?e=2147483647\&t=XR9LzHa3qQqh4CFp92c359LOo2JjkpS4r6wouSZ2xDw\&v=beta)

![Image](https://miro.medium.com/1%2A7WTnMZaDDEMapnFJ04732g%402x.jpeg)

---

# 1. Цели проекта

Система предназначена для:

* управления процессами обслуживания физических объектов
* оркестрации задач
* контроля SLA
* приоритизации и эскалаций
* аналитики эксплуатационных процессов

Основная особенность — **единый UI для всех платформ**, три клиента:

* Android
* Windows
* Web

Используют общий код интерфейса на базе **React Native**.

Backend — микросервисная архитектура на **Rust**.

---

# 2. Общая архитектура системы

![Image](https://microservices.io/i/Microservice_Architecture.png)

![Image](https://softjourn.com/media/ArticlesMN/microfrontends/Pic1.1_Exploring_Micro_Fronend_Architecture.png)

![Image](https://platform9.com/media/kubernetes-constructs-concepts-architecture.jpg)

Система состоит из четырёх слоёв:

1. **Frontend (единый UI)**
   На базе

    * **React Native**
    * **React Native Web**
    * **React Native Windows**

2. **Backend (Hexagonal Architecture)**
   Реализован на

    * **Rust**
    * Framework: Axum / Actix
    * gRPC: Tonic

3. **Messaging & Storage**

    * Брокер: **RabbitMQ**
    * База данных: **PostgreSQL**
    * Хранилище файлов: **Amazon S3**
    * Секреты: **AWS Secrets Manager**

4. **Инфраструктура**

    * k8s: **Kubernetes**
    * IaC: **Terraform**
    * GitOps: **Flux CD**
    * Контейнеризация: **Docker**
    * Monitoring stack:

        * **Prometheus**
        * **Grafana**
        * **Loki**
        * **Alertmanager**

---

# 3. Архитектура UI (единый интерфейс)

![Image](https://miro.medium.com/v2/resize%3Afit%3A1400/1%2A1_Grxzn27wqPeE7mqVcnfA.png)

![Image](https://microsoft.github.io/react-native-windows/docs/assets/rn-windows-app-layout-with-native-modules.png)

![Image](https://res.cloudinary.com/formidablelabs/image/upload/f_auto%2Cq_auto/v1675121564/dotcom/uploads-old-diagram-full)

## 3.1 Технологический стек UI

| Платформа | Технология              |
| --------- | ----------------------- |
| Android   | React Native (Expo/CLI) |
| Windows   | React Native Windows    |
| Web       | React Native Web        |

## 3.2 Monorepo структура

```plaintext
/monorepo
    /apps
        /mobile      # Android/iOS
        /web         # Web SSR/SPA
        /windows     # Windows Desktop
    /packages
        /ui          # единый UI-кит
        /design-system
        /features    # экраны
        /api-sdk     # общий API-клиент
        /utils
```

## 3.3 Единый UI-кит

* кнопки, поля ввода, списки, таблицы
* карточки задач
* единая тема и дизайн

## 3.4 Платформенные адаптеры

* камера (Android)
* файловая система (Windows)
* адаптация под браузер (Web)

---

# 4. Backend архитектура (Hexagonal Architecture)

![Image](https://miro.medium.com/v2/resize%3Afit%3A1400/1%2AMw9B-6CmH_XeOhf_KiX9BQ.png)

![Image](https://images.ctfassets.net/o7xu9whrs0u9/LStg56PHqOEXZgOzOmriV/5a0a71d2e3460fa0c2dc45f5ae39206e/rust-services-OG.png)

![Image](https://miro.medium.com/0%2A3FZGIgynXuegHO4Y.)

## 4.1 Структура backend

```plaintext
/backend
    /core
        /entities
        /value_objects
        /usecases
        /services (SLA, Orchestration, Escalation)
        /events
    /application
        /ports
            rest/
            grpc/
            mq/
            storage/
    /infrastructure
        /postgres
        /rabbitmq
        /s3
        /redis
        /secrets (ESO)
        /observability
    main.rs
```

## 4.2 Основные сервисы

* **Orchestration Service**
* **Task Management Service**
* **SLA Engine**
* **Escalation Engine**
* **Analytics Service**
* **File Storage Service (S3)**
* **Auth & RBAC Service**

## 4.3 Транспорт

* REST API
* gRPC
* WebSockets
* AMQP (RabbitMQ)

---

# 5. Инфраструктура и DevOps

![Image](https://miro.medium.com/1%2A7x7SmXVPuUZyP9GbOISZbA.png)

![Image](https://civo-com-assets.ams3.digitaloceanspaces.com/content_images/1972.blog.png?1671100664=)

![Image](https://miro.medium.com/1%2Ahg5YB0q7KVxKH-6sDcPWQQ.png)

---

# 5.1 Terraform (IaC)

```plaintext
/infra/terraform
    /modules
        vpc
        eks
        rds
        s3
        ecr
    main.tf
```

Создаёт:

* кластер **Amazon EKS**
* базу данных
* S3
* сети
* роли

---

# 5.2 GitOps (FluxCD)

```plaintext
/infra/gitops
    /clusters
        dev/
        prod/
    /apps
        backend/
        frontend/
        monitoring/
```

Flux:

* подтягивает Helm-чарты
* обновляет систему при изменении в Git

---

# 5.3 Helm charts

```plaintext
/infra/helm
   backend/
   frontend/
   rabbitmq/
   monitoring/
   ingress/
```

---

# 5.4 Observability stack

Метрики → **Prometheus**
Логи → **Loki**
Дашборды → **Grafana**
Алерты → **Alertmanager**

---

# 6. Messaging: RabbitMQ

Используются события:

* `task.created`
* `task.assigned`
* `task.updated`
* `task.completed`
* `sla.breach`
* `escalation.started`
* `escalation.resolved`

Сервисы подписываются в стиле event-driven.

---

# 7. Хранилища данных

## 7.1 Реляционная БД

**PostgreSQL**

Таблицы:

* tasks
* objects
* technicians
* sla_policies
* escalations
* events
* audit_logs

## 7.2 Объектное хранилище

**Amazon S3**

Хранит:

* фото–отчёты
* чеклисты
* документы

## 7.3 Секреты

Обрабатываются через:

* **External Secrets Operator**
* **AWS Secrets Manager**

---

# 8. CI/CD (GitLab)

![Image](https://stytex.de/images/2016/04/deployment_gitlab_ci_dokku.png)

![Image](https://fluxcd.io/img/diagrams/gitops-toolkit.png)

---

## 8.1 Конвейер

Стадии:

1. build
2. test
3. security scan
4. dockerize
5. push to registry
6. deploy via GitOps

## 8.2 Пример pipeline

```yaml
stages: [build, test, docker, deploy]

build_backend:
  image: rust:latest
  script: cargo build --release

build_frontend:
  image: node:20
  script: npm ci && npm run build

dockerize:
  image: docker:stable
  script:
    - docker build -t registry/app:latest .

deploy:
  script:
    - git push gitops-repo
```

---

# 9. Полная структура GitHub репозитория

```plaintext
/docs
    architecture.md
    rup/
    idef0/
    
/monorepo
    /apps
        mobile/
        web/
        windows/
    /packages
        ui/
        design-system/
        api-sdk/
        features/
        utils/

/backend
/infra
    /terraform
    /helm
    /gitops

.gitlab-ci.yml
README.md
```


