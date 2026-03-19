use crate::domain::entities::{Escalation, ServiceRequest};
use crate::domain::errors::DomainError;
use crate::domain::value_objects::EscalationState;
use crate::ports::outbound::{EscalationRepository, EventPublisherPort, ServiceRequestRepository};

#[derive(Clone)]
pub struct SlaAppService<R, ERepo, Event>
where
    R: ServiceRequestRepository,
    ERepo: EscalationRepository,
    Event: EventPublisherPort,
{
    pub requests: R,
    pub escalations: ERepo,
    pub events: Event,
}

impl<R, ERepo, Event> SlaAppService<R, ERepo, Event>
where
    R: ServiceRequestRepository,
    ERepo: EscalationRepository,
    Event: EventPublisherPort,
{
    pub fn list_overdue_requests(&self, now_epoch: u64) -> Result<Vec<ServiceRequest>, DomainError> {
        let requests = self.requests.list()?;
        Ok(requests
            .into_iter()
            .filter(|r| !r.is_terminal())
            .filter(|r| now_epoch > r.created_at_epoch_sec + (r.sla_minutes as u64) * 60)
            .collect())
    }

    pub fn auto_escalate_overdue(
        &self,
        now_epoch: u64,
        reason: &str,
    ) -> Result<Vec<Escalation>, DomainError> {
        let overdue = self.list_overdue_requests(now_epoch)?;
        let mut created = Vec::new();
        for req in overdue {
            let existing = self.escalations.list_by_request(&req.id)?;
            let has_open = existing.iter().any(|e| e.state == EscalationState::Open);
            if has_open {
                continue;
            }
            let esc_id = format!("esc-worker-{}-{}", now_epoch, req.id);
            let escalation = Escalation::new(esc_id, req.id.clone(), reason.to_string())?;
            self.escalations.save(escalation.clone())?;
            self.events.publish(
                "service_request.escalated",
                &format!("id={},request_id={}", escalation.id, escalation.request_id),
            )?;
            created.push(escalation);
        }
        Ok(created)
    }
}
