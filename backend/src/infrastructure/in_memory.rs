use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::domain::analytics::{
    AnalyticsSnapshot, DashboardSummary, SlaComplianceByPriorityItem, SlaComplianceSummary,
    TechnicianWorkloadSummary,
};
use crate::domain::entities::{Asset, AuditRecord, Escalation, ServiceRequest, Technician, WorkOrder};
use crate::domain::errors::DomainError;
use crate::domain::value_objects::{EscalationState, Priority, RequestStatus, WorkOrderStatus};
use crate::ports::data_scope::DataScope;
use crate::ports::outbound::{
    AnalyticsQueryPort, AnalyticsSnapshotRepository, AssetRepository, AuditRepository,
    EscalationRepository, EventPublisherPort, PriorityPolicyPort, ServiceRequestRepository,
    SlaPolicyPort, TechnicianRepository, WorkOrderRepository,
};

fn scope_matches(owner_user_id: &str, scope: &DataScope) -> bool {
    match scope {
        DataScope::All => true,
        DataScope::Owner(u) => owner_user_id == u.to_string(),
    }
}

fn enforce_entity_save(
    existing_owner: Option<&str>,
    new_owner: &str,
    actor_scope: &DataScope,
) -> Result<(), DomainError> {
    match actor_scope {
        DataScope::All => Ok(()),
        DataScope::Owner(u) => {
            let sid = u.to_string();
            if new_owner != sid {
                return Err(DomainError::Forbidden(
                    "entity owner must match current user for non-admin",
                ));
            }
            if let Some(ex) = existing_owner {
                if ex != sid {
                    return Err(DomainError::Forbidden("cannot modify another user's resource"));
                }
            }
            Ok(())
        }
    }
}

#[derive(Clone)]
pub struct InMemoryAssetRepository {
    data: Arc<Mutex<HashMap<String, Asset>>>,
}

impl InMemoryAssetRepository {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl AssetRepository for InMemoryAssetRepository {
    async fn save(&self, asset: Asset, actor_scope: DataScope) -> Result<(), DomainError> {
        let mut g = self.data.lock().await;
        let ex_owner = g.get(&asset.id).map(|a| a.owner_user_id.as_str());
        enforce_entity_save(ex_owner, &asset.owner_user_id, &actor_scope)?;
        g.insert(asset.id.clone(), asset);
        Ok(())
    }

    async fn get_by_id(&self, id: &str, scope: DataScope) -> Result<Option<Asset>, DomainError> {
        let g = self.data.lock().await;
        let a = g.get(id).cloned();
        Ok(a.filter(|x| scope_matches(&x.owner_user_id, &scope)))
    }

    async fn list(&self, scope: DataScope) -> Result<Vec<Asset>, DomainError> {
        let g = self.data.lock().await;
        Ok(g.values()
            .filter(|a| scope_matches(&a.owner_user_id, &scope))
            .cloned()
            .collect())
    }
}

#[derive(Clone)]
pub struct InMemoryRequestRepository {
    data: Arc<Mutex<HashMap<String, ServiceRequest>>>,
    assets: Arc<InMemoryAssetRepository>,
}

impl InMemoryRequestRepository {
    pub fn new(assets: Arc<InMemoryAssetRepository>) -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
            assets,
        }
    }
}

#[async_trait]
impl ServiceRequestRepository for InMemoryRequestRepository {
    async fn save(&self, request: ServiceRequest, actor_scope: DataScope) -> Result<(), DomainError> {
        let mut g = self.data.lock().await;
        let ex_owner = g.get(&request.id).map(|r| r.owner_user_id.as_str());
        enforce_entity_save(ex_owner, &request.owner_user_id, &actor_scope)?;
        g.insert(request.id.clone(), request);
        Ok(())
    }

    async fn get_by_id(&self, id: &str, scope: DataScope) -> Result<Option<ServiceRequest>, DomainError> {
        let g = self.data.lock().await;
        let r = g.get(id).cloned();
        Ok(r.filter(|x| scope_matches(&x.owner_user_id, &scope)))
    }

