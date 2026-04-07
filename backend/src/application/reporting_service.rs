use crate::domain::analytics::{
    DashboardSummary, SlaComplianceByPriorityItem, SlaComplianceSummary,
    TechnicianWorkloadSummary,
};
use crate::domain::errors::DomainError;
use crate::ports::data_scope::DataScope;
use crate::ports::outbound::AnalyticsQueryPort;

#[derive(Clone)]
pub struct ReportingAppService<A>
where
    A: AnalyticsQueryPort,
{
    pub analytics: A,
}

impl<A> ReportingAppService<A>
where
    A: AnalyticsQueryPort + Send + Sync,
{
    pub async fn dashboard_summary(
        &self,
        now_epoch: u64,
        scope: DataScope,
    ) -> Result<DashboardSummary, DomainError> {
        self.analytics.dashboard_summary(now_epoch, scope).await
    }

    pub async fn technician_workload_summary(
        &self,
        scope: DataScope,
    ) -> Result<Vec<TechnicianWorkloadSummary>, DomainError> {
        self.analytics.technician_workload_summary(scope).await
    }

    pub async fn sla_compliance_summary(
        &self,
        now_epoch: u64,
        scope: DataScope,
    ) -> Result<SlaComplianceSummary, DomainError> {
        self.analytics.sla_compliance_summary(now_epoch, scope).await
    }

    pub async fn sla_compliance_by_priority_summary(
        &self,
        now_epoch: u64,
        scope: DataScope,
    ) -> Result<Vec<SlaComplianceByPriorityItem>, DomainError> {
        self.analytics
            .sla_compliance_by_priority_summary(now_epoch, scope)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::ReportingAppService;
    use crate::domain::entities::{Escalation, ServiceRequest, Technician, WorkOrder};
    use crate::domain::value_objects::{EscalationState, Priority, RequestStatus, WorkOrderStatus};
    use crate::infrastructure::in_memory::{
        InMemoryAnalyticsQuery, InMemoryAssetRepository, InMemoryEscalationRepository,
        InMemoryRequestRepository, InMemoryTechnicianRepository, InMemoryWorkOrderRepository,
    };
    use crate::ports::data_scope::DataScope;
    use crate::ports::outbound::{
        EscalationRepository, ServiceRequestRepository, TechnicianRepository, WorkOrderRepository,
    };
    use crate::auth::users::InMemoryUserStore;

    async fn service() -> (
        ReportingAppService<InMemoryAnalyticsQuery>,
        InMemoryRequestRepository,
        InMemoryWorkOrderRepository,
        InMemoryEscalationRepository,
        InMemoryTechnicianRepository,
    ) {
        let assets = std::sync::Arc::new(InMemoryAssetRepository::new());
        let requests = InMemoryRequestRepository::new(assets.clone());
        let work_orders = InMemoryWorkOrderRepository::new();
        let escalations = InMemoryEscalationRepository::new();
        let technicians = InMemoryTechnicianRepository::new();
        let service = ReportingAppService {
            analytics: InMemoryAnalyticsQuery {
                requests: requests.clone(),
                work_orders: work_orders.clone(),
                escalations: escalations.clone(),
                technicians: technicians.clone(),
            },
        };
        (service, requests, work_orders, escalations, technicians)
    }

    fn scope_all() -> DataScope {
        DataScope::All
    }

    #[tokio::test]
    async fn dashboard_summary_counts_core_metrics() {
        let (svc, requests, work_orders, escalations, _technicians) = service().await;
        let owner = InMemoryUserStore::demo_admin_id().to_string();

        let mut req_new = ServiceRequest::new(
            "req-new".to_string(),
            "asset-1".to_string(),
            "desc".to_string(),
            Priority::High,
            60,
            owner.clone(),
        )
        .expect("request");
        req_new.created_at_epoch_sec = 0;
        requests.save(req_new).await.expect("save request");

        let mut req_progress = ServiceRequest::new(
            "req-progress".to_string(),
            "asset-1".to_string(),
            "desc".to_string(),
            Priority::Medium,
            60,
            owner.clone(),
        )
        .expect("request");
        req_progress.status = RequestStatus::InProgress;
        requests.save(req_progress).await.expect("save request");

        let mut req_closed = ServiceRequest::new(
            "req-closed".to_string(),
            "asset-1".to_string(),
            "desc".to_string(),
            Priority::Low,
            60,
            owner.clone(),
        )
        .expect("request");
        req_closed.status = RequestStatus::Closed;
        requests.save(req_closed).await.expect("save request");

        let mut wo = WorkOrder::new("wo-1".to_string(), "req-progress".to_string(), owner.clone()).expect("wo");
        wo.assignee = Some("tech-1".to_string());
        wo.status = WorkOrderStatus::InProgress;
        work_orders.save(wo).await.expect("save wo");

        let mut esc = Escalation::new(
            "esc-1".to_string(),
            "req-new".to_string(),
            "reason".to_string(),
            owner.clone(),
        )
        .expect("esc");
        esc.state = EscalationState::Open;
        escalations.save(esc).await.expect("save esc");

        let summary = svc.dashboard_summary(3601, scope_all()).await.expect("summary");
        assert_eq!(summary.total_requests, 3);
        assert_eq!(summary.open_requests, 2);
        assert_eq!(summary.in_progress_requests, 1);
        assert_eq!(summary.closed_requests, 1);
        assert_eq!(summary.overdue_requests, 1);
        assert_eq!(summary.total_work_orders, 1);
        assert_eq!(summary.active_work_orders, 1);
        assert_eq!(summary.open_escalations, 1);
    }

    #[tokio::test]
    async fn technician_workload_summary_aggregates_by_assignee() {
        let (svc, _requests, work_orders, _escalations, technicians) = service().await;
        let owner = InMemoryUserStore::demo_admin_id().to_string();

        technicians
            .save(
                Technician::new(
                    "tech-1".to_string(),
                    "Ivan".to_string(),
                    vec!["electrical".to_string()],
                    owner.clone(),
                )
                .expect("tech"),
            )
            .await
            .expect("save tech");
        technicians
            .save(
                Technician::new(
                    "tech-2".to_string(),
                    "Petr".to_string(),
                    vec!["hvac".to_string()],
                    owner.clone(),
                )
                .expect("tech"),
            )
            .await
            .expect("save tech");

        let mut w1 = WorkOrder::new("wo-1".to_string(), "req-1".to_string(), owner.clone()).expect("wo");
        w1.assignee = Some("tech-1".to_string());
        w1.status = WorkOrderStatus::Assigned;
        work_orders.save(w1).await.expect("save");

        let mut w2 = WorkOrder::new("wo-2".to_string(), "req-2".to_string(), owner.clone()).expect("wo");
        w2.assignee = Some("tech-1".to_string());
        w2.status = WorkOrderStatus::Completed;
        work_orders.save(w2).await.expect("save");

        let mut w3 = WorkOrder::new("wo-3".to_string(), "req-3".to_string(), owner.clone()).expect("wo");
        w3.assignee = Some("tech-2".to_string());
        w3.status = WorkOrderStatus::InProgress;
        work_orders.save(w3).await.expect("save");

        let items = svc
            .technician_workload_summary(scope_all())
            .await
            .expect("workload summary");
        assert_eq!(items.len(), 2);

        let first = &items[0];
        assert_eq!(first.technician_id, "tech-1");
        assert_eq!(first.assigned, 1);
        assert_eq!(first.in_progress, 0);
        assert_eq!(first.completed, 1);
        assert_eq!(first.total, 2);
    }

    #[tokio::test]
    async fn sla_compliance_summary_counts_only_open_requests() {
        let (svc, requests, _work_orders, _escalations, _technicians) = service().await;
        let owner = InMemoryUserStore::demo_admin_id().to_string();

        let mut open_overdue = ServiceRequest::new(
            "req-open-overdue".to_string(),
            "asset-1".to_string(),
            "desc".to_string(),
            Priority::High,
            60,
            owner.clone(),
        )
        .expect("request");
        open_overdue.created_at_epoch_sec = 0;
        requests.save(open_overdue).await.expect("save request");

        let mut open_ok = ServiceRequest::new(
            "req-open-ok".to_string(),
            "asset-1".to_string(),
            "desc".to_string(),
            Priority::Medium,
            60,
            owner.clone(),
        )
        .expect("request");
        open_ok.created_at_epoch_sec = 3_590;
        open_ok.status = RequestStatus::InProgress;
        requests.save(open_ok).await.expect("save request");

        let mut closed_overdue = ServiceRequest::new(
            "req-closed-overdue".to_string(),
            "asset-1".to_string(),
            "desc".to_string(),
            Priority::Low,
            60,
            owner.clone(),
        )
        .expect("request");
        closed_overdue.created_at_epoch_sec = 0;
        closed_overdue.status = RequestStatus::Closed;
        requests.save(closed_overdue).await.expect("save request");

        let summary = svc.sla_compliance_summary(3_601, scope_all()).await.expect("sla summary");
        assert_eq!(summary.total_open_requests, 2);
        assert_eq!(summary.overdue_open_requests, 1);
        assert_eq!(summary.compliant_open_requests, 1);
        assert!((summary.compliance_percent - 50.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn sla_compliance_by_priority_groups_overdue_and_compliance() {
        let (svc, requests, _work_orders, _escalations, _technicians) = service().await;
        let owner = InMemoryUserStore::demo_admin_id().to_string();

        let mut req_high = ServiceRequest::new(
            "req-high".to_string(),
            "asset-1".to_string(),
            "desc".to_string(),
            Priority::High,
            60,
            owner.clone(),
        )
        .expect("request");
        req_high.created_at_epoch_sec = 0;
        requests.save(req_high).await.expect("save request");

        let mut req_low = ServiceRequest::new(
            "req-low".to_string(),
            "asset-1".to_string(),
            "desc".to_string(),
            Priority::Low,
            60,
            owner.clone(),
        )
        .expect("request");
        req_low.created_at_epoch_sec = 30;
        req_low.status = RequestStatus::Planned;
        requests.save(req_low).await.expect("save request");

        let items = svc
            .sla_compliance_by_priority_summary(3_601, scope_all())
            .await
            .expect("sla compliance by priority");

        assert_eq!(items.len(), 2);

        let high = items.iter().find(|x| x.priority == "High").expect("high item");
        assert_eq!(high.total_open_requests, 1);
        assert_eq!(high.overdue_open_requests, 1);
        assert_eq!(high.compliant_open_requests, 0);

        let low = items.iter().find(|x| x.priority == "Low").expect("low item");
        assert_eq!(low.total_open_requests, 1);
        assert_eq!(low.overdue_open_requests, 0);
        assert_eq!(low.compliant_open_requests, 1);
    }
}
