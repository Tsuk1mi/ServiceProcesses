use crate::domain::analytics::{AnalyticsSnapshot, DashboardSummary};
use crate::domain::errors::DomainError;
use crate::ports::outbound::{AnalyticsQueryPort, AnalyticsSnapshotRepository};

#[derive(Clone)]
pub struct AnalyticsSnapshotAppService<Q, S>
where
    Q: AnalyticsQueryPort,
    S: AnalyticsSnapshotRepository,
{
    pub analytics: Q,
    pub snapshots: S,
}

impl<Q, S> AnalyticsSnapshotAppService<Q, S>
where
    Q: AnalyticsQueryPort,
    S: AnalyticsSnapshotRepository,
{
    pub fn refresh(&self, now_epoch: u64) -> Result<(), DomainError> {
        let snapshot = self.compute_snapshot(now_epoch)?;
        self.snapshots.upsert(snapshot)?;
        Ok(())
    }

    fn compute_snapshot(&self, now_epoch: u64) -> Result<AnalyticsSnapshot, DomainError> {
        let dashboard = self.analytics.dashboard_summary(now_epoch)?;
        let sla_compliance = self.analytics.sla_compliance_summary(now_epoch)?;
        let sla_compliance_by_priority =
            self.analytics.sla_compliance_by_priority_summary(now_epoch)?;
        let technician_workload = self.analytics.technician_workload_summary()?;

        Ok(AnalyticsSnapshot {
            updated_at_epoch_sec: now_epoch,
            dashboard,
            sla_compliance,
            sla_compliance_by_priority,
            technician_workload,
        })
    }

    pub fn get_dashboard_summary(&self, now_epoch: u64) -> Result<DashboardSummary, DomainError> {
        if let Some(latest) = self.snapshots.get_latest()? {
            return Ok(latest.dashboard);
        }
        let snapshot = self.compute_snapshot(now_epoch)?;
        // Best-effort cache population; ignore cache write errors only if needed later.
        self.snapshots.upsert(snapshot.clone())?;
        Ok(snapshot.dashboard)
    }

    pub fn get_sla_compliance_summary(
        &self,
        now_epoch: u64,
    ) -> Result<crate::domain::analytics::SlaComplianceSummary, DomainError> {
        if let Some(latest) = self.snapshots.get_latest()? {
            return Ok(latest.sla_compliance);
        }
        let snapshot = self.compute_snapshot(now_epoch)?;
        self.snapshots.upsert(snapshot.clone())?;
        Ok(snapshot.sla_compliance)
    }

    pub fn get_sla_compliance_by_priority_summary(
        &self,
        now_epoch: u64,
    ) -> Result<Vec<crate::domain::analytics::SlaComplianceByPriorityItem>, DomainError> {
        if let Some(latest) = self.snapshots.get_latest()? {
            return Ok(latest.sla_compliance_by_priority);
        }
        let snapshot = self.compute_snapshot(now_epoch)?;
        self.snapshots.upsert(snapshot.clone())?;
        Ok(snapshot.sla_compliance_by_priority)
    }

    pub fn get_technician_workload_summary(
        &self,
        now_epoch: u64,
    ) -> Result<Vec<crate::domain::analytics::TechnicianWorkloadSummary>, DomainError> {
        if let Some(latest) = self.snapshots.get_latest()? {
            return Ok(latest.technician_workload);
        }
        let snapshot = self.compute_snapshot(now_epoch)?;
        self.snapshots.upsert(snapshot.clone())?;
        Ok(snapshot.technician_workload)
    }
}

#[cfg(test)]
mod tests {
    use super::AnalyticsSnapshotAppService;
    use crate::domain::entities::{ServiceRequest, Technician, WorkOrder};
    use crate::domain::value_objects::{Priority, RequestStatus, WorkOrderStatus};
    use crate::infrastructure::in_memory::{
        InMemoryAnalyticsQuery, InMemoryAnalyticsSnapshotRepository, InMemoryRequestRepository,
        InMemoryTechnicianRepository, InMemoryWorkOrderRepository, InMemoryEscalationRepository,
    };
    use crate::domain::errors::DomainError;
    use crate::ports::outbound::{ServiceRequestRepository, TechnicianRepository, WorkOrderRepository};

    #[test]
    fn refresh_populates_cache() -> Result<(), DomainError> {
        let requests = InMemoryRequestRepository::new();
        let work_orders = InMemoryWorkOrderRepository::new();
        let escalations = InMemoryEscalationRepository::new();
        let technicians = InMemoryTechnicianRepository::new();
        let snapshots = InMemoryAnalyticsSnapshotRepository::new();

        technicians.save(
            Technician::new(
                "tech-1".to_string(),
                "Ivan".to_string(),
                vec!["electrical".to_string()],
            )
            .unwrap(),
        )?;

        let mut req = ServiceRequest::new(
            "req-1".to_string(),
            "asset-1".to_string(),
            "desc".to_string(),
            Priority::High,
            60,
        )
        .unwrap();
        req.status = RequestStatus::InProgress;
        req.created_at_epoch_sec = 0;
        requests.save(req)?;

        let mut wo = WorkOrder::new("wo-1".to_string(), "req-1".to_string()).unwrap();
        wo.assignee = Some("tech-1".to_string());
        wo.status = WorkOrderStatus::InProgress;
        work_orders.save(wo)?;

        let analytics_query = InMemoryAnalyticsQuery {
            requests: requests.clone(),
            work_orders: work_orders.clone(),
            escalations: escalations.clone(),
            technicians: technicians.clone(),
        };

        let svc = AnalyticsSnapshotAppService {
            analytics: analytics_query,
            snapshots,
        };

        svc.refresh(61)?;
        let cached = svc.get_dashboard_summary(61)?;
        assert_eq!(cached.total_requests, 1);
        assert_eq!(cached.in_progress_requests, 1);
        Ok(())
    }
}