    async fn list(&self, scope: DataScope) -> Result<Vec<ServiceRequest>, DomainError> {
        let g = self.data.lock().await;
        Ok(g.values()
            .filter(|r| scope_matches(&r.owner_user_id, &scope))
            .cloned()
            .collect())
    }

    async fn update(&self, request: ServiceRequest, actor_scope: DataScope) -> Result<(), DomainError> {
        self.save(request, actor_scope).await
    }

    async fn list_with_assets(
        &self,
        scope: DataScope,
    ) -> Result<Vec<(ServiceRequest, Option<Asset>)>, DomainError> {
        let reqs = self.list(scope.clone()).await?;
        let mut out = Vec::with_capacity(reqs.len());
        for r in reqs {
            let asset = self.assets.get_by_id(&r.asset_id, scope.clone()).await?;
            out.push((r, asset));
        }
        Ok(out)
    }
}

#[derive(Clone)]
pub struct InMemoryWorkOrderRepository {
    data: Arc<Mutex<HashMap<String, WorkOrder>>>,
}

impl InMemoryWorkOrderRepository {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl WorkOrderRepository for InMemoryWorkOrderRepository {
    async fn save(&self, work_order: WorkOrder, actor_scope: DataScope) -> Result<(), DomainError> {
        let mut g = self.data.lock().await;
        let ex_owner = g.get(&work_order.id).map(|w| w.owner_user_id.as_str());
        enforce_entity_save(ex_owner, &work_order.owner_user_id, &actor_scope)?;
        g.insert(work_order.id.clone(), work_order);
        Ok(())
    }

    async fn get_by_id(&self, id: &str, scope: DataScope) -> Result<Option<WorkOrder>, DomainError> {
        let g = self.data.lock().await;
        let w = g.get(id).cloned();
        Ok(w.filter(|x| scope_matches(&x.owner_user_id, &scope)))
    }

    async fn list(&self, scope: DataScope) -> Result<Vec<WorkOrder>, DomainError> {
        let g = self.data.lock().await;
        Ok(g.values()
            .filter(|w| scope_matches(&w.owner_user_id, &scope))
            .cloned()
            .collect())
    }

    async fn list_by_request(
        &self,
        request_id: &str,
        scope: DataScope,
    ) -> Result<Vec<WorkOrder>, DomainError> {
        let g = self.data.lock().await;
        Ok(g.values()
            .filter(|wo| wo.request_id == request_id && scope_matches(&wo.owner_user_id, &scope))
            .cloned()
            .collect())
    }

    async fn update(&self, work_order: WorkOrder, actor_scope: DataScope) -> Result<(), DomainError> {
        self.save(work_order, actor_scope).await
    }
}

#[derive(Clone)]
pub struct InMemoryEscalationRepository {
    data: Arc<Mutex<HashMap<String, Escalation>>>,
}

impl InMemoryEscalationRepository {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl EscalationRepository for InMemoryEscalationRepository {
    async fn save(&self, escalation: Escalation, actor_scope: DataScope) -> Result<(), DomainError> {
        let mut g = self.data.lock().await;
        let ex_owner = g.get(&escalation.id).map(|e| e.owner_user_id.as_str());
        enforce_entity_save(ex_owner, &escalation.owner_user_id, &actor_scope)?;
        g.insert(escalation.id.clone(), escalation);
        Ok(())
    }

    async fn get_by_id(&self, id: &str, scope: DataScope) -> Result<Option<Escalation>, DomainError> {
        let g = self.data.lock().await;
        let e = g.get(id).cloned();
        Ok(e.filter(|x| scope_matches(&x.owner_user_id, &scope)))
    }

    async fn list(&self, scope: DataScope) -> Result<Vec<Escalation>, DomainError> {
        let g = self.data.lock().await;
        Ok(g.values()
            .filter(|e| scope_matches(&e.owner_user_id, &scope))
            .cloned()
            .collect())
    }

