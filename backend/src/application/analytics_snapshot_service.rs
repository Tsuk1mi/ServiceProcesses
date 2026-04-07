use std::sync::Arc;

use crate::domain::analytics::{AnalyticsSnapshot, DashboardSummary};
use crate::domain::errors::DomainError;
use crate::ports::data_scope::DataScope;
use crate::ports::outbound::{AnalyticsQueryPort, AnalyticsSnapshotRepository};

#[derive(Clone)]
pub struct AnalyticsSnapshotAppService {
    pub analytics: Arc<dyn AnalyticsQueryPort>,
    pub snapshots: Arc<dyn AnalyticsSnapshotRepository>,
}

impl AnalyticsSnapshotAppService {
    /// Кэш дашборда только для глобального (admin) представления.
    pub async fn refresh(&self, now_epoch: u64) -> Result<(), DomainError> {
        let snapshot = self.compute_snapshot(now_epoch, DataScope::All).await?;
        self.snapshots.upsert(snapshot).await?;
        Ok(())
    }

    async fn compute_snapshot(&self, now_epoch: u64, scope: DataScope) -> Result<AnalyticsSnapshot, DomainError> {
        let dashboard = self.analytics.dashboard_summary(now_epoch, scope.clone()).await?;
        let sla_compliance = self.analytics.sla_compliance_summary(now_epoch, scope.clone()).await?;
        let sla_compliance_by_priority = self
            .analytics
            .sla_compliance_by_priority_summary(now_epoch, scope.clone())
            .await?;
        let technician_workload = self.analytics.technician_workload_summary(scope).await?;

        Ok(AnalyticsSnapshot {
            updated_at_epoch_sec: now_epoch,
            dashboard,
            sla_compliance,
            sla_compliance_by_priority,
            technician_workload,
        })
    }

    pub async fn get_dashboard_summary(
        &self,
        now_epoch: u64,
        scope: DataScope,
    ) -> Result<DashboardSummary, DomainError> {
        if scope.is_all() {
            if let Some(latest) = self.snapshots.get_latest().await? {
                return Ok(latest.dashboard);
            }
            let snapshot = self.compute_snapshot(now_epoch, DataScope::All).await?;
            self.snapshots.upsert(snapshot.clone()).await?;
            return Ok(snapshot.dashboard);
        }
        let snapshot = self.compute_snapshot(now_epoch, scope).await?;
        Ok(snapshot.dashboard)
    }

    pub async fn get_sla_compliance_summary(
        &self,
        now_epoch: u64,
        scope: DataScope,
    ) -> Result<crate::domain::analytics::SlaComplianceSummary, DomainError> {
        if scope.is_all() {
            if let Some(latest) = self.snapshots.get_latest().await? {
                return Ok(latest.sla_compliance);
            }
            let snapshot = self.compute_snapshot(now_epoch, DataScope::All).await?;
            self.snapshots.upsert(snapshot.clone()).await?;
            return Ok(snapshot.sla_compliance);
        }
        let snapshot = self.compute_snapshot(now_epoch, scope).await?;
        Ok(snapshot.sla_compliance)
    }

    pub async fn get_sla_compliance_by_priority_summary(
        &self,
        now_epoch: u64,
        scope: DataScope,
    ) -> Result<Vec<crate::domain::analytics::SlaComplianceByPriorityItem>, DomainError> {
        if scope.is_all() {
            if let Some(latest) = self.snapshots.get_latest().await? {
                return Ok(latest.sla_compliance_by_priority);
            }
            let snapshot = self.compute_snapshot(now_epoch, DataScope::All).await?;
            self.snapshots.upsert(snapshot.clone()).await?;
            return Ok(snapshot.sla_compliance_by_priority);
        }
        let snapshot = self.compute_snapshot(now_epoch, scope).await?;
        Ok(snapshot.sla_compliance_by_priority)
    }

