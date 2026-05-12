export type Role = "ADMIN" | "MANAGER" | "DISPATCHER" | "TECHNICIAN" | "CLIENT";

export type Priority = "LOW" | "MEDIUM" | "HIGH" | "CRITICAL";

export type RequestStatus =
  | "NEW"
  | "PLANNED"
  | "IN_PROGRESS"
  | "RESOLVED"
  | "CLOSED"
  | "CANCELLED"
  | "ESCALATED";

export type SlaStatus = "OK" | "WARNING" | "BREACHED";

export type WorkOrderStatus =
  | "NEW"
  | "ASSIGNED"
  | "IN_PROGRESS"
  | "PAUSED"
  | "COMPLETED"
  | "CANCELLED";

export interface User {
  id: string;
  name: string;
  email: string;
  phone?: string;
  role: Role;
  team?: string;
  status: "ACTIVE" | "BLOCKED";
  workload?: number;
}

export interface Technician {
  id: string;
  fullName: string;
  skills: string[];
  isActive: boolean;
}

export interface ServiceObject {
  id: string;
  name: string;
  type: string;
  serialNumber: string;
  address: string;
  status: "OPERATIONAL" | "MAINTENANCE" | "FAILED";
  manufacturer: string;
  model: string;
  installedAt: string;
}

export interface ServiceRequest {
  id: string;
  title: string;
  description: string;
  type: "PLANNED" | "EMERGENCY" | "PREVENTIVE" | "CONSULTATION";
  category: string;
  status: RequestStatus;
  priority: Priority;
  slaStatus: SlaStatus;
  objectId: string;
  objectName: string;
  requester: string;
  assignee?: string;
  createdAt: string;
  dueAt: string;
}

export interface WorkOrder {
  id: string;
  requestId: string;
  objectName: string;
  assignee: string;
  status: WorkOrderStatus;
  plannedStart: string;
  actualHours?: number;
  tasks: Array<{ id: string; title: string; done: boolean }>;
}

export interface SlaPolicy {
  id: string;
  name: string;
  objectType: string;
  reactionMinutes: Record<Priority, number>;
  resolutionMinutes: Record<Priority, number>;
  schedule: "BUSINESS_HOURS" | "TWENTY_FOUR_SEVEN";
}

export interface Escalation {
  id: string;
  requestId: string;
  level: "L1" | "L2" | "L3";
  reason: "SLA_BREACH" | "NO_RESPONSE" | "MANUAL";
  target: string;
  elapsedMinutes: number;
}

export interface AuditRecord {
  id: string;
  requestId?: string;
  entity: string;
  action: string;
  actorRole: string;
  actorId?: string;
  details: string;
  createdAtUtc: string;
}

export interface Notification {
  id: string;
  type: "REQUEST" | "ASSIGNMENT" | "SLA" | "ESCALATION" | "COMMENT" | "STATUS";
  title: string;
  body: string;
  href: string;
  read: boolean;
  createdAt: string;
}

export interface DashboardSummary {
  newRequests: number;
  inProgress: number;
  slaBreached: number;
  completedToday: number;
  slaCompliance: number;
  workloadByTechnician: Array<{ name: string; closed: number }>;
  activity: Array<{ id: string; text: string; at: string }>;
}
