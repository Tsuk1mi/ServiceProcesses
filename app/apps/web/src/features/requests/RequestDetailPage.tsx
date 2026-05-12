import { useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { serviceDeskApi } from "@/shared/api/serviceDeskApi";
import { ApiError } from "@/shared/api/http";
import { PageHeader } from "@/shared/components/PageHeader";
import { StatusBadge } from "@/shared/components/StatusBadge";
import { useAsync } from "@/shared/hooks/useAsync";

export function RequestDetailPage() {
  const navigate = useNavigate();
  const { id = "" } = useParams();
  const [reloadKey, setReloadKey] = useState(0);
  const [busyAction, setBusyAction] = useState<string | null>(null);
  const [error, setError] = useState("");
  const { data, loading } = useAsync(() => serviceDeskApi.requests.get(id), [id, reloadKey]);
  const { data: history } = useAsync(() => serviceDeskApi.audit.requestHistory(id), [id, reloadKey]);
  const { data: escalations } = useAsync(() => serviceDeskApi.escalations.listByRequest(id), [id, reloadKey]);

  async function handleStatusUpdate(status: "planned" | "in_progress" | "resolved" | "closed" | "escalated") {
    try {
      setBusyAction(status);
      setError("");
      await serviceDeskApi.requests.updateStatus(id, status);
      setReloadKey((value) => value + 1);
    } catch (reason) {
      setError(reason instanceof ApiError ? `Ошибка API: ${reason.status}` : "Не удалось изменить статус заявки.");
    } finally {
      setBusyAction(null);
    }
  }

  async function handleCreateWorkOrder() {
    try {
      setBusyAction("create-work-order");
      setError("");
      const order = await serviceDeskApi.requests.createWorkOrder(id);
      navigate(`/work-orders/${order.id}`);
    } catch (reason) {
      setError(reason instanceof ApiError ? `Ошибка API: ${reason.status}` : "Не удалось создать наряд.");
    } finally {
      setBusyAction(null);
    }
  }

  async function handleManualEscalation() {
    try {
      setBusyAction("manual-escalation");
      setError("");
      await serviceDeskApi.escalations.create(id, "Manual escalation from request detail");
      setReloadKey((value) => value + 1);
    } catch (reason) {
      setError(reason instanceof ApiError ? `Не удалось создать эскалацию. Код API: ${reason.status}.` : "Не удалось создать эскалацию.");
    } finally {
      setBusyAction(null);
    }
  }

  if (loading || !data) {
    return <div className="loading">Загрузка заявки...</div>;
  }

  const primaryAction =
    data.status === "NEW"
      ? { label: "Принять в план", status: "planned" as const }
      : data.status === "PLANNED"
        ? { label: "Начать выполнение", status: "in_progress" as const }
        : data.status === "IN_PROGRESS"
          ? { label: "Отметить решенной", status: "resolved" as const }
          : data.status === "RESOLVED"
            ? { label: "Закрыть", status: "closed" as const }
            : null;

  return (
    <>
      <PageHeader
        title={`${data.id} ${data.title}`}
        actions={
          primaryAction ? (
            <button
              className="primary-button"
              onClick={() => handleStatusUpdate(primaryAction.status)}
              disabled={busyAction !== null}
            >
              {busyAction === primaryAction.status ? "Сохраняю..." : primaryAction.label}
            </button>
          ) : null
        }
      />
      <section className="detail-layout">
        <div className="panel">
          <div className="badge-row"><StatusBadge value={data.status} /><StatusBadge value={data.priority} /></div>
          {error ? <p className="form-error">{error}</p> : null}
          <dl className="definition-list">
            <dt>Тип</dt><dd>{data.type}</dd>
            <dt>Категория</dt><dd>{data.category}</dd>
            <dt>Описание</dt><dd>{data.description}</dd>
            <dt>Объект</dt><dd>{data.objectName}</dd>
            <dt>Заявитель</dt><dd>{data.requester}</dd>
            <dt>Исполнитель</dt><dd>{data.assignee ?? "Не назначен"}</dd>
          </dl>
        </div>
        <aside className="panel">
          <h2>SLA</h2>
          <StatusBadge value={data.slaStatus} />
          <p>Время реакции</p>
          <div className="progress"><span style={{ width: "68%" }} /></div>
          <p>Время решения</p>
          <div className="progress warning"><span style={{ width: "82%" }} /></div>
          <div className="action-stack">
            <button onClick={handleCreateWorkOrder} disabled={busyAction !== null}>
              {busyAction === "create-work-order" ? "Создаю наряд..." : "Создать наряд"}
            </button>
            {data.status === "NEW" ? (
              <button onClick={() => handleStatusUpdate("planned")} disabled={busyAction !== null}>
                Перевести в план
              </button>
            ) : null}
            {data.status === "PLANNED" ? (
              <button onClick={() => handleStatusUpdate("in_progress")} disabled={busyAction !== null}>
                Начать выполнение
              </button>
            ) : null}
            {data.status === "IN_PROGRESS" ? (
              <button onClick={() => handleStatusUpdate("resolved")} disabled={busyAction !== null}>
                Отметить решенной
              </button>
            ) : null}
            {data.status === "RESOLVED" ? (
              <button onClick={() => handleStatusUpdate("closed")} disabled={busyAction !== null}>
                Закрыть
              </button>
            ) : null}
            <button onClick={handleManualEscalation} disabled={busyAction !== null}>
              {busyAction === "manual-escalation" ? "Эскалирую..." : "Создать эскалацию"}
            </button>
            <button onClick={() => handleStatusUpdate("escalated")} disabled={busyAction !== null || data.status === "ESCALATED"}>
              Отметить как escalated
            </button>
          </div>
        </aside>
      </section>
      <section className="dashboard-grid">
        <section className="panel">
          <h2>История заявки</h2>
          {(history ?? []).length ? (
            <ul className="activity-list">
              {(history ?? []).map((item) => (
                <li key={item.id}>
                  <span>{item.createdAtUtc}</span>
                  {item.entity}.{item.action} / {item.actorRole} / {item.details}
                </li>
              ))}
            </ul>
          ) : (
            <p className="loading">История пока пуста.</p>
          )}
        </section>
        <section className="panel">
          <h2>Эскалации по заявке</h2>
          {(escalations ?? []).length ? (
            <ul className="activity-list">
              {(escalations ?? []).map((item) => (
                <li key={item.id}>
                  <span>{item.id}</span>
                  {item.reason} / {item.target}
                </li>
              ))}
            </ul>
          ) : (
            <p className="loading">Эскалаций по заявке пока нет.</p>
          )}
        </section>
      </section>
    </>
  );
}
