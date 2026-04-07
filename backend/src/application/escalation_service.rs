use crate::domain::entities::Escalation;
use crate::domain::errors::DomainError;
use crate::ports::data_scope::DataScope;
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
    R: ServiceRequestRepository + Send + Sync,
    ERepo: EscalationRepository + Send + Sync,
    Event: EventPublisherPort + Send + Sync,
{
    pub async fn create_escalation(
        &self,
        escalation_id: String,
        request_id: String,
        reason: String,
        scope: DataScope,
    ) -> Result<Escalation, DomainError> {
        let request = self
            .requests
            .get_by_id(&request_id, scope.clone())
            .await?
            .ok_or(DomainError::NotFound("service_request"))?;

        let escalation = Escalation::new(escalation_id, request_id, reason, request.owner_user_id)?;
        self.escalations.save(escalation.clone()).await?;
        self.events
            .publish(
                "service_request.escalated",
                &format!("id={},request_id={}", escalation.id, escalation.request_id),
            )
            .await?;

        Ok(escalation)
    }

    pub async fn list_by_request(
        &self,
        request_id: &str,
        scope: DataScope,
    ) -> Result<Vec<Escalation>, DomainError> {
        self.escalations.list_by_request(request_id, scope).await
    }

    pub async fn list_all(&self, scope: DataScope) -> Result<Vec<Escalation>, DomainError> {
        self.escalations.list(scope).await
    }

    pub async fn resolve(&self, escalation_id: &str, scope: DataScope) -> Result<Escalation, DomainError> {
        let mut escalation = self
            .escalations
            .get_by_id(escalation_id, scope.clone())
            .await?
            .ok_or(DomainError::NotFound("escalation"))?;
        escalation.resolve()?;
        self.escalations.update(escalation.clone()).await?;
        self.events
            .publish(
                "service_request.escalation_resolved",
                &format!("id={},request_id={}", escalation.id, escalation.request_id),
            )
            .await?;
        Ok(escalation)
    }
}
