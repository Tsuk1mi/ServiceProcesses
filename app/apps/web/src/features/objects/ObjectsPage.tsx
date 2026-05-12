import { Link } from "react-router-dom";
import { serviceDeskApi } from "@/shared/api/serviceDeskApi";
import { DataTable } from "@/shared/components/DataTable";
import { EmptyState } from "@/shared/components/EmptyState";
import { PageHeader } from "@/shared/components/PageHeader";
import { StatusBadge } from "@/shared/components/StatusBadge";
import { useAsync } from "@/shared/hooks/useAsync";
import type { ServiceObject } from "@/shared/types/domain";

export function ObjectsPage() {
  const { data } = useAsync(() => serviceDeskApi.objects.list(), []);
  const objects = data ?? [];
  return (
    <>
      <PageHeader title="Реестр объектов" actions={<Link className="primary-button" to="/objects/new">Добавить объект</Link>} />
      <section className="split-grid">
        {objects.length ? (
          <DataTable<ServiceObject>
            data={objects}
            columns={[
              { key: "name", title: "Объект", render: (item) => <Link to={`/objects/${item.id}`}>{item.name}</Link> },
              { key: "type", title: "Тип", render: (item) => item.type },
              { key: "serial", title: "Серийный номер", render: (item) => item.serialNumber },
              { key: "status", title: "Статус", render: (item) => <StatusBadge value={item.status} /> }
            ]}
          />
        ) : (
          <section className="panel">
            <EmptyState
              title="Объектов пока нет"
              description="Создайте первый объект обслуживания, чтобы затем оформлять по нему заявки."
              action={<Link className="primary-button" to="/objects/new">Создать объект</Link>}
            />
          </section>
        )}
        <div className="map-placeholder">Карта объектов</div>
      </section>
    </>
  );
}
