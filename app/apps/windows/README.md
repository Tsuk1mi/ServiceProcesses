# frontend

Репозиторий фронтендов. В рамках этого каркаса добавлен проект **Windows Desktop Client** в папке `windows/`.

## Стек (Windows)
- **C# / .NET (WPF)**: основной целевой стек для промышленной эксплуатации на Windows.
- **MVVM**: разделение Views / ViewModels / Models/DTO.

## Быстрый старт
См. отдельную инструкцию: `../../docs/clients-run-guide.md` (запуск backend + Windows + Android).

## Архитектура
См. `docs/architecture.md`.

## Структура
Ключевая структура проекта:

- `docs/` — пользовательская и разработческая документация
- `src/` — исходный код клиента (тонкий клиент, без бизнес-логики)
- `packaging/` — упаковка MSIX/MSI, автообновление, интеграция с корпоративными репозиториями
- `.gitlab-ci.yml` — CI/CD пайплайн (restore/build/test/publish/package/deploy artifacts)
