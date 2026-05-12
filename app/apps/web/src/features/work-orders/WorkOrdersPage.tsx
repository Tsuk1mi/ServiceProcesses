import { Link } from "react-router-dom";
import { serviceDeskApi } from "@/shared/api/serviceDeskApi";
import { DataTable } from "@/shared/components/DataTable";
import { EmptyState } from "@/shared/components/EmptyState";
import { PageHeader } from "@/shared/components/PageHeader";
import { StatusBadge } from "@/shared/components/StatusBadge";
import { useAsync } from "@/shared/hooks/useAsync";
import { formatDateTime } from "@/shared/utils/format";
import type { WorkOrder } from "@/shared/types/domain";

export function WorkOrdersPage() {
  const { data } = useAsync(() => serviceDeskApi.workOrders.list(), []);
  const workOrders = data ?? [];
  return (
    <>
      <PageHeader title="Наряды" description="Исполнители, чеклисты, трудозатраты и фотоотчеты." />
      {workOrders.length ? (
        <DataTable<WorkOrder>
          data={workOrders}
          columns={[
            { key: "id", title: "Наряд", render: (item) => <Link to={`/work-orders/${item.id}`}>{item.id}</Link> },
            { key: "request", title: "Заявка", render: (item) => <Link to={`/requests/${item.requestId}`}>{item.requestId}</Link> },
            { key: "object", title: "Объект", render: (item) => item.objectName },
            { key: "assignee", title: "Исполнитель", render: (item) => item.assignee },
            { key: "status", title: "Статус", render: (item) => <StatusBadge value={item.status} /> },
            { key: "planned", title: "План", render: (item) => formatDateTime(item.plannedStart) }
          ]}
        />
      ) : (
        <section className="panel">
          <EmptyState
            title="Нарядов пока нет"
            description="Они появятся после обработки заявок и назначения работ исполнителям."
            action={<Link className="primary-button" to="/requests">Перейти к заявкам</Link>}
          />
        </section>
      )}
    </>
  );
}
