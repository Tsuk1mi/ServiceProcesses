import { useState } from "react";
import { serviceDeskApi } from "@/shared/api/serviceDeskApi";
import { ApiError } from "@/shared/api/http";
import { DataTable } from "@/shared/components/DataTable";
import { EmptyState } from "@/shared/components/EmptyState";
import { PageHeader } from "@/shared/components/PageHeader";
import { StatusBadge } from "@/shared/components/StatusBadge";
import { useAsync } from "@/shared/hooks/useAsync";
import type { Escalation } from "@/shared/types/domain";

export function EscalationsPage() {
  const [reloadKey, setReloadKey] = useState(0);
  const [busyId, setBusyId] = useState<string | null>(null);
  const [error, setError] = useState("");
  const { data } = useAsync(() => serviceDeskApi.escalations.list(), [reloadKey]);
  const escalations = data ?? [];

  async function handleResolve(id: string) {
    try {
      setBusyId(id);
      setError("");
      await serviceDeskApi.escalations.resolve(id);
      setReloadKey((value) => value + 1);
    } catch (reason) {
      setError(reason instanceof ApiError ? `Не удалось завершить эскалацию. Код API: ${reason.status}.` : "Не удалось завершить эскалацию.");
    } finally {
      setBusyId(null);
    }
  }

  return (
    <>
      <PageHeader title="Эскалации" description="Активные уровни L1-L3 и правила уведомлений." />
      {error ? <section className="panel"><p className="form-error">{error}</p></section> : null}
      {escalations.length ? (
        <DataTable<Escalation>
          data={escalations}
          columns={[
            { key: "level", title: "Уровень", render: (item) => <StatusBadge value={item.level} /> },
            { key: "request", title: "Заявка", render: (item) => item.requestId },
            { key: "reason", title: "Причина", render: (item) => item.reason },
            { key: "target", title: "Кому", render: (item) => item.target },
            { key: "elapsed", title: "Прошло", render: (item) => `${item.elapsedMinutes} мин` },
            {
              key: "actions",
              title: "Действия",
              render: (item) => (
                <button onClick={() => handleResolve(item.id)} disabled={busyId !== null}>
                  {busyId === item.id ? "Завершаю..." : "Resolve"}
                </button>
              )
            }
          ]}
        />
      ) : (
        <section className="panel">
          <EmptyState
            title="Эскалаций пока нет"
            description="Когда заявки начнут нарушать SLA или их эскалируют вручную, они появятся здесь."
          />
        </section>
      )}
    </>
  );
}
