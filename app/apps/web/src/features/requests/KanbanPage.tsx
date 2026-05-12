import { serviceDeskApi } from "@/shared/api/serviceDeskApi";
import { EmptyState } from "@/shared/components/EmptyState";
import { StatusBadge } from "@/shared/components/StatusBadge";
import { useAsync } from "@/shared/hooks/useAsync";
import type { RequestStatus } from "@/shared/types/domain";
import { Link } from "react-router-dom";

const columns: RequestStatus[] = ["NEW", "PLANNED", "IN_PROGRESS", "RESOLVED"];

export function RequestsKanbanPage() {
  const { data } = useAsync(() => serviceDeskApi.requests.list(), []);
  const requests = data ?? [];
  if (!requests.length) {
    return (
      <section className="panel">
        <EmptyState
          title="Канбан пока пуст"
          description="Создайте первую заявку, и она появится в колонке NEW."
          action={<Link className="primary-button" to="/requests/new">Создать заявку</Link>}
        />
      </section>
    );
  }

  return (
    <div className="kanban">
      {columns.map((status) => (
        <section key={status} className="kanban-column">
          <h2>{status}</h2>
          {requests.filter((item) => item.status === status).map((item) => (
            <article key={item.id} className="kanban-card">
              <strong>{item.id}</strong>
              <p>{item.title}</p>
              <StatusBadge value={item.priority} />
            </article>
          ))}
        </section>
      ))}
    </div>
  );
}