    async fn list_by_request(
        &self,
        request_id: &str,
        scope: DataScope,
    ) -> Result<Vec<Escalation>, DomainError> {
        let g = self.data.lock().await;
        Ok(g.values()
            .filter(|e| e.request_id == request_id && scope_matches(&e.owner_user_id, &scope))
            .cloned()
            .collect())
    }

    async fn update(&self, escalation: Escalation, actor_scope: DataScope) -> Result<(), DomainError> {
        self.save(escalation, actor_scope).await
    }
}

#[derive(Clone)]
pub struct InMemoryTechnicianRepository {
    data: Arc<Mutex<HashMap<String, Technician>>>,
}

impl InMemoryTechnicianRepository {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl TechnicianRepository for InMemoryTechnicianRepository {
    async fn save(&self, technician: Technician, actor_scope: DataScope) -> Result<(), DomainError> {
        let mut g = self.data.lock().await;
        let ex_owner = g.get(&technician.id).map(|t| t.owner_user_id.as_str());
        enforce_entity_save(ex_owner, &technician.owner_user_id, &actor_scope)?;
        g.insert(technician.id.clone(), technician);
        Ok(())
    }

    async fn get_by_id(&self, id: &str, scope: DataScope) -> Result<Option<Technician>, DomainError> {
        let g = self.data.lock().await;
        let t = g.get(id).cloned();
        Ok(t.filter(|x| scope_matches(&x.owner_user_id, &scope)))
    }

    async fn list(&self, scope: DataScope) -> Result<Vec<Technician>, DomainError> {
        let g = self.data.lock().await;
        Ok(g.values()
            .filter(|t| scope_matches(&t.owner_user_id, &scope))
            .cloned()
            .collect())
    }
}

#[derive(Clone)]
pub struct InMemoryAuditRepository {
    data: Arc<Mutex<Vec<AuditRecord>>>,
}

impl InMemoryAuditRepository {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl AuditRepository for InMemoryAuditRepository {
    async fn save(&self, record: AuditRecord) -> Result<(), DomainError> {
        self.data.lock().await.push(record);
        Ok(())
    }

    async fn list(&self, scope: DataScope) -> Result<Vec<AuditRecord>, DomainError> {
        let g = self.data.lock().await;
        Ok(g.iter()
            .filter(|r| scope_matches(&r.owner_user_id, &scope))
            .cloned()
            .collect())
    }

