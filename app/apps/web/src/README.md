# web/src structure

Frontend построен как feature-first React приложение.

## app

- `router.tsx` - все маршруты React Router v6.
- `store.ts` - глобальное состояние Zustand: сессия, выбранная роль, уведомления.
- `App.tsx` - композиция провайдеров.
- `providers/` - Auth, Theme, WebSocket.
- `layout/` - общий shell: sidebar, topbar, рабочая область.

## features

Каждый модуль содержит свои страницы и локальные компоненты:

- `auth` - login, forgot, reset, MFA, AuthGuard, RoleGuard.
- `dashboard` - role-based dashboard для manager/dispatcher, technician, client.
- `requests` - список, карточка, создание/редактирование, kanban.
- `work-orders` - список и карточка наряда.
- `objects` - реестр и паспорт объекта.
- `sla` - dashboard и политики SLA.
- `escalations` - активные эскалации и конструктор правил.
- `analytics` - KPI и отчеты.
- `users` - пользователи, роли и права.
- `notifications` - уведомления.
- `settings` - профиль, безопасность, уведомления, внешний вид, системные настройки.

## shared

- `api/http.ts` - базовый fetch-клиент для `/api/v1`.
- `api/serviceDeskApi.ts` - типизированные методы модулей.
- `api/mockData.ts` - mock-данные до подключения backend.
- `components/` - общий UI-kit.
- `hooks/` - общие React hooks.
- `types/` - доменные типы.
- `utils/` - форматирование и вспомогательные функции.

## API mode

По умолчанию используется mock-режим. Для подключения реального backend:

```env
VITE_USE_MOCKS=false
VITE_API_BASE_URL=/api/v1
VITE_WS_URL=ws://localhost:8080/ws
```

После этого методы из `shared/api/serviceDeskApi.ts` будут ходить в реальные endpoints.
