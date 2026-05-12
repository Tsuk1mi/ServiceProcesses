import { FormEvent, useState } from "react";
import { useNavigate } from "react-router-dom";
import { serviceDeskApi } from "@/shared/api/serviceDeskApi";
import { PageHeader } from "@/shared/components/PageHeader";

export function ObjectCreatePage() {
  const navigate = useNavigate();
  const [type, setType] = useState("building");
  const [name, setName] = useState("");
  const [address, setAddress] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState("");

  async function handleSubmit(event: FormEvent) {
    event.preventDefault();
    setError("");

    if (!name.trim() || !address.trim()) {
      setError("Заполните название и адрес объекта.");
      return;
    }

    try {
      setSubmitting(true);
      await serviceDeskApi.objects.create({
        type: type.trim(),
        name: name.trim(),
        address: address.trim()
      });
      navigate("/objects", { replace: true });
    } catch {
      setError("Не удалось создать объект.");
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <>
      <PageHeader title="Добавить объект" description="Форма создает реальный объект через backend API." />
      <form className="form-grid panel" onSubmit={handleSubmit}>
        <label>
          Тип
          <select value={type} onChange={(event) => setType(event.target.value)}>
            <option value="building">Здание</option>
            <option value="chiller">Чиллер</option>
            <option value="ups">ИБП</option>
            <option value="generator">Генератор</option>
          </select>
        </label>
        <label>
          Название
          <input value={name} onChange={(event) => setName(event.target.value)} placeholder="Склад N1" />
        </label>
        <label className="span-2">
          Адрес
          <input value={address} onChange={(event) => setAddress(event.target.value)} placeholder="Москва, ул. Примерная, 1" />
        </label>
        {error ? <p className="form-error span-2">{error}</p> : null}
        <button className="primary-button span-2" type="submit" disabled={submitting}>
          {submitting ? "Создание..." : "Создать объект"}
        </button>
      </form>
    </>
  );
}
