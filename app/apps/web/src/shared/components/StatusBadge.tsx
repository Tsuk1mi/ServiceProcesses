import type { Priority, RequestStatus, SlaStatus, WorkOrderStatus } from "@/shared/types/domain";

type BadgeTone = "neutral" | "info" | "success" | "warning" | "danger";

const toneByValue: Record<string, BadgeTone> = {
  NEW: "info",
  PLANNED: "info",
  ASSIGNED: "info",
  IN_PROGRESS: "warning",
  RESOLVED: "success",
  CLOSED: "success",
  COMPLETED: "success",
  CANCELLED: "neutral",
  ESCALATED: "danger",
  LOW: "neutral",
  MEDIUM: "info",
  HIGH: "warning",
  CRITICAL: "danger",
  OK: "success",
  WARNING: "warning",
  BREACHED: "danger"
};

export function StatusBadge({ value }: { value: RequestStatus | WorkOrderStatus | Priority | SlaStatus | string }) {
  return <span className={`badge badge-${toneByValue[value] ?? "neutral"}`}>{value}</span>;
}
