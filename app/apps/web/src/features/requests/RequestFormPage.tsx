import { FormEvent, useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import { serviceDeskApi } from "@/shared/api/serviceDeskApi";
import { PageHeader } from "@/shared/components/PageHeader";
import { useAsync } from "@/shared/hooks/useAsync";

export function RequestFormPage() {
  const navigate = useNavigate();
  const { data: objects } = useAsync(() => serviceDeskApi.objects.list(), []);
  const [objectId, setObjectId] = useState("");
  const [description, setDescription] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState("");

  const availableObjects = useMemo(() => objects ?? [], [objects]);

  async function handleSubmit(event: FormEvent) {
    event.preventDefault();
    setError("");

    const selectedObjectId = objectId || availableObjects[0]?.id;
    if (!selectedObjectId) {
      setError("Сначала создайте хотя бы один объект в системе.");
      return;
    }
    if (!description.trim()) {
      setError("Опишите проблему.");
      return;
    }

    try {
      setSubmitting(true);
      await serviceDeskApi.requests.create({
        objectId: selectedObjectId,
        description: description.trim(),
        title: description.trim()
      });
      navigate("/requests", { replace: true });
    } catch {
      setError("Не удалось создать заявку.");
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <>
      <PageHeader title="Создание заявки" description="Форма создает реальную заявку через backend API." />
      <form className="form-grid panel" onSubmit={handleSubmit}>
        <label>Тип<select defaultValue="EMERGENCY"><option value="EMERGENCY">Аварийное</option><option value="PLANNED">Плановое</option><option value="PREVENTIVE">Профилактика</option><option value="CONSULTATION">Консультация</option></select></label>
        <label>
          Объект
          <select value={objectId} onChange={(event) => setObjectId(event.target.value)}>
            <option value="">Выберите объект</option>
            {availableObjects.map((item) => (
              <option key={item.id} value={item.id}>
                {item.name}
              </option>
            ))}
          </select>
        </label>
        {!availableObjects.length ? (
          <p className="form-error span-2">Нет доступных объектов. Сначала создайте объект в разделе `Объекты`.</p>
        ) : null}
        <label>Приоритет<select><option>Авто</option><option>Critical</option><option>High</option><option>Medium</option></select></label>
        <label>Желаемый срок<input type="datetime-local" /></label>
        <label className="span-2">Описание<textarea rows={6} placeholder="Опишите проблему" value={description} onChange={(event) => setDescription(event.target.value)} /></label>
        <label>Контактное лицо<input /></label>
        <label>Вложения<input type="file" multiple /></label>
        {error ? <p className="form-error span-2">{error}</p> : null}
        <button className="primary-button span-2" type="submit" disabled={submitting || !availableObjects.length}>
          {submitting ? "Создание..." : "Создать заявку"}
        </button>
      </form>
    </>
  );
}
