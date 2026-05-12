import { useMemo } from "react";
import { Link, useParams } from "react-router-dom";
import { serviceDeskApi } from "@/shared/api/serviceDeskApi";
import { EmptyState } from "@/shared/components/EmptyState";
import { MetricCard } from "@/shared/components/MetricCard";
import { PageHeader } from "@/shared/components/PageHeader";
import { StatusBadge } from "@/shared/components/StatusBadge";
import { useAsync } from "@/shared/hooks/useAsync";
import { formatDateTime } from "@/shared/utils/format";

export function ObjectDetailPage() {
  const { id = "" } = useParams();
  const { data } = useAsync(() => serviceDeskApi.objects.get(id), [id]);
  const { data: requests } = useAsync(() => serviceDeskApi.requests.list(), []);
  const { data: workOrders } = useAsync(() => serviceDeskApi.workOrders.list(), []);

  const relatedRequests = useMemo(
    () => (requests ?? []).filter((item) => item.objectId === id),
    [id, requests]
  );
  const relatedRequestIds = useMemo(() => new Set(relatedRequests.map((item) => item.id)), [relatedRequests]);
  const relatedWorkOrders = useMemo(
    () => (workOrders ?? []).filter((item) => relatedRequestIds.has(item.requestId)),
    [relatedRequestIds, workOrders]
  );
  const closedRequests = relatedRequests.filter((item) => item.status === "CLOSED").length;
  const activeWorkOrders = relatedWorkOrders.filter((item) => item.status !== "COMPLETED" && item.status !== "CANCELLED").length;

  if (!data) return <div className="loading">Загрузка объекта...</div>;

  return (
    <>
      <PageHeader
        title={data.name}
        description={`${data.type} / ${data.serialNumber}`}
        actions={<Link className="primary-button" to="/requests/new">Создать заявку</Link>}
      />
      <section className="metric-grid">
        <MetricCard label="Заявок по объекту" value={relatedRequests.length} />
        <MetricCard label="Закрыто" value={closedRequests} tone="success" />
        <MetricCard label="Активных нарядов" value={activeWorkOrders} />
        <MetricCard label="Статус объекта" value={data.status} />
      </section>
      <section className="detail-layout">
        <div className="panel">
          <dl className="definition-list">
            <dt>Статус</dt><dd><StatusBadge value={data.status} /></dd>
            <dt>Адрес</dt><dd>{data.address}</dd>
            <dt>Производитель</dt><dd>{data.manufacturer}</dd>
            <dt>Модель</dt><dd>{data.model}</dd>
            <dt>Дата установки</dt><dd>{data.installedAt}</dd>
          </dl>
        </div>
        <aside className="panel">
          <h2>QR-код</h2>
          <div className="qr-box">{data.id}</div>
          <h2>Связанные данные</h2>
          <p>Заявок: {relatedRequests.length}</p>
          <p>Нарядов: {relatedWorkOrders.length}</p>
          <p>Дата установки: {formatDateTime(data.installedAt)}</p>
        </aside>
      </section>
      <section className="dashboard-grid">
        <section className="panel">
          <h2>Последние заявки по объекту</h2>
          {relatedRequests.length ? (
            <ul className="activity-list">
              {relatedRequests.slice(0, 5).map((item) => (
                <li key={item.id}>
                  <span><Link to={`/requests/${item.id}`}>{item.id}</Link></span>
                  {item.title} / {item.status}
                </li>
              ))}
            </ul>
          ) : (
            <EmptyState title="По объекту еще нет заявок" description="Создайте первую заявку, чтобы начать обслуживание объекта." />
          )}
        </section>
        <section className="panel">
          <h2>Наряды по объекту</h2>
          {relatedWorkOrders.length ? (
            <ul className="activity-list">
              {relatedWorkOrders.slice(0, 5).map((item) => (
                <li key={item.id}>
                  <span><Link to={`/work-orders/${item.id}`}>{item.id}</Link></span>
                  {item.assignee} / {item.status}
                </li>
              ))}
            </ul>
          ) : (
            <EmptyState title="Нарядов пока нет" description="Они появятся после создания и обработки заявок по этому объекту." />
          )}
        </section>
      </section>
    </>
  );
}
