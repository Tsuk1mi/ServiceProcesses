import { serviceDeskApi } from "@/shared/api/serviceDeskApi";
import { EmptyState } from "@/shared/components/EmptyState";
import { MetricCard } from "@/shared/components/MetricCard";
import { PageHeader } from "@/shared/components/PageHeader";
import { useAsync } from "@/shared/hooks/useAsync";
import { useMemo } from "react";
import { Link } from "react-router-dom";

export function AnalyticsPage() {
  const { data } = useAsync(() => serviceDeskApi.analytics.overview(), []);
  const { data: requests } = useAsync(() => serviceDeskApi.requests.list(), []);
  const { data: workOrders } = useAsync(() => serviceDeskApi.workOrders.list(), []);
  const { data: objects } = useAsync(() => serviceDeskApi.objects.list(), []);
  const { data: technicians } = useAsync(() => serviceDeskApi.technicians.list(), []);

  const requestStats = useMemo(() => {
    const items = requests ?? [];
    return {
      total: items.length,
      planned: items.filter((item) => item.status === "PLANNED").length,
      inProgress: items.filter((item) => item.status === "IN_PROGRESS").length,
      resolved: items.filter((item) => item.status === "RESOLVED").length,
      escalated: items.filter((item) => item.status === "ESCALATED").length
    };
  }, [requests]);

  const workOrderStats = useMemo(() => {
    const items = workOrders ?? [];
    return {
      total: items.length,
      assigned: items.filter((item) => item.status === "ASSIGNED").length,
      inProgress: items.filter((item) => item.status === "IN_PROGRESS").length,
      completed: items.filter((item) => item.status === "COMPLETED").length
    };
  }, [workOrders]);

  const isEmpty =
    requestStats.total === 0 &&
    (objects?.length ?? 0) === 0 &&
    (workOrders?.length ?? 0) === 0 &&
    (technicians?.length ?? 0) === 0 &&
    !(data?.activity.length);

  return (
    <>
      <PageHeader title="Аналитика KPI" description="Заявки, персонал, объекты и отчеты." />
      {isEmpty ? (
        <section className="panel">
          <EmptyState
            title="Для аналитики пока нет данных"
            description="Создайте объекты и заявки, чтобы метрики и отчеты начали наполняться."
            action={<Link className="primary-button" to="/objects/new">Начать с объекта</Link>}
          />
        </section>
      ) : null}
      <section className="metric-grid">
        <MetricCard label="Всего заявок" value={requestStats.total} />
        <MetricCard label="Объектов" value={objects?.length ?? 0} />
        <MetricCard label="Исполнителей" value={technicians?.length ?? 0} />
        <MetricCard label="SLA в норме" value={`${data?.slaCompliance ?? 0}%`} tone="success" />
      </section>
      <section className="dashboard-grid">
        <div className="panel">
          <h2>Статусы заявок</h2>
          <p>Запланировано: {requestStats.planned}</p>
          <p>В работе: {requestStats.inProgress}</p>
          <p>Решено: {requestStats.resolved}</p>
          <p>Эскалировано: {requestStats.escalated}</p>
        </div>
        <div className="panel">
          <h2>Статусы нарядов</h2>
          <p>Всего: {workOrderStats.total}</p>
          <p>Назначено: {workOrderStats.assigned}</p>
          <p>В работе: {workOrderStats.inProgress}</p>
          <p>Завершено: {workOrderStats.completed}</p>
        </div>
        <div className="panel">
          <h2>Топ исполнителей</h2>
          {(data?.workloadByTechnician ?? []).length ? (
            (data?.workloadByTechnician ?? []).map((item) => <p key={item.name}>{item.name}: {item.closed} закрыто</p>)
          ) : (
            <p>Пока нет данных по загрузке.</p>
          )}
        </div>
        <div className="panel">
          <h2>Последняя активность</h2>
          {(data?.activity ?? []).length ? (
            <ul className="activity-list">
              {(data?.activity ?? []).map((item) => <li key={item.id}><span>{item.at}</span>{item.text}</li>)}
            </ul>
          ) : (
            <p>Активности пока нет.</p>
          )}
        </div>
      </section>
    </>
  );
}
