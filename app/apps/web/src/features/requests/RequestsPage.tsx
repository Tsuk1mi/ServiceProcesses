import { Link } from "react-router-dom";
import { serviceDeskApi } from "@/shared/api/serviceDeskApi";
import { DataTable } from "@/shared/components/DataTable";
import { EmptyState } from "@/shared/components/EmptyState";
import { PageHeader } from "@/shared/components/PageHeader";
import { StatusBadge } from "@/shared/components/StatusBadge";
import { useAsync } from "@/shared/hooks/useAsync";
import { formatDateTime } from "@/shared/utils/format";
import type { ServiceRequest } from "@/shared/types/domain";

export function RequestsPage() {
  const { data } = useAsync(() => serviceDeskApi.requests.list(), []);
  const requests = data ?? [];

  return (
    <>
      <PageHeader title="Заявки" description="Список, фильтры, быстрые пресеты и bulk-действия." actions={<Link className="primary-button" to="/requests/new">Новая заявка</Link>} />
      <div className="toolbar">
        <input placeholder="Поиск по ID, описанию, объекту" />
        <select><option>Все статусы</option><option>NEW</option><option>IN_PROGRESS</option></select>
        <select><option>Все приоритеты</option><option>CRITICAL</option><option>HIGH</option></select>
        <button>Мои</button><button>Срочные</button><button>Просроченные</button><button>Экспорт</button>
      </div>
      {requests.length ? (
        <DataTable<ServiceRequest>
          data={requests}
          columns={[
            { key: "id", title: "ID", render: (item) => <Link to={`/requests/${item.id}`}>{item.id}</Link> },
            { key: "title", title: "Заявка", render: (item) => item.title },
            { key: "object", title: "Объект", render: (item) => item.objectName },
            { key: "status", title: "Статус", render: (item) => <StatusBadge value={item.status} /> },
            { key: "priority", title: "Приоритет", render: (item) => <StatusBadge value={item.priority} /> },
            { key: "sla", title: "SLA", render: (item) => <StatusBadge value={item.slaStatus} /> },
            { key: "due", title: "Срок", render: (item) => formatDateTime(item.dueAt) }
          ]}
        />
      ) : (
        <section className="panel">
          <EmptyState
            title="Заявок пока нет"
            description="Сначала создайте объект, затем оформите по нему первую заявку."
            action={
              <div className="action-stack">
                <Link className="primary-button" to="/objects/new">Создать объект</Link>
                <Link className="primary-button" to="/requests/new">Создать заявку</Link>
              </div>
            }
          />
        </section>
      )}
    </>
  );
}
