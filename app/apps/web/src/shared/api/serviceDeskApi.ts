import { apiRequest } from "@/shared/api/http";
import type {
  AuditRecord,
  DashboardSummary,
  Escalation,
  Notification,
  ServiceObject,
  ServiceRequest,
  SlaPolicy,
  Technician,
  User,
  WorkOrder
} from "@/shared/types/domain";

interface BffAuthUser {
  id: string;
  name: string;
  email: string;
  role: string;
  status: string;
  team?: string;
  workload?: number;
}

interface BffLoginResponse {
  accessToken: string;
  refreshToken?: string | null;
  expiresIn: number;
  user: BffAuthUser;
}

interface BffWebDashboard {
  newRequests: number;
  inProgress: number;
  slaBreached: number;
  completedToday: number;
  slaCompliance: number;
  workloadByTechnician: Array<{ name: string; closed: number }>;
  activity: Array<{ id: string; text: string; at: string }>;
}

interface BffWebRequest {
  id: string;
  title: string;
  type: string;
  category: string;
  status: string;
  priority: string;
  slaStatus: string;
  objectId: string;
  objectName: string;
  requester: string;
  assignee?: string | null;
  createdAt: string;
  dueAt: string;
}

interface BffRequestsListResponse {
  items: BffWebRequest[];
}

interface WorkOrderCommandResponse {
  id: string;
  request_id: string;
  assignee?: string | null;
  status: string;
  owner_user_id: string;
}

interface ApiTechnician {
  id: string;
  full_name: string;
  skills: string[];
  is_active: boolean;
  owner_user_id: string;
}

interface ApiEscalation {
  id: string;
  request_id: string;
  reason: string;
  state: string;
  owner_user_id: string;
}

interface ApiAuditRecord {
  id: string;
  request_id?: string | null;
  entity: string;
  action: string;
  actor_role: string;
  actor_id?: string | null;
  details: string;
  created_at_utc: string;
  owner_user_id: string;
}

const mapRole = (role: string): User["role"] => {
  switch (role.toUpperCase()) {
    case "ADMIN":
      return "ADMIN";
    case "DISPATCHER":
      return "DISPATCHER";
    case "TECHNICIAN":
      return "TECHNICIAN";
    case "MANAGER":
      return "MANAGER";
    default:
      return "CLIENT";
  }
};

const mapUser = (user: BffAuthUser): User => ({
  id: user.id,
  name: user.name,
  email: user.email,
  role: mapRole(user.role),
  team: user.team,
  status: user.status === "BLOCKED" ? "BLOCKED" : "ACTIVE",
  workload: user.workload
});

const mapRequestStatus = (status: string): ServiceRequest["status"] => {
  switch (status.toUpperCase()) {
    case "NEW":
      return "NEW";
    case "PLANNED":
      return "PLANNED";
    case "IN_PROGRESS":
      return "IN_PROGRESS";
    case "RESOLVED":
      return "RESOLVED";
    case "CLOSED":
      return "CLOSED";
    case "ESCALATED":
      return "ESCALATED";
    case "CANCELLED":
      return "CANCELLED";
    default:
      return "NEW";
  }
};

const mapPriority = (priority: string): ServiceRequest["priority"] => {
  switch (priority.toUpperCase()) {
    case "CRITICAL":
      return "CRITICAL";
    case "HIGH":
      return "HIGH";
    case "LOW":
      return "LOW";
    default:
      return "MEDIUM";
  }
};

const mapRequest = (item: BffWebRequest): ServiceRequest => ({
  id: item.id,
  title: item.title,
  description: item.title,
  type: (item.type.toUpperCase() as ServiceRequest["type"]) ?? "PLANNED",
  category: item.category,
  status: mapRequestStatus(item.status),
  priority: mapPriority(item.priority),
  slaStatus: item.slaStatus.toUpperCase() === "BREACHED" ? "BREACHED" : item.slaStatus.toUpperCase() === "WARNING" ? "WARNING" : "OK",
  objectId: item.objectId,
  objectName: item.objectName,
  requester: item.requester,
  assignee: item.assignee ?? undefined,
  createdAt: item.createdAt,
  dueAt: item.dueAt
});

const mapDashboard = (item: BffWebDashboard): DashboardSummary => ({
  newRequests: item.newRequests,
  inProgress: item.inProgress,
  slaBreached: item.slaBreached,
  completedToday: item.completedToday,
  slaCompliance: item.slaCompliance,
  workloadByTechnician: item.workloadByTechnician,
  activity: item.activity
});

const mapTechnician = (item: ApiTechnician): Technician => ({
  id: item.id,
  fullName: item.full_name,
  skills: item.skills,
  isActive: item.is_active
});

const mapEscalation = (item: ApiEscalation): Escalation => ({
  id: item.id,
  requestId: item.request_id,
  level: "L1",
  reason: item.reason.toUpperCase().includes("SLA") ? "SLA_BREACH" : "MANUAL",
  target: item.state.toUpperCase() === "RESOLVED" ? "resolved" : "dispatcher",
  elapsedMinutes: item.state.toUpperCase() === "RESOLVED" ? 0 : 15
});

const mapAuditRecord = (item: ApiAuditRecord): AuditRecord => ({
  id: item.id,
  requestId: item.request_id ?? undefined,
  entity: item.entity,
  action: item.action,
  actorRole: item.actor_role,
  actorId: item.actor_id ?? undefined,
  details: item.details,
  createdAtUtc: item.created_at_utc
});

