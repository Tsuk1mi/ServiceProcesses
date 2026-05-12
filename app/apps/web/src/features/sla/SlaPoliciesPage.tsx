import { serviceDeskApi } from "@/shared/api/serviceDeskApi";
import { DataTable } from "@/shared/components/DataTable";
import { PageHeader } from "@/shared/components/PageHeader";
import { useAsync } from "@/shared/hooks/useAsync";
import type { SlaPolicy } from "@/shared/types/domain";

export function SlaPoliciesPage() {
  const { data } = useAsync(() => serviceDeskApi.sla.policies(), []);
  return (
    <>
      <PageHeader title="Политики SLA" actions={<button className="primary-button">Создать политику</button>} />
      <DataTable<SlaPolicy>
        data={data ?? []}
        columns={[
          { key: "name", title: "Название", render: (item) => item.name },
          { key: "object", title: "Тип объекта", render: (item) => item.objectType },
          { key: "schedule", title: "График", render: (item) => item.schedule },
          { key: "critical", title: "Critical реакция", render: (item) => `${item.reactionMinutes.CRITICAL} мин` }
        ]}
      />
    </>
  );
}
