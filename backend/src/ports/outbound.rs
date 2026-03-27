use crate::domain::entities::{Asset, AuditRecord, Escalation, ServiceRequest, Technician, WorkOrder};
use crate::domain::errors::DomainError;
use crate::domain::analytics::{
    DashboardSummary, SlaComplianceByPriorityItem, SlaComplianceSummary, TechnicianWorkloadSummary,
    AnalyticsSnapshot,
};
use crate::domain::value_objects::Priority;

pub trait AssetRepository {
    fn save(&self, asset: Asset) -> Result<(), DomainError>;
    fn get_by_id(&self, id: &str) -> Result<Option<Asset>, DomainError>;
    fn list(&self) -> Result<Vec<Asset>, DomainError>;
}

pub trait ServiceRequestRepository {
    fn save(&self, request: ServiceRequest) -> Result<(), DomainError>;
    fn get_by_id(&self, id: &str) -> Result<Option<ServiceRequest>, DomainError>;
    fn list(&self) -> Result<Vec<ServiceRequest>, DomainError>;
    fn update(&self, request: ServiceRequest) -> Result<(), DomainError>;
}

pub trait SlaPolicyPort {
    fn resolve_sla_minutes(&self, priority: Priority) -> Result<u32, DomainError>;
}

pub trait PriorityPolicyPort {
    fn resolve_priority(&self, description: &str) -> Result<Priority, DomainError>;
}

pub trait EventPublisherPort {
    fn publish(&self, topic: &str, payload: &str) -> Result<(), DomainError>;
}

pub trait WorkOrderRepository {
    fn save(&self, work_order: WorkOrder) -> Result<(), DomainError>;
    fn get_by_id(&self, id: &str) -> Result<Option<WorkOrder>, DomainError>;
    fn list(&self) -> Result<Vec<WorkOrder>, DomainError>;
    fn list_by_request(&self, request_id: &str) -> Result<Vec<WorkOrder>, DomainError>;
    fn update(&self, work_order: WorkOrder) -> Result<(), DomainError>;
}

pub trait EscalationRepository {
    fn save(&self, escalation: Escalation) -> Result<(), DomainError>;
    fn get_by_id(&self, id: &str) -> Result<Option<Escalation>, DomainError>;
    fn list(&self) -> Result<Vec<Escalation>, DomainError>;
    fn list_by_request(&self, request_id: &str) -> Result<Vec<Escalation>, DomainError>;
    fn update(&self, escalation: Escalation) -> Result<(), DomainError>;
}

pub trait TechnicianRepository {
    fn save(&self, technician: Technician) -> Result<(), DomainError>;
    fn get_by_id(&self, id: &str) -> Result<Option<Technician>, DomainError>;
    fn list(&self) -> Result<Vec<Technician>, DomainError>;
}

pub trait AuditRepository {
    fn save(&self, record: AuditRecord) -> Result<(), DomainError>;
    fn list(&self) -> Result<Vec<AuditRecord>, DomainError>;
    fn list_by_request(&self, request_id: &str) -> Result<Vec<AuditRecord>, DomainError>;
}

pub trait AnalyticsQueryPort {
    fn dashboard_summary(&self, now_epoch: u64) -> Result<DashboardSummary, DomainError>;
    fn sla_compliance_summary(&self, now_epoch: u64) -> Result<SlaComplianceSummary, DomainError>;
    fn sla_compliance_by_priority_summary(
        &self,
        now_epoch: u64,
    ) -> Result<Vec<SlaComplianceByPriorityItem>, DomainError>;
    fn technician_workload_summary(&self) -> Result<Vec<TechnicianWorkloadSummary>, DomainError>;

    fn list_requests(&self) -> Result<Vec<ServiceRequest>, DomainError>;
    fn list_work_orders(&self) -> Result<Vec<WorkOrder>, DomainError>;
    fn list_escalations(&self) -> Result<Vec<Escalation>, DomainError>;
    fn list_technicians(&self) -> Result<Vec<Technician>, DomainError>;
}

pub trait AnalyticsSnapshotRepository {
    fn get_latest(&self) -> Result<Option<AnalyticsSnapshot>, DomainError>;
    fn upsert(&self, snapshot: AnalyticsSnapshot) -> Result<(), DomainError>;
}
