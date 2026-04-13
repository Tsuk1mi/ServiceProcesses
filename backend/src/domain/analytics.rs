use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DashboardSummary {
    pub total_requests: usize,
    pub open_requests: usize,
    pub in_progress_requests: usize,
    pub resolved_requests: usize,
    pub closed_requests: usize,
    pub overdue_requests: usize,
    pub total_work_orders: usize,
    pub active_work_orders: usize,
    pub open_escalations: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SlaComplianceSummary {
    pub total_open_requests: usize,
    pub overdue_open_requests: usize,
    pub compliant_open_requests: usize,
    pub compliance_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SlaComplianceByPriorityItem {
    pub priority: String,
    pub total_open_requests: usize,
    pub overdue_open_requests: usize,
    pub compliant_open_requests: usize,
    pub compliance_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TechnicianWorkloadSummary {
    pub technician_id: String,
    pub full_name: String,
    pub assigned: usize,
    pub in_progress: usize,
    pub completed: usize,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AnalyticsSnapshot {
    pub updated_at_epoch_sec: u64,
    pub dashboard: DashboardSummary,
    pub sla_compliance: SlaComplianceSummary,
    pub sla_compliance_by_priority: Vec<SlaComplianceByPriorityItem>,
    pub technician_workload: Vec<TechnicianWorkloadSummary>,
}