    async fn list_by_request(
        &self,
        request_id: &str,
        scope: DataScope,
    ) -> Result<Vec<AuditRecord>, DomainError> {
        let g = self.data.lock().await;
        Ok(g.iter()
            .filter(|r| {
                r.request_id.as_deref() == Some(request_id) && scope_matches(&r.owner_user_id, &scope)
            })
            .cloned()
            .collect())
    }
}

#[derive(Clone, Copy)]
pub struct BasicSlaPolicy;

impl SlaPolicyPort for BasicSlaPolicy {
    fn resolve_sla_minutes(&self, priority: Priority) -> Result<u32, DomainError> {
        let minutes = match priority {
            Priority::Low => 24 * 60,
            Priority::Medium => 8 * 60,
            Priority::High => 4 * 60,
            Priority::Critical => 60,
        };
        Ok(minutes)
    }
}

#[derive(Clone, Copy)]
pub struct KeywordPriorityPolicy;

impl PriorityPolicyPort for KeywordPriorityPolicy {
    fn resolve_priority(&self, description: &str) -> Result<Priority, DomainError> {
        let lower = description.to_lowercase();
        if lower.contains("авар") || lower.contains("critical") {
            return Ok(Priority::Critical);
        }
        if lower.contains("срочно") || lower.contains("urgent") {
            return Ok(Priority::High);
        }
        if lower.contains("план") {
            return Ok(Priority::Low);
        }
        Ok(Priority::Medium)
    }
}

#[derive(Clone, Copy)]
pub struct StdoutEventPublisher;

#[async_trait]
impl EventPublisherPort for StdoutEventPublisher {
    async fn publish(&self, topic: &str, payload: &str) -> Result<(), DomainError> {
        tracing::info!(topic = %topic, "event published");
        println!("[event] topic={topic}; payload={payload}");
        Ok(())
    }
}

#[derive(Clone)]
pub struct InMemoryAnalyticsQuery {
    pub requests: Arc<dyn ServiceRequestRepository>,
    pub work_orders: Arc<dyn WorkOrderRepository>,
    pub escalations: Arc<dyn EscalationRepository>,
    pub technicians: Arc<dyn TechnicianRepository>,
}

#[async_trait]
impl AnalyticsQueryPort for InMemoryAnalyticsQuery {
    async fn dashboard_summary(
        &self,
        now_epoch: u64,
        scope: DataScope,
    ) -> Result<DashboardSummary, DomainError> {
        let requests = self.requests.list(scope.clone()).await?;
        let work_orders = self.work_orders.list(scope.clone()).await?;
        let escalations = self.escalations.list(scope).await?;

        let total_requests = requests.len();
        let open_requests = requests.iter().filter(|r| !r.is_terminal()).count();
        let in_progress_requests = requests
            .iter()
            .filter(|r| r.status == RequestStatus::InProgress)
            .count();
        let resolved_requests = requests
            .iter()
            .filter(|r| r.status == RequestStatus::Resolved)
            .count();
        let closed_requests = requests
            .iter()
            .filter(|r| r.status == RequestStatus::Closed)
            .count();
        let overdue_requests = requests
            .iter()
            .filter(|r| !r.is_terminal())
            .filter(|r| now_epoch > r.created_at_epoch_sec + (r.sla_minutes as u64) * 60)
            .count();

        let total_work_orders = work_orders.len();
        let active_work_orders = work_orders
            .iter()
            .filter(|o| o.status != WorkOrderStatus::Completed && o.status != WorkOrderStatus::Cancelled)
            .count();
        let open_escalations = escalations
            .iter()
            .filter(|e| e.state == EscalationState::Open)
            .count();

        Ok(DashboardSummary {
            total_requests,
            open_requests,
            in_progress_requests,
            resolved_requests,
            closed_requests,
            overdue_requests,
            total_work_orders,
            active_work_orders,
            open_escalations,
        })
    }

    async fn sla_compliance_summary(
        &self,
        now_epoch: u64,
        scope: DataScope,
    ) -> Result<SlaComplianceSummary, DomainError> {
        let requests = self.requests.list(scope).await?;
        let open_requests = requests.into_iter().filter(|r| !r.is_terminal()).collect::<Vec<_>>();
        let total_open_requests = open_requests.len();
        let overdue_open_requests = open_requests
            .iter()
            .filter(|r| now_epoch > r.created_at_epoch_sec + (r.sla_minutes as u64) * 60)
            .count();
        let compliant_open_requests = total_open_requests.saturating_sub(overdue_open_requests);
        let compliance_percent = if total_open_requests == 0 {
            100.0
        } else {
            (compliant_open_requests as f64 / total_open_requests as f64) * 100.0
        };

        Ok(SlaComplianceSummary {
            total_open_requests,
            overdue_open_requests,
            compliant_open_requests,
            compliance_percent,
        })
    }

