use crate::domain::entities::ServiceRequest;
use crate::domain::errors::DomainError;
use crate::domain::value_objects::RequestStatus;
use crate::ports::inbound::{CreateRequestCommand, ServiceRequestUseCase};
use crate::ports::outbound::{
    AssetRepository, EventPublisherPort, PriorityPolicyPort, ServiceRequestRepository, SlaPolicyPort,
};

#[derive(Clone)]
pub struct ServiceRequestAppService<A, R, S, P, E>
where
    A: AssetRepository,
    R: ServiceRequestRepository,
    S: SlaPolicyPort,
    P: PriorityPolicyPort,
    E: EventPublisherPort,
{
    pub assets: A,
    pub requests: R,
    pub sla: S,
    pub priority: P,
    pub events: E,
}

impl<A, R, S, P, E> ServiceRequestUseCase for ServiceRequestAppService<A, R, S, P, E>
where
    A: AssetRepository,
    R: ServiceRequestRepository,
    S: SlaPolicyPort,
    P: PriorityPolicyPort,
    E: EventPublisherPort,
{
    fn create_request(&self, command: CreateRequestCommand) -> Result<(), DomainError> {
        let asset = self
            .assets
            .get_by_id(&command.asset_id)?
            .ok_or(DomainError::NotFound("asset"))?;

        let priority = self.priority.resolve_priority(&command.description)?;
        let sla_minutes = self.sla.resolve_sla_minutes(priority)?;

        let request = ServiceRequest::new(
            command.request_id,
            asset.id,
            command.description,
            priority,
            sla_minutes,
        )?;

        self.requests.save(request.clone())?;
        self.events.publish(
            "service_request.created",
            &format!(
                "id={},asset_id={},priority={:?},sla={}",
                request.id, request.asset_id, request.priority, request.sla_minutes
            ),
        )?;

        Ok(())
    }

    fn update_status(&self, request_id: &str, next: RequestStatus) -> Result<(), DomainError> {
        let mut request = self
            .requests
            .get_by_id(request_id)?
            .ok_or(DomainError::NotFound("service_request"))?;

        request.transition_to(next)?;
        self.requests.update(request.clone())?;
        self.events.publish(
            "service_request.status_changed",
            &format!("id={},status={:?}", request.id, request.status),
        )?;

        Ok(())
    }
}
