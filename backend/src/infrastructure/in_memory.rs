use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::domain::entities::{Asset, ServiceRequest, WorkOrder};
use crate::domain::errors::DomainError;
use crate::domain::value_objects::Priority;
use crate::ports::outbound::{
    AssetRepository, EventPublisherPort, PriorityPolicyPort, ServiceRequestRepository, SlaPolicyPort,
    WorkOrderRepository,
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