    async fn sla_compliance_by_priority_summary(
        &self,
        now_epoch: u64,
        scope: DataScope,
    ) -> Result<Vec<SlaComplianceByPriorityItem>, DomainError> {
        #[derive(Default, Clone, Copy)]
        struct Counters {
            total: usize,
            overdue: usize,
        }

        let mut low = Counters::default();
        let mut medium = Counters::default();
        let mut high = Counters::default();
        let mut critical = Counters::default();

        let requests = self.requests.list(scope).await?;
        for req in requests.into_iter().filter(|r| !r.is_terminal()) {
            let overdue = now_epoch > req.created_at_epoch_sec + (req.sla_minutes as u64) * 60;
            match req.priority {
                Priority::Low => {
                    low.total += 1;
                    if overdue {
                        low.overdue += 1;
                    }
                }
                Priority::Medium => {
                    medium.total += 1;
                    if overdue {
                        medium.overdue += 1;
                    }
                }
                Priority::High => {
                    high.total += 1;
                    if overdue {
                        high.overdue += 1;
                    }
                }
                Priority::Critical => {
                    critical.total += 1;
                    if overdue {
                        critical.overdue += 1;
                    }
                }
            }
        }

        let mut items = Vec::with_capacity(4);
        for (priority, counters) in [
            (Priority::Low, low),
            (Priority::Medium, medium),
            (Priority::High, high),
            (Priority::Critical, critical),
        ] {
            if counters.total == 0 {
                continue;
            }
            let compliant = counters.total.saturating_sub(counters.overdue);
            let compliance_percent = (compliant as f64 / counters.total as f64) * 100.0;
            items.push(SlaComplianceByPriorityItem {
                priority: format!("{:?}", priority),
                total_open_requests: counters.total,
                overdue_open_requests: counters.overdue,
                compliant_open_requests: compliant,
                compliance_percent,
            });
        }

        items.sort_by(|a, b| a.priority.cmp(&b.priority));
        Ok(items)
    }

    async fn technician_workload_summary(
        &self,
        scope: DataScope,
    ) -> Result<Vec<TechnicianWorkloadSummary>, DomainError> {
        let technicians = self.technicians.list(scope.clone()).await?;
        let orders = self.work_orders.list(scope).await?;

        let mut by_id: HashMap<String, TechnicianWorkloadSummary> = technicians
            .into_iter()
            .map(|t| {
                (
                    t.id.clone(),
                    TechnicianWorkloadSummary {
                        technician_id: t.id,
                        full_name: t.full_name,
                        assigned: 0,
                        in_progress: 0,
                        completed: 0,
                        total: 0,
                    },
                )
            })
            .collect();

        for order in orders {
            let Some(assignee) = order.assignee else {
                continue;
            };
            let Some(entry) = by_id.get_mut(&assignee) else {
                continue;
            };
            entry.total += 1;
            match order.status {
                WorkOrderStatus::Assigned => entry.assigned += 1,
                WorkOrderStatus::InProgress => entry.in_progress += 1,
                WorkOrderStatus::Completed => entry.completed += 1,
                WorkOrderStatus::Created | WorkOrderStatus::Cancelled => {}
            }
        }

        let mut items: Vec<TechnicianWorkloadSummary> = by_id.into_values().collect();
        items.sort_by(|a, b| {
            b.total
                .cmp(&a.total)
                .then_with(|| a.technician_id.cmp(&b.technician_id))
        });
        Ok(items)
    }

    async fn list_requests(&self, scope: DataScope) -> Result<Vec<ServiceRequest>, DomainError> {
        self.requests.list(scope).await
    }

    async fn list_work_orders(&self, scope: DataScope) -> Result<Vec<WorkOrder>, DomainError> {
        self.work_orders.list(scope).await
    }

    async fn list_escalations(&self, scope: DataScope) -> Result<Vec<Escalation>, DomainError> {
        self.escalations.list(scope).await
    }

    async fn list_technicians(&self, scope: DataScope) -> Result<Vec<Technician>, DomainError> {
        self.technicians.list(scope).await
    }
}

#[derive(Clone)]
pub struct InMemoryAnalyticsSnapshotRepository {
    latest: Arc<Mutex<Option<AnalyticsSnapshot>>>,
}

impl InMemoryAnalyticsSnapshotRepository {
    pub fn new() -> Self {
        Self {
            latest: Arc::new(Mutex::new(None)),
        }
    }
}

#[async_trait]
impl AnalyticsSnapshotRepository for InMemoryAnalyticsSnapshotRepository {
    async fn get_latest(&self) -> Result<Option<AnalyticsSnapshot>, DomainError> {
        Ok(self.latest.lock().await.clone())
    }

    async fn upsert(&self, snapshot: AnalyticsSnapshot) -> Result<(), DomainError> {
        *self.latest.lock().await = Some(snapshot);
        Ok(())
    }
}
