import { Link } from "react-router-dom";
import { useAuth } from "@/app/providers/AuthProvider";
import { serviceDeskApi } from "@/shared/api/serviceDeskApi";
import { EmptyState } from "@/shared/components/EmptyState";
import { MetricCard } from "@/shared/components/MetricCard";
import { PageHeader } from "@/shared/components/PageHeader";
import { useAsync } from "@/shared/hooks/useAsync";

export function ClientDashboardPage() {
  const { user } = useAuth();
  const { data: requests } = useAsync(() => serviceDeskApi.requests.list(), []);
  const { data: notifications } = useAsync(() => serviceDeskApi.notifications.list(), []);

  return (
    <>
      <PageHeader
        title={`Клиентский кабинет`}
        description={`Самообслуживание для ${user?.name ?? "пользователя"}: заявки, уведомления и профиль.`}
        actions={<Link className="primary-button" to="/requests/new">Создать заявку</Link>}
      />
      <section className="metric-grid">
        <MetricCard label="Мои заявки" value={requests?.length ?? 0} />
        <MetricCard label="Новые уведомления" value={(notifications ?? []).filter((item) => !item.read).length} />
        <MetricCard label="Роль" value={user?.role ?? "CLIENT"} />
      </section>
      <section className="dashboard-grid">
        <section className="panel">
          <h2>Быстрые действия</h2>
          <p><Link to="/requests/new">Открыть новую заявку</Link></p>
          <p><Link to="/requests">Посмотреть все заявки</Link></p>
          <p><Link to="/notifications">Открыть уведомления</Link></p>
          <p><Link to="/settings/profile">Профиль и настройки</Link></p>
        </section>
        <section className="panel">
          <h2>Последние обращения</h2>
          {(requests ?? []).length ? (
            <ul className="activity-list">
              {(requests ?? []).slice(0, 5).map((item) => (
                <li key={item.id}>
                  <span>{item.id}</span>
                  {item.title} / {item.status}
                </li>
              ))}
            </ul>
          ) : (
            <EmptyState
              title="Заявок пока нет"
              description="Создайте первое обращение, и оно появится здесь."
              action={<Link className="primary-button" to="/requests/new">Создать заявку</Link>}
            />
          )}
        </section>
      </section>
    </>
  );
}
