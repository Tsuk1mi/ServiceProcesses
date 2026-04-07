use crate::domain::analytics::{
    AnalyticsSnapshot, DashboardSummary, SlaComplianceByPriorityItem, SlaComplianceSummary,
    TechnicianWorkloadSummary,
};
use crate::domain::entities::{Asset, AuditRecord, Escalation, ServiceRequest, Technician, WorkOrder};
use crate::domain::errors::DomainError;
use crate::domain::value_objects::Priority;
use crate::ports::data_scope::DataScope;
use async_trait::async_trait;

#[async_trait]
pub trait AssetRepository: Send + Sync {
    /// `actor_scope`: при `Owner` запрещает создавать/менять чужие сущности.
    async fn save(&self, asset: Asset, actor_scope: DataScope) -> Result<(), DomainError>;
    async fn get_by_id(&self, id: &str, scope: DataScope) -> Result<Option<Asset>, DomainError>;
    async fn list(&self, scope: DataScope) -> Result<Vec<Asset>, DomainError>;
}

#[async_trait]
pub trait ServiceRequestRepository: Send + Sync {
    async fn save(&self, request: ServiceRequest, actor_scope: DataScope) -> Result<(), DomainError>;
    async fn get_by_id(&self, id: &str, scope: DataScope) -> Result<Option<ServiceRequest>, DomainError>;
    async fn list(&self, scope: DataScope) -> Result<Vec<ServiceRequest>, DomainError>;
    async fn update(&self, request: ServiceRequest, actor_scope: DataScope) -> Result<(), DomainError>;
    /// Eager: заявки с объектом в одном запросе (без N+1 на asset).
    async fn list_with_assets(
        &self,
        scope: DataScope,
    ) -> Result<Vec<(ServiceRequest, Option<Asset>)>, DomainError>;
}

pub trait SlaPolicyPort: Send + Sync {
    fn resolve_sla_minutes(&self, priority: Priority) -> Result<u32, DomainError>;
}

pub trait PriorityPolicyPort: Send + Sync {
    fn resolve_priority(&self, description: &str) -> Result<Priority, DomainError>;
}

#[async_trait]
pub trait EventPublisherPort: Send + Sync {
    async fn publish(&self, topic: &str, payload: &str) -> Result<(), DomainError>;
}

#[async_trait]
pub trait WorkOrderRepository: Send + Sync {
    async fn save(&self, work_order: WorkOrder, actor_scope: DataScope) -> Result<(), DomainError>;
    async fn get_by_id(&self, id: &str, scope: DataScope) -> Result<Option<WorkOrder>, DomainError>;
    async fn list(&self, scope: DataScope) -> Result<Vec<WorkOrder>, DomainError>;
    async fn list_by_request(
        &self,
        request_id: &str,
        scope: DataScope,
    ) -> Result<Vec<WorkOrder>, DomainError>;
    async fn update(&self, work_order: WorkOrder, actor_scope: DataScope) -> Result<(), DomainError>;
}

#[async_trait]
pub trait EscalationRepository: Send + Sync {
    async fn save(&self, escalation: Escalation, actor_scope: DataScope) -> Result<(), DomainError>;
    async fn get_by_id(&self, id: &str, scope: DataScope) -> Result<Option<Escalation>, DomainError>;
    async fn list(&self, scope: DataScope) -> Result<Vec<Escalation>, DomainError>;
    async fn list_by_request(
        &self,
        request_id: &str,
        scope: DataScope,
    ) -> Result<Vec<Escalation>, DomainError>;
    async fn update(&self, escalation: Escalation, actor_scope: DataScope) -> Result<(), DomainError>;
}

#[async_trait]
pub trait TechnicianRepository: Send + Sync {
    async fn save(&self, technician: Technician, actor_scope: DataScope) -> Result<(), DomainError>;
    async fn get_by_id(&self, id: &str, scope: DataScope) -> Result<Option<Technician>, DomainError>;
    async fn list(&self, scope: DataScope) -> Result<Vec<Technician>, DomainError>;
}

#[async_trait]
pub trait AuditRepository: Send + Sync {
    /// Запись аудита доверена application-слою; `owner_user_id` в записи — владелец сущности для фильтрации чтения.
    async fn save(&self, record: AuditRecord) -> Result<(), DomainError>;
    async fn list(&self, scope: DataScope) -> Result<Vec<AuditRecord>, DomainError>;
    async fn list_by_request(
        &self,
        request_id: &str,
        scope: DataScope,
    ) -> Result<Vec<AuditRecord>, DomainError>;
}

#[async_trait]
pub trait AnalyticsQueryPort: Send + Sync {
    async fn dashboard_summary(
        &self,
        now_epoch: u64,
        scope: DataScope,
    ) -> Result<DashboardSummary, DomainError>;
    async fn sla_compliance_summary(
        &self,
        now_epoch: u64,
        scope: DataScope,
    ) -> Result<SlaComplianceSummary, DomainError>;
    async fn sla_compliance_by_priority_summary(
        &self,
        now_epoch: u64,
        scope: DataScope,
    ) -> Result<Vec<SlaComplianceByPriorityItem>, DomainError>;
    async fn technician_workload_summary(
        &self,
        scope: DataScope,
    ) -> Result<Vec<TechnicianWorkloadSummary>, DomainError>;

    async fn list_requests(&self, scope: DataScope) -> Result<Vec<ServiceRequest>, DomainError>;
    async fn list_work_orders(&self, scope: DataScope) -> Result<Vec<WorkOrder>, DomainError>;
    async fn list_escalations(&self, scope: DataScope) -> Result<Vec<Escalation>, DomainError>;
    async fn list_technicians(&self, scope: DataScope) -> Result<Vec<Technician>, DomainError>;
}

#[async_trait]
pub trait AnalyticsSnapshotRepository: Send + Sync {
    async fn get_latest(&self) -> Result<Option<AnalyticsSnapshot>, DomainError>;
    async fn upsert(&self, snapshot: AnalyticsSnapshot) -> Result<(), DomainError>;
}
