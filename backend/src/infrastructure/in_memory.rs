use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::domain::entities::{Asset, AuditRecord, Escalation, ServiceRequest, Technician, WorkOrder};
use crate::domain::errors::DomainError;
use crate::domain::analytics::{
    AnalyticsSnapshot, DashboardSummary, SlaComplianceByPriorityItem, SlaComplianceSummary,
    TechnicianWorkloadSummary,
};
use crate::domain::value_objects::{EscalationState, Priority, RequestStatus, WorkOrderStatus};
use crate::ports::outbound::{
    AnalyticsQueryPort, AnalyticsSnapshotRepository, AssetRepository, AuditRepository, EscalationRepository,
    EventPublisherPort, PriorityPolicyPort,
    ServiceRequestRepository, SlaPolicyPort, TechnicianRepository, WorkOrderRepository,
};

#[derive(Clone, Default)]
pub struct InMemoryAssetRepository {
    data: Arc<Mutex<HashMap<String, Asset>>>,
}

impl InMemoryAssetRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl AssetRepository for InMemoryAssetRepository {
    fn save(&self, asset: Asset) -> Result<(), DomainError> {
        self.data
            .lock()
            .expect("asset repo mutex poisoned")
            .insert(asset.id.clone(), asset);
        Ok(())
    }

    fn get_by_id(&self, id: &str) -> Result<Option<Asset>, DomainError> {
        Ok(self.data.lock().expect("asset repo mutex poisoned").get(id).cloned())
    }

    fn list(&self) -> Result<Vec<Asset>, DomainError> {
        Ok(self
            .data
            .lock()
            .expect("asset repo mutex poisoned")
            .values()
            .cloned()
            .collect())
    }
}

#[derive(Clone, Default)]
pub struct InMemoryRequestRepository {
    data: Arc<Mutex<HashMap<String, ServiceRequest>>>,
}

impl InMemoryRequestRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ServiceRequestRepository for InMemoryRequestRepository {
    fn save(&self, request: ServiceRequest) -> Result<(), DomainError> {
        self.data
            .lock()
            .expect("request repo mutex poisoned")
            .insert(request.id.clone(), request);
        Ok(())
    }

    fn get_by_id(&self, id: &str) -> Result<Option<ServiceRequest>, DomainError> {
        Ok(self
            .data
            .lock()
            .expect("request repo mutex poisoned")
            .get(id)
            .cloned())
    }

    fn list(&self) -> Result<Vec<ServiceRequest>, DomainError> {
        Ok(self
            .data
            .lock()
            .expect("request repo mutex poisoned")
            .values()
            .cloned()
            .collect())
    }

    fn update(&self, request: ServiceRequest) -> Result<(), DomainError> {
        self.data
            .lock()
            .expect("request repo mutex poisoned")
            .insert(request.id.clone(), request);
        Ok(())
    }
}

#[derive(Clone, Default)]
pub struct InMemoryWorkOrderRepository {
    data: Arc<Mutex<HashMap<String, WorkOrder>>>,
}

impl InMemoryWorkOrderRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl WorkOrderRepository for InMemoryWorkOrderRepository {
    fn save(&self, work_order: WorkOrder) -> Result<(), DomainError> {
        self.data
            .lock()
            .expect("work order repo mutex poisoned")
            .insert(work_order.id.clone(), work_order);
        Ok(())
    }

    fn get_by_id(&self, id: &str) -> Result<Option<WorkOrder>, DomainError> {
        Ok(self
            .data
            .lock()
            .expect("work order repo mutex poisoned")
            .get(id)
            .cloned())
    }

    fn list(&self) -> Result<Vec<WorkOrder>, DomainError> {
        Ok(self
            .data
            .lock()
            .expect("work order repo mutex poisoned")
            .values()
            .cloned()
            .collect())
    }

    fn list_by_request(&self, request_id: &str) -> Result<Vec<WorkOrder>, DomainError> {
        Ok(self
            .data
            .lock()
            .expect("work order repo mutex poisoned")
            .values()
            .filter(|wo| wo.request_id == request_id)
            .cloned()
            .collect())
    }

    fn update(&self, work_order: WorkOrder) -> Result<(), DomainError> {
        self.data
            .lock()
            .expect("work order repo mutex poisoned")
            .insert(work_order.id.clone(), work_order);
        Ok(())
    }
}

#[derive(Clone, Default)]
pub struct InMemoryEscalationRepository {
    data: Arc<Mutex<HashMap<String, Escalation>>>,
}

impl InMemoryEscalationRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl EscalationRepository for InMemoryEscalationRepository {
    fn save(&self, escalation: Escalation) -> Result<(), DomainError> {
        self.data
            .lock()
            .expect("escalation repo mutex poisoned")
            .insert(escalation.id.clone(), escalation);
        Ok(())
    }

    fn get_by_id(&self, id: &str) -> Result<Option<Escalation>, DomainError> {
        Ok(self
            .data
            .lock()
            .expect("escalation repo mutex poisoned")
            .get(id)
            .cloned())
    }

    fn list(&self) -> Result<Vec<Escalation>, DomainError> {
        Ok(self
            .data
            .lock()
            .expect("escalation repo mutex poisoned")
            .values()
            .cloned()
            .collect())
    }

