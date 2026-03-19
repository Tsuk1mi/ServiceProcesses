use crate::domain::entities::{Asset, ServiceRequest};
use crate::domain::errors::DomainError;
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
