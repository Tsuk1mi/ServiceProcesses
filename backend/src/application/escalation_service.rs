use crate::domain::entities::Escalation;
use crate::domain::errors::DomainError;
use crate::ports::outbound::{EscalationRepository, EventPublisherPort, ServiceRequestRepository};

#[derive(Clone)]
pub struct EscalationAppService<R, ERepo, Event>
where
    R: ServiceRequestRepository,
    ERepo: EscalationRepository,
    Event: EventPublisherPort,
{
    pub requests: R,
    pub escalations: ERepo,
    pub events: Event,
}

impl<R, ERepo, Event> EscalationAppService<R, ERepo, Event>
where
    R: ServiceRequestRepository,
    ERepo: EscalationRepository,
    Event: EventPublisherPort,
{
    pub fn create_escalation(
        &self,
        escalation_id: String,
        request_id: String,
        reason: String,
    ) -> Result<Escalation, DomainError> {
        let _request = self
            .requests
            .get_by_id(&request_id)?
            .ok_or(DomainError::NotFound("service_request"))?;

        let escalation = Escalation::new(escalation_id, request_id, reason)?;
        self.escalations.save(escalation.clone())?;
        self.events.publish(
            "service_request.escalated",
            &format!("id={},request_id={}", escalation.id, escalation.request_id),
        )?;

        Ok(escalation)
    }

    pub fn list_by_request(&self, request_id: &str) -> Result<Vec<Escalation>, DomainError> {
        self.escalations.list_by_request(request_id)
    }

    pub fn resolve(&self, escalation_id: &str) -> Result<Escalation, DomainError> {
        let mut escalation = self
            .escalations
            .get_by_id(escalation_id)?
            .ok_or(DomainError::NotFound("escalation"))?;
        escalation.resolve()?;
        self.escalations.update(escalation.clone())?;
        self.events.publish(
            "service_request.escalation_resolved",
            &format!("id={},request_id={}", escalation.id, escalation.request_id),
        )?;
        Ok(escalation)
    }
}
