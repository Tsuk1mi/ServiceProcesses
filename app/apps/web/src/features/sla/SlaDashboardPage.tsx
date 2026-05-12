import { useState } from "react";
import { serviceDeskApi } from "@/shared/api/serviceDeskApi";
import { ApiError } from "@/shared/api/http";
import { EmptyState } from "@/shared/components/EmptyState";
import { PageHeader } from "@/shared/components/PageHeader";
import { StatusBadge } from "@/shared/components/StatusBadge";
import { useAsync } from "@/shared/hooks/useAsync";
import { Link } from "react-router-dom";

export function SlaDashboardPage() {
  const [reloadKey, setReloadKey] = useState(0);
  const [running, setRunning] = useState(false);
  const [error, setError] = useState("");
  const { data } = useAsync(() => serviceDeskApi.sla.breaches(), [reloadKey]);
  const breaches = data ?? [];

  async function handleEscalateOverdue() {
    try {
      setRunning(true);
      setError("");
      await serviceDeskApi.slaAdmin.escalateOverdue();
      setReloadKey((value) => value + 1);
    } catch (reason) {
      setError(reason instanceof ApiError ? `Не удалось запустить SLA escalation. Код API: ${reason.status}.` : "Не удалось запустить SLA escalation.");
    } finally {
      setRunning(false);
    }
  }

  return (
    <>
      <PageHeader
        title="SLA мониторинг"
        description="Общий обзор, угрозы и журнал нарушений."
        actions={
          <button className="primary-button" onClick={handleEscalateOverdue} disabled={running}>
            {running ? "Обрабатываю..." : "Запустить overdue escalation"}
          </button>
        }
      />
      {error ? <section className="panel"><p className="form-error">{error}</p></section> : null}
      {!breaches.length ? (
        <section className="panel">
          <EmptyState
            title="Нарушений SLA пока нет"
            description="После создания и накопления заявок здесь появятся риски и нарушения SLA."
            action={<Link className="primary-button" to="/requests/new">Создать заявку</Link>}
          />
        </section>
      ) : null}
      <section className="dashboard-grid">
        <div className="panel gauge-panel">
          <h2>Общий SLA</h2><strong>94.2%</strong><div className="progress"><span style={{ width: "94.2%" }} /></div>
        </div>
        <div className="panel">
          <h2>По приоритетам</h2>
          {["Critical 82%", "High 96%", "Medium 98%", "Low 99%"].map((item) => <p key={item}>{item}</p>)}
        </div>
        <div className="panel wide-panel">
          <h2>Заявки с угрозой SLA</h2>
          {breaches.map((item) => <p key={item.id}>{item.id} / {item.objectName} <StatusBadge value={item.slaStatus} /></p>)}
        </div>
        <div className="heatmap wide-panel">{Array.from({ length: 48 }).map((_, index) => <span key={index} />)}</div>
      </section>
    </>
  );
}
