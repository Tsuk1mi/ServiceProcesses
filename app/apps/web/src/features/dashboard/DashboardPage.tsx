import { Link } from "react-router-dom";
import { serviceDeskApi } from "@/shared/api/serviceDeskApi";
import { EmptyState } from "@/shared/components/EmptyState";
import { MetricCard } from "@/shared/components/MetricCard";
import { PageHeader } from "@/shared/components/PageHeader";
import { useAsync } from "@/shared/hooks/useAsync";
import { useAppStore } from "@/app/store";

export function DashboardPage() {
  const role = useAppStore((state) => state.user?.role);
  const { data } = useAsync(() => serviceDeskApi.dashboard.getSummary(), []);

  if (!data) {
    return <div className="loading">Загрузка dashboard...</div>;
  }

  const isEmptySystem =
    data.newRequests === 0 &&
    data.inProgress === 0 &&
    data.slaBreached === 0 &&
    data.completedToday === 0 &&
    data.activity.length === 0;

  if (role === "TECHNICIAN") {
    return (
      <>
        <PageHeader title="Мой день" description="Наряды, маршрут и срочные задачи на сегодня." />
        <section className="split-grid">
          <div className="panel">
            <h2>Мои наряды</h2>
            <ul className="activity-list">
              <li>WO-501: Чиллер York YVAA, старт 09:00</li>
              <li>WO-502: Замена фильтров, старт 12:00</li>
            </ul>
          </div>
          <div className="map-placeholder">Карта маршрута</div>
        </section>
      </>
    );
  }

  if (role === "CLIENT") {
    return (
      <>
        <PageHeader title="Мои заявки" actions={<Link className="primary-button" to="/requests/new">Создать заявку</Link>} />
        <section className="panel">
          <h2>Активные заявки</h2>
          <p>REQ-1234: Нет охлаждения в серверной</p>
          <p>REQ-1236: ИБП показывает ошибку батареи</p>
        </section>
      </>
    );
  }

  return (
    <>
      <PageHeader title="Операционный dashboard" description="Сводка по заявкам, SLA и текущей активности." />
      {isEmptySystem ? (
        <section className="panel">
          <EmptyState
            title="Система пока пустая"
            description="Начните с создания первого объекта, затем оформите по нему первую заявку."
            action={<Link className="primary-button" to="/objects/new">Создать объект</Link>}
          />
        </section>
      ) : null}
      <section className="metric-grid">
        <MetricCard label="Новых заявок" value={data.newRequests} />
        <MetricCard label="В работе" value={data.inProgress} />
        <MetricCard label="Просрочено SLA" value={data.slaBreached} tone="danger" />
        <MetricCard label="Выполнено сегодня" value={data.completedToday} tone="success" />
      </section>
      <section className="dashboard-grid">
        <div className="panel chart-panel">
          <h2>Заявки за 7 дней</h2>
          <div className="bar-chart">
            {[32, 48, 36, 52, 43, 61, 45].map((value, index) => <span key={index} style={{ height: `${value}%` }} />)}
          </div>
        </div>
        <div className="panel gauge-panel">
          <h2>SLA Compliance</h2>
          <strong>{data.slaCompliance}%</strong>
          <div className="progress"><span style={{ width: `${data.slaCompliance}%` }} /></div>
        </div>
        <div className="map-placeholder">Карта объектов со статусами</div>
        <div className="panel">
          <h2>Топ исполнителей</h2>
          {data.workloadByTechnician.map((item) => <p key={item.name}>{item.name}: {item.closed} закрыто</p>)}
        </div>
        <div className="panel wide-panel">
          <h2>Лента активности</h2>
          <ul className="activity-list">
            {data.activity.map((item) => <li key={item.id}><span>{item.at}</span>{item.text}</li>)}
          </ul>
        </div>
      </section>
    </>
  );
}