    pub async fn get_technician_workload_summary(
        &self,
        now_epoch: u64,
        scope: DataScope,
    ) -> Result<Vec<crate::domain::analytics::TechnicianWorkloadSummary>, DomainError> {
        if scope.is_all() {
            if let Some(latest) = self.snapshots.get_latest().await? {
                return Ok(latest.technician_workload);
            }
            let snapshot = self.compute_snapshot(now_epoch, DataScope::All).await?;
            self.snapshots.upsert(snapshot.clone()).await?;
            return Ok(snapshot.technician_workload);
        }
        let snapshot = self.compute_snapshot(now_epoch, scope).await?;
        Ok(snapshot.technician_workload)
    }
}

#[cfg(test)]
mod tests {
    use super::AnalyticsSnapshotAppService;
    use crate::auth::users::InMemoryUserStore;
    use crate::domain::entities::{ServiceRequest, Technician, WorkOrder};
    use crate::domain::errors::DomainError;
    use crate::domain::value_objects::{Priority, RequestStatus, WorkOrderStatus};
    use crate::infrastructure::in_memory::{
        InMemoryAnalyticsQuery, InMemoryAnalyticsSnapshotRepository, InMemoryAssetRepository,
        InMemoryEscalationRepository, InMemoryRequestRepository, InMemoryTechnicianRepository,
        InMemoryWorkOrderRepository,
    };
    use crate::ports::data_scope::DataScope;
    use crate::ports::outbound::{
        AnalyticsSnapshotRepository, EscalationRepository, ServiceRequestRepository, TechnicianRepository,
        WorkOrderRepository,
    };

    #[tokio::test]
    async fn refresh_populates_cache() -> Result<(), DomainError> {
        let assets = std::sync::Arc::new(InMemoryAssetRepository::new());
        let requests = InMemoryRequestRepository::new(assets);
        let work_orders = InMemoryWorkOrderRepository::new();
        let escalations = InMemoryEscalationRepository::new();
        let technicians = InMemoryTechnicianRepository::new();
        let snapshots: std::sync::Arc<dyn AnalyticsSnapshotRepository> =
            std::sync::Arc::new(InMemoryAnalyticsSnapshotRepository::new());
        let owner = InMemoryUserStore::demo_admin_id().to_string();

        technicians
            .save(
                Technician::new(
                    "tech-1".to_string(),
                    "Ivan".to_string(),
                    vec!["electrical".to_string()],
                    owner.clone(),
                )
                .unwrap(),
            )
            .await?;

        let mut req = ServiceRequest::new(
            "req-1".to_string(),
            "asset-1".to_string(),
            "desc".to_string(),
            Priority::High,
            60,
            owner.clone(),
        )
        .unwrap();
        req.status = RequestStatus::InProgress;
        req.created_at_epoch_sec = 0;
        requests.save(req).await?;

        let mut wo = WorkOrder::new("wo-1".to_string(), "req-1".to_string(), owner.clone()).unwrap();
        wo.assignee = Some("tech-1".to_string());
        wo.status = WorkOrderStatus::InProgress;
        work_orders.save(wo).await?;

        let analytics_query = InMemoryAnalyticsQuery {
            requests: std::sync::Arc::new(requests.clone()) as std::sync::Arc<dyn ServiceRequestRepository>,
            work_orders: std::sync::Arc::new(work_orders.clone()) as std::sync::Arc<dyn WorkOrderRepository>,
            escalations: std::sync::Arc::new(escalations.clone()) as std::sync::Arc<dyn EscalationRepository>,
            technicians: std::sync::Arc::new(technicians.clone()) as std::sync::Arc<dyn TechnicianRepository>,
        };

        let svc = AnalyticsSnapshotAppService {
            analytics: std::sync::Arc::new(analytics_query),
            snapshots,
        };

        svc.refresh(61).await?;
        let cached = svc.get_dashboard_summary(61, DataScope::All).await?;
        assert_eq!(cached.total_requests, 1);
        assert_eq!(cached.in_progress_requests, 1);
        Ok(())
    }
}