    fn list_by_request(&self, request_id: &str) -> Result<Vec<Escalation>, DomainError> {
        Ok(self
            .data
            .lock()
            .expect("escalation repo mutex poisoned")
            .values()
            .filter(|item| item.request_id == request_id)
            .cloned()
            .collect())
    }

    fn update(&self, escalation: Escalation) -> Result<(), DomainError> {
        self.data
            .lock()
            .expect("escalation repo mutex poisoned")
            .insert(escalation.id.clone(), escalation);
        Ok(())
    }
}

#[derive(Clone, Default)]
pub struct InMemoryTechnicianRepository {
    data: Arc<Mutex<HashMap<String, Technician>>>,
}

impl InMemoryTechnicianRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl TechnicianRepository for InMemoryTechnicianRepository {
    fn save(&self, technician: Technician) -> Result<(), DomainError> {
        self.data
            .lock()
            .expect("technician repo mutex poisoned")
            .insert(technician.id.clone(), technician);
        Ok(())
    }

    fn get_by_id(&self, id: &str) -> Result<Option<Technician>, DomainError> {
        Ok(self
            .data
            .lock()
            .expect("technician repo mutex poisoned")
            .get(id)
            .cloned())
    }

    fn list(&self) -> Result<Vec<Technician>, DomainError> {
        Ok(self
            .data
            .lock()
            .expect("technician repo mutex poisoned")
            .values()
            .cloned()
            .collect())
    }
}

#[derive(Clone, Default)]
pub struct InMemoryAuditRepository {
    data: Arc<Mutex<Vec<AuditRecord>>>,
}

impl InMemoryAuditRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl AuditRepository for InMemoryAuditRepository {
    fn save(&self, record: AuditRecord) -> Result<(), DomainError> {
        self.data.lock().expect("audit repo mutex poisoned").push(record);
        Ok(())
    }

    fn list(&self) -> Result<Vec<AuditRecord>, DomainError> {
        Ok(self
            .data
            .lock()
            .expect("audit repo mutex poisoned")
            .iter()
            .cloned()
            .collect())
    }

    fn list_by_request(&self, request_id: &str) -> Result<Vec<AuditRecord>, DomainError> {
        Ok(self
            .data
            .lock()
            .expect("audit repo mutex poisoned")
            .iter()
            .filter(|r| r.request_id.as_deref() == Some(request_id))
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

impl EventPublisherPort for StdoutEventPublisher {
    fn publish(&self, topic: &str, payload: &str) -> Result<(), DomainError> {
        println!("[event] topic={topic}; payload={payload}");
        Ok(())
    }
}

#[derive(Clone)]
pub struct InMemoryAnalyticsQuery {
    pub requests: InMemoryRequestRepository,
    pub work_orders: InMemoryWorkOrderRepository,
    pub escalations: InMemoryEscalationRepository,
    pub technicians: InMemoryTechnicianRepository,
}

impl AnalyticsQueryPort for InMemoryAnalyticsQuery {
    fn dashboard_summary(&self, now_epoch: u64) -> Result<DashboardSummary, DomainError> {
        let requests = self.requests.list()?;
        let work_orders = self.work_orders.list()?;
        let escalations = self.escalations.list()?;

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

    fn sla_compliance_summary(&self, now_epoch: u64) -> Result<SlaComplianceSummary, DomainError> {
        let requests = self.requests.list()?;
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

    fn sla_compliance_by_priority_summary(
        &self,
        now_epoch: u64,
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

        let requests = self.requests.list()?;
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

    fn technician_workload_summary(
        &self,
    ) -> Result<Vec<TechnicianWorkloadSummary>, DomainError> {
        let technicians = self.technicians.list()?;
        let orders = self.work_orders.list()?;

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
            b.total.cmp(&a.total).then_with(|| a.technician_id.cmp(&b.technician_id))
        });
        Ok(items)
    }

    fn list_requests(&self) -> Result<Vec<ServiceRequest>, DomainError> {
        self.requests.list()
    }

    fn list_work_orders(&self) -> Result<Vec<WorkOrder>, DomainError> {
        self.work_orders.list()
    }

    fn list_escalations(&self) -> Result<Vec<Escalation>, DomainError> {
        self.escalations.list()
    }

    fn list_technicians(&self) -> Result<Vec<Technician>, DomainError> {
        self.technicians.list()
    }
}

#[derive(Clone, Default)]
pub struct InMemoryAnalyticsSnapshotRepository {
    latest: Arc<Mutex<Option<AnalyticsSnapshot>>>,
}

impl InMemoryAnalyticsSnapshotRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl AnalyticsSnapshotRepository for InMemoryAnalyticsSnapshotRepository {
    fn get_latest(&self) -> Result<Option<AnalyticsSnapshot>, DomainError> {
        Ok(self.latest.lock().expect("analytics snapshot mutex poisoned").clone())
    }

    fn upsert(&self, snapshot: AnalyticsSnapshot) -> Result<(), DomainError> {
        *self.latest.lock().expect("analytics snapshot mutex poisoned") = Some(snapshot);
        Ok(())
    }
}
