# Запуск клиентских приложений (Windows и Android)

Документ **отдельный** от общей RUP/API-документации. Здесь только практика: что установить, в каком порядке запускать backend и клиенты, как проверить связь.

---

## 1. Общая схема

1. Запускается **сервер** (Rust backend на порту **8080**).
2. **Windows (WPF)** или **Android** обращаются к HTTP API, в каркасе — в первую очередь к `GET /health`.
3. Операции с данными (`/api/v1/...`) требуют **JWT**: сначала `POST /auth/login`, затем заголовок `Authorization: Bearer <token>`.

Сейчас `GET /health` отвечает **JSON**:

```json
{ "status": "ok" }
```

Полный стек в Docker (Postgres, Redis, RabbitMQ, воркеры): см. **`docs/server-stack.md`** и `infra/docker/docker-compose.yml`.

---

## 2. Требования к окружению

| Компонент | Минимум |
|-----------|---------|
| Backend | [Rust + Cargo](https://rustup.rs/), установленные зависимости как в `backend/README.md` |
| Windows-клиент | [.NET SDK 8+](https://dotnet.microsoft.com/) (на машине разработчика успешно собиралось на .NET 8 / WindowsDesktop) |
| Android-клиент | **JDK 17**, **Android SDK** (часто уже есть вместе с Visual Studio / Android Studio) |

Проверка:

```powershell
cargo --version
dotnet --version
java -version
```

Путь к SDK по умолчанию на Windows часто такой:

`%LOCALAPPDATA%\Android\Sdk`

Для сборки Android из командной строки задайте:

```powershell
$env:ANDROID_HOME = "$env:LOCALAPPDATA\Android\Sdk"
```

---

## 3. Запуск сервера (обязательно первым)

Из корня репозитория (или папки `backend`):

```powershell
cd C:\Users\Tsukimi\RustroverProjects\ServiceProcesses\backend
cargo run
```

Ожидаемый вывод в консоли: сервер слушает `0.0.0.0:8080`. Локально с клиента используйте базовый URL:

`http://localhost:8080/`

Проверка в браузере или PowerShell:

```powershell
Invoke-RestMethod -Uri "http://localhost:8080/health"
```

Должен вернуться объект с полем `status: ok`.

### 3.1 Вход (JWT) и вызов защищённого API

Демо-пользователи (см. `backend/README.md`): например `admin` / `admin`.

```powershell
$base = "http://localhost:8080"
$login = Invoke-RestMethod -Uri "$base/auth/login" -Method Post -ContentType "application/json" `
  -Body '{"username":"admin","password":"admin"}'
$token = $login.access_token
$headers = @{ Authorization = "Bearer $token" }
Invoke-RestMethod -Uri "$base/api/v1/assets" -Headers $headers
```

Опционально для фоновых задач нужны **Redis** и **RabbitMQ** (переменные `REDIS_URL`, `RABBITMQ_URL` при `cargo run`). Иначе `POST /api/v1/jobs` вернёт 503 — для проверки `GET /health` и списка активов этого достаточно.

---

## 4. Windows-приложение (WPF)

### 4.1. Где проект

`app\apps\windows\src\Frontend.Windows\Frontend.Windows.csproj`

Архитектура каркаса: **MVVM**, тонкий клиент, обращение к API через `Infrastructure\Api\ApiClient`.

### 4.2. Сборка

```powershell
cd C:\Users\Tsukimi\RustroverProjects\ServiceProcesses\app\apps\windows\src\Frontend.Windows
dotnet build -c Release
```

Тесты (опционально):

```powershell
dotnet test C:\Users\Tsukimi\RustroverProjects\ServiceProcesses\app\apps\windows\tests\Frontend.Windows.Tests\Frontend.Windows.Tests.csproj -c Release
```

### 4.3. Запуск

```powershell
dotnet run --project C:\Users\Tsukimi\RustroverProjects\ServiceProcesses\app\apps\windows\src\Frontend.Windows\Frontend.Windows.csproj
```

По умолчанию клиент берёт адрес API из переменной окружения **`SERVICE_PROCESSES_API_BASE_URL`**. Если её нет — используется **`http://localhost:8080/`**.

Пример с другим хостом:

```powershell
$env:SERVICE_PROCESSES_API_BASE_URL = "http://192.168.1.10:8080/"
dotnet run --project .\Frontend.Windows.csproj
```

В окне нажмите кнопку проверки API (команда «Проверить API» в интерфейсе). При работающем backend статус должен стать вроде **«API доступен»**.

### 4.4. NuGet

Для локальной разработки используется публичный **nuget.org** (см. `app\apps\windows\nuget.config`). В корпоративной сети добавьте свой feed **дополнительно**, не удаляя рабочий источник пакетов.

---

## 5. Android-приложение

### 5.1. Где проект

`app\apps\android\`

Минимальный **каркас**: одна активность, кнопка «Проверить API», запрос `GET /health` по HTTP.

### 5.2. Адрес API: эмулятор vs телефон

| Окружение | Базовый URL в `BuildConfig.API_BASE_URL` |
|-----------|------------------------------------------|
| Эмулятор Android | `http://10.0.2.2:8080` — это «localhost» хост-компьютера |
| Физическое устройство | Укажите **LAN-IP** вашего ПК, например `http://192.168.1.10:8080` (правка в `app\build.gradle.kts`, поле `buildConfigField`, затем пересборка) |

На устройстве и ПК должна быть одна сеть; брандмауэр Windows должен разрешать входящие на **8080** (при необходимости).

В манифесте включён **`usesCleartextTraffic`** для удобства отладки по HTTP. Для production перейдите на HTTPS.

### 5.3. Сборка APK (командная строка)

```powershell
$env:ANDROID_HOME = "$env:LOCALAPPDATA\Android\Sdk"
cd C:\Users\Tsukimi\RustroverProjects\ServiceProcesses\app\apps\android
.\gradlew.bat assembleDebug
```

Готовый APK:

`app\build\outputs\apk\debug\app-debug.apk`

### 5.4. Установка на устройство / эмулятор

При наличии `adb`:

```powershell
adb install -r .\app\build\outputs\apk\debug\app-debug.apk
```

Запустите приложение **ServiceProcesses**, нажмите **«Проверить API»**. При успехе увидите тело ответа с `\"status\":\"ok\"`.

### 5.5. Android Studio

Можно открыть папку `app\apps\android` как проект: **File → Open**, выбрать каталог. Далее **Sync Gradle** и **Run** на выбранном эмуляторе или устройстве.

---

## 6. Типичные проблемы

| Симптом | Что проверить |
|---------|----------------|
| Windows: «API недоступен» / ошибка сети | Запущен ли `cargo run`, не блокирует ли порт **8080** другой процесс, верен ли `SERVICE_PROCESSES_API_BASE_URL` |
| Android: таймаут | Backend запущен, IP/10.0.2.2 верный, Wi‑Fi тот же, cleartext и разрешение INTERNET |
| Windows: ошибки NuGet | Актуальный `nuget.config`, доступ в интернет или корп. Nexus |
| Android: SDK not found | Переменная `ANDROID_HOME`, установка platform-tools через SDK Manager |

---

## 7. Что дальше (по коду клиентов)

- Windows: подключить к `POST /auth/login` сохранение `access_token` и подстановку заголовка **`Authorization: Bearer …`** в `ApiClient` для всех запросов к `/api/v1/*` (вместо устаревших `x-role` / `x-actor-id`). См. `docs\api\backend-api.md` и Swagger `http://localhost:8080/swagger-ui/`.
- Android: то же для OkHttp interceptor; вынести базовый URL в `buildTypes` / product flavors (dev/stage/prod).
