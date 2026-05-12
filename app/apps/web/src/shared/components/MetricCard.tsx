export function MetricCard({ label, value, tone = "default" }: { label: string; value: string | number; tone?: "default" | "danger" | "success" }) {
  return (
    <article className={`metric-card metric-${tone}`}>
      <span>{label}</span>
      <strong>{value}</strong>
    </article>
  );
}
