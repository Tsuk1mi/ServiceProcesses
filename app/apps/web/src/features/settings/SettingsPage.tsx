import { PageHeader } from "@/shared/components/PageHeader";
import { useAuth } from "@/app/providers/AuthProvider";
import { useTheme } from "@/app/providers/ThemeProvider";
import { useAppStore } from "@/app/store";
import { API_BASE_URL } from "@/shared/api/http";

export function SettingsPage({ section = "profile" }: { section?: string }) {
  const user = useAppStore((state) => state.user);
  const notifications = useAppStore((state) => state.notifications);
  const { token, logout } = useAuth();
  const { theme, toggleTheme } = useTheme();

  const sectionTitle =
    section === "security"
      ? "Безопасность"
      : section === "notifications"
        ? "Уведомления"
        : section === "appearance"
          ? "Оформление"
          : section === "system"
            ? "Система"
            : "Профиль";

  return (
    <>
      <PageHeader title="Настройки" description={`Раздел: ${sectionTitle}`} />
      {section === "profile" ? (
        <section className="form-grid panel">
          <label>Имя<input value={user?.name ?? "admin"} readOnly /></label>
          <label>Email<input value={user?.email ?? "admin@service.local"} readOnly /></label>
          <label>Роль<input value={user?.role ?? "ADMIN"} readOnly /></label>
          <label>Статус<input value={user?.status ?? "ACTIVE"} readOnly /></label>
        </section>
      ) : null}
      {section === "security" ? (
        <section className="form-grid panel">
          <label>Авторизация<input value={token ? "Активная сессия" : "Сессия отсутствует"} readOnly /></label>
          <label>Режим доступа<input value="Одна админская учетная запись" readOnly /></label>
          <label className="span-2">Токен<input value={token ? `${token.slice(0, 24)}...` : "Нет токена"} readOnly /></label>
          <button className="primary-button span-2" type="button" onClick={logout}>Выйти из системы</button>
        </section>
      ) : null}
      {section === "notifications" ? (
        <section className="form-grid panel">
          <label>Всего уведомлений<input value={notifications.length} readOnly /></label>
          <label>Непрочитанных<input value={notifications.filter((item) => !item.read).length} readOnly /></label>
          <label className="span-2">Последний канал<input value="Уведомления поступают из backend API и websocket-провайдера" readOnly /></label>
        </section>
      ) : null}
      {section === "appearance" ? (
        <section className="form-grid panel">
          <label>Текущая тема<input value={theme} readOnly /></label>
          <label>Источник настройки<input value="localStorage / service-desk-theme" readOnly /></label>
          <button className="primary-button span-2" type="button" onClick={toggleTheme}>Переключить тему</button>
        </section>
      ) : null}
      {section === "system" ? (
        <section className="form-grid panel">
          <label>API base URL<input value={API_BASE_URL} readOnly /></label>
          <label>Источник данных<input value="Только реальный backend API" readOnly /></label>
          <label>Mock-режим<input value="Отключен" readOnly /></label>
          <label>Bootstrap<input value="admin -> object -> request -> work order" readOnly /></label>
        </section>
      ) : null}
    </>
  );
}
