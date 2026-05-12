import { FormEvent, useState } from "react";
import { serviceDeskApi } from "@/shared/api/serviceDeskApi";
import { ApiError } from "@/shared/api/http";
import { DataTable } from "@/shared/components/DataTable";
import { EmptyState } from "@/shared/components/EmptyState";
import { PageHeader } from "@/shared/components/PageHeader";
import { StatusBadge } from "@/shared/components/StatusBadge";
import { useAsync } from "@/shared/hooks/useAsync";
import type { Technician, User } from "@/shared/types/domain";

export function UsersPage() {
  const [reloadKey, setReloadKey] = useState(0);
  const [fullName, setFullName] = useState("");
  const [skills, setSkills] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState("");
  const { data: users } = useAsync(() => serviceDeskApi.users.list(), []);
  const { data: technicians } = useAsync(() => serviceDeskApi.technicians.list(), [reloadKey]);

  async function handleSubmit(event: FormEvent) {
    event.preventDefault();
    setError("");

    const trimmedName = fullName.trim();
    const parsedSkills = skills
      .split(",")
      .map((item) => item.trim())
      .filter(Boolean);

    if (!trimmedName) {
      setError("Введите ФИО исполнителя.");
      return;
    }

    try {
      setSubmitting(true);
      await serviceDeskApi.technicians.create({ fullName: trimmedName, skills: parsedSkills });
      setFullName("");
      setSkills("");
      setReloadKey((value) => value + 1);
    } catch (reason) {
      setError(reason instanceof ApiError ? `Ошибка API: ${reason.status}` : "Не удалось создать исполнителя.");
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <>
      <PageHeader title="Пользователи" description="ADMIN: управление доступом и bootstrap исполнителей для нарядов." />
      <section className="dashboard-grid">
        <section className="panel">
          <h2>Администратор и пользователи</h2>
          <DataTable<User>
            data={users ?? []}
            columns={[
              { key: "name", title: "ФИО", render: (item) => item.name },
              { key: "email", title: "Email", render: (item) => item.email },
              { key: "role", title: "Роль", render: (item) => <StatusBadge value={item.role} /> },
              { key: "team", title: "Команда", render: (item) => item.team ?? "-" },
              { key: "workload", title: "Загрузка", render: (item) => item.workload ?? 0 }
            ]}
          />
        </section>
        <section className="panel">
          <h2>Добавить исполнителя</h2>
          <form className="form-grid" onSubmit={handleSubmit}>
            <label className="span-2">
              ФИО
              <input value={fullName} onChange={(event) => setFullName(event.target.value)} placeholder="Иван Иванов" />
            </label>
            <label className="span-2">
              Навыки
              <input
                value={skills}
                onChange={(event) => setSkills(event.target.value)}
                placeholder="electrical, inspection"
              />
            </label>
            {error ? <p className="form-error span-2">{error}</p> : null}
            <button className="primary-button span-2" type="submit" disabled={submitting}>
              {submitting ? "Создание..." : "Создать исполнителя"}
            </button>
          </form>
        </section>
      </section>
      <section className="panel">
        <h2>Исполнители</h2>
        {(technicians ?? []).length ? (
          <DataTable<Technician>
            data={technicians ?? []}
            columns={[
              { key: "fullName", title: "ФИО", render: (item) => item.fullName },
              { key: "skills", title: "Навыки", render: (item) => item.skills.join(", ") || "-" },
              { key: "active", title: "Статус", render: (item) => <StatusBadge value={item.isActive ? "ACTIVE" : "BLOCKED"} /> }
            ]}
          />
        ) : (
          <EmptyState title="Исполнителей пока нет" description="Создайте первого техника, чтобы можно было назначать наряды." />
        )}
      </section>
    </>
  );
}
