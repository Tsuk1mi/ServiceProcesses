import { PageHeader } from "@/shared/components/PageHeader";

export function RolesPage() {
  return (
    <>
      <PageHeader title="Роли и права" />
      <section className="panel permissions-grid">
        {["requests", "work-orders", "objects", "sla", "users", "settings"].map((resource) => (
          <div key={resource}>
            <strong>{resource}</strong>
            <label><input type="checkbox" defaultChecked />read</label>
            <label><input type="checkbox" />write</label>
            <label><input type="checkbox" />delete</label>
          </div>
        ))}
      </section>
    </>
  );
}
