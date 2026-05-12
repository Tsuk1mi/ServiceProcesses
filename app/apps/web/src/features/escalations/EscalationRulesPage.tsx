import { PageHeader } from "@/shared/components/PageHeader";

export function EscalationRulesPage() {
  return (
    <>
      <PageHeader title="Правила эскалации" />
      <form className="form-grid panel">
        <label>Триггер<select><option>SLA нарушен</option><option>Нет реакции</option></select></label>
        <label>Задержка<select><option>30 минут</option><option>2 часа</option></select></label>
        <label>Действие<select><option>Уведомить менеджера</option><option>Переназначить</option></select></label>
        <label>Следующий уровень<select><option>Через 2 часа</option><option>Через 4 часа</option></select></label>
        <label className="check-row"><input type="checkbox" defaultChecked />Email</label>
        <label className="check-row"><input type="checkbox" defaultChecked />Push</label>
        <button className="primary-button span-2" type="button">Сохранить правило</button>
      </form>
    </>
  );
}
