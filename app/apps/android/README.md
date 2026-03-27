# Android-клиент (каркас)

Минимальное приложение для проверки связи с backend: `GET /health`.

Подробные шаги (эмулятор, IP, сборка, установка) — в репозитории:

`docs/clients-run-guide.md`

Сборка debug APK из корня этого каталога:

```powershell
$env:ANDROID_HOME = "$env:LOCALAPPDATA\Android\Sdk"
.\gradlew.bat assembleDebug
```

APK: `app\build\outputs\apk\debug\app-debug.apk`