export const serviceDeskApi = {
  auth: {
    login: (body: { username: string; password: string }) =>
      apiRequest<BffLoginResponse, { username: string; password: string }>("/bff/web/auth/login", {
        method: "POST",
        body
      }).then((response) => ({
        accessToken: response.accessToken,
        refreshToken: response.refreshToken ?? null,
        expiresIn: response.expiresIn,
        user: mapUser(response.user)
      })),
    refresh: (refreshToken: string) =>
      apiRequest<BffLoginResponse, { refresh_token: string }>("/bff/web/auth/refresh", {
        method: "POST",
        body: { refresh_token: refreshToken }
      }).then((response) => ({
        accessToken: response.accessToken,
        refreshToken: response.refreshToken ?? null,
        expiresIn: response.expiresIn,
        user: mapUser(response.user)
      })),
    me: () => apiRequest<BffAuthUser>("/bff/web/auth/me").then(mapUser),
    logout: () => apiRequest<{ ok: boolean }>("/bff/web/auth/logout", { method: "POST" })
  },
  dashboard: {
    getSummary: () => apiRequest<BffWebDashboard>("/bff/web/dashboard").then(mapDashboard)
  },
  requests: {
    list: () => apiRequest<BffRequestsListResponse>("/bff/web/requests").then((response) => response.items.map(mapRequest)),
    get: (id: string) => apiRequest<BffWebRequest>(`/bff/web/requests/${id}`).then(mapRequest),
    create: (body: Partial<ServiceRequest>) =>
      apiRequest<{ result: string }, { asset_id: string; description: string }>("/commands/requests", {
        method: "POST",
        body: { asset_id: body.objectId ?? "", description: body.description ?? body.title ?? "Новая заявка" }
      }).then(() => undefined),
    updateStatus: (id: string, status: "new" | "planned" | "in_progress" | "resolved" | "closed" | "escalated") =>
      apiRequest<{ result: string }, { status: string }>(`/commands/requests/${id}/status`, {
        method: "PUT",
        body: { status }
      }),
    createWorkOrder: (requestId: string) =>
      apiRequest<WorkOrderCommandResponse, { request_id: string }>("/commands/work-orders", {
        method: "POST",
        body: { request_id: requestId }
      })
  },
  workOrders: {
    list: () => apiRequest<WorkOrder[]>("/bff/web/work-orders"),
    get: (id: string) => apiRequest<WorkOrder>(`/bff/web/work-orders/${id}`),
    assign: (id: string, assignee: string) =>
      apiRequest<WorkOrderCommandResponse, { assignee: string }>(`/commands/work-orders/${id}/assign`, {
        method: "PUT",
        body: { assignee }
      }),
    start: (id: string) => apiRequest<WorkOrder>(`/work-orders/${id}/start`, { method: "PUT" }),
    complete: (id: string) => apiRequest<WorkOrder>(`/work-orders/${id}/complete`, { method: "PUT" })
  },
  objects: {
    list: () => apiRequest<ServiceObject[]>("/bff/web/objects"),
    get: (id: string) => apiRequest<ServiceObject>(`/bff/web/objects/${id}`),
    create: (body: { type: string; name: string; address: string }) =>
      apiRequest("/commands/assets", {
        method: "POST",
        body: {
          kind: body.type,
          title: body.name,
          location: body.address
        }
      })
  },
  sla: {
    policies: () => apiRequest<SlaPolicy[]>("/bff/web/sla/policies"),
    breaches: () => apiRequest<BffWebRequest[]>("/bff/web/sla/breaches").then((items) => items.map(mapRequest))
  },
  escalations: {
    list: () => apiRequest<Escalation[]>("/bff/web/escalations"),
    listByRequest: (requestId: string) =>
      apiRequest<ApiEscalation[]>(`/requests/${requestId}/escalations`).then((items) => items.map(mapEscalation)),
    create: (requestId: string, reason: string) =>
      apiRequest<ApiEscalation, { request_id: string; reason: string }>("/escalations", {
        method: "POST",
        body: { request_id: requestId, reason }
      }).then(mapEscalation),
    resolve: (id: string) => apiRequest<ApiEscalation>(`/escalations/${id}/resolve`, { method: "PUT" }).then(mapEscalation)
  },
  analytics: {
    overview: () => apiRequest<BffWebDashboard>("/bff/web/analytics/overview").then(mapDashboard)
  },
  users: {
    list: () => apiRequest<BffAuthUser[]>("/bff/web/users").then((items) => items.map(mapUser))
  },
  technicians: {
    list: () => apiRequest<ApiTechnician[]>("/technicians").then((items) => items.map(mapTechnician)),
    create: (body: { fullName: string; skills: string[] }) =>
      apiRequest<ApiTechnician, { full_name: string; skills: string[] }>("/commands/technicians", {
        method: "POST",
        body: {
          full_name: body.fullName,
          skills: body.skills
        }
      }).then(mapTechnician)
  },
  audit: {
    requestHistory: (requestId: string) =>
      apiRequest<ApiAuditRecord[]>(`/requests/${requestId}/audit`).then((items) => items.map(mapAuditRecord))
  },
  notifications: {
    list: () => apiRequest<Notification[]>("/bff/web/notifications")
  },
  slaAdmin: {
    escalateOverdue: () => apiRequest<{ created: number }>("/sla/escalate-overdue", { method: "POST" })
  }
};
