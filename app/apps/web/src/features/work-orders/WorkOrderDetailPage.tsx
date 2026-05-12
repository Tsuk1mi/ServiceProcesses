import { useState } from "react";
import { Link, useParams } from "react-router-dom";
import { serviceDeskApi } from "@/shared/api/serviceDeskApi";
import { ApiError } from "@/shared/api/http";
import { PageHeader } from "@/shared/components/PageHeader";
import { StatusBadge } from "@/shared/components/StatusBadge";
import { useAsync } from "@/shared/hooks/useAsync";

export function WorkOrderDetailPage() {
  const { id = "" } = useParams();
  const [reloadKey, setReloadKey] = useState(0);
  const [busyAction, setBusyAction] = useState<string | null>(null);
  const [error, setError] = useState("");
  const [assignee, setAssignee] = useState("");
  const { data, loading } = useAsync(() => serviceDeskApi.workOrders.get(id), [id, reloadKey]);
  const { data: technicians } = useAsync(() => serviceDeskApi.technicians.list(), [reloadKey]);

  async function handleAssign() {
    if (!assignee) {
      setError("Сначала выберите исполнителя.");
      return;
    }

    try {
      setBusyAction("assign");
      setError("");
      await serviceDeskApi.workOrders.assign(id, assignee);
      setReloadKey((value) => value + 1);
    } catch (reason) {
      setError(reason instanceof ApiError ? `Не удалось назначить исполнителя. Код API: ${reason.status}.` : "Не удалось назначить исполнителя.");
    } finally {
      setBusyAction(null);
    }
  }

  async function handleStart() {
    try {
      setBusyAction("start");
      setError("");
      await serviceDeskApi.workOrders.start(id);
      setReloadKey((value) => value + 1);
    } catch (reason) {
      setError(
        reason instanceof ApiError
          ? `Не удалось запустить наряд. Код API: ${reason.status}. Возможно, сначала нужно назначить исполнителя.`
          : "Не удалось запустить наряд."
      );
    } finally {
      setBusyAction(null);
    }
  }

  async function handleComplete() {
    try {
      setBusyAction("complete");
      setError("");
      await serviceDeskApi.workOrders.complete(id);
      setReloadKey((value) => value + 1);
    } catch (reason) {
      setError(reason instanceof ApiError ? `Не удалось завершить наряд. Код API: ${reason.status}.` : "Не удалось завершить наряд.");
    } finally {
      setBusyAction(null);
    }
  }

  if (loading || !data) return <div className="loading">Загрузка наряда...</div>;

  return (
    <>
      <PageHeader
        title={data.id}
        description={`Связанная заявка ${data.requestId}`}
        actions={<Link className="primary-button" to={`/requests/${data.requestId}`}>Открыть заявку</Link>}
      />
      <section className="detail-layout">
        <div className="panel">
          <StatusBadge value={data.status} />
          {error ? <p className="form-error">{error}</p> : null}
          <dl className="definition-list">
            <dt>Объект</dt><dd>{data.objectName}</dd>
            <dt>Исполнитель</dt><dd>{data.assignee}</dd>
            <dt>Трудозатраты</dt><dd>{data.actualHours ?? 0} ч</dd>
          </dl>
          <h2>Чеклист</h2>
          {data.tasks.map((task) => <label key={task.id} className="check-row"><input type="checkbox" defaultChecked={task.done} />{task.title}</label>)}
          <div className="action-stack">
            <label>
              Исполнитель
              <select value={assignee} onChange={(event) => setAssignee(event.target.value)}>
                <option value="">Выберите исполнителя</option>
                {(technicians ?? []).map((item) => (
                  <option key={item.id} value={item.id}>
                    {item.fullName}
                  </option>
                ))}
              </select>
            </label>
            <button onClick={handleAssign} disabled={busyAction !== null || !(technicians ?? []).length}>
              {busyAction === "assign" ? "Назначаю..." : "Назначить исполнителя"}
            </button>
            <button onClick={handleStart} disabled={busyAction !== null || data.status !== "ASSIGNED"}>
              {busyAction === "start" ? "Запускаю..." : "Запустить"}
            </button>
            <button onClick={handleComplete} disabled={busyAction !== null || data.status !== "IN_PROGRESS"}>
              {busyAction === "complete" ? "Завершаю..." : "Завершить"}
            </button>
          </div>
        </div>
        <aside className="panel">
          <h2>Подпись клиента</h2>
          <div className="signature-box">Canvas signature</div>
          <h2>Фотоотчет</h2>
          <div className="photo-grid"><span>До</span><span>После</span></div>
          {!(technicians ?? []).length ? (
            <p className="form-error">Нет исполнителей. Создайте техника в разделе `Пользователи`.</p>
          ) : null}
          {data.assignee === "Не назначен" ? (
            <p className="form-error">Наряд нельзя запустить, пока ему не назначен исполнитель.</p>
          ) : null}
        </aside>
      </section>
    </>
  );
}
