use std::sync::Arc;

use crate::application::rbac;
use crate::auth::AuthUser;
use crate::domain::events::make_event;
use crate::domain::entities::{Escalation, ServiceRequest};
use crate::domain::errors::DomainError;
use crate::domain::value_objects::EscalationState;
use crate::ports::data_scope::DataScope;
use crate::ports::outbound::{EscalationRepository, EventPublisherPort, ServiceRequestRepository};

#[derive(Clone)]
pub struct SlaAppService {
    pub requests: Arc<dyn ServiceRequestRepository>,
    pub escalations: Arc<dyn EscalationRepository>,
    pub events: Arc<dyn EventPublisherPort>,
}

impl SlaAppService {
    pub async fn list_overdue_requests(
        &self,
        now_epoch: u64,
        scope: DataScope,
    ) -> Result<Vec<ServiceRequest>, DomainError> {
        let requests = self.requests.list(scope).await?;
        Ok(requests
            .into_iter()
            .filter(|r| !r.is_terminal())
            .filter(|r| now_epoch > r.created_at_epoch_sec + (r.sla_minutes as u64) * 60)
            .collect())
    }

    pub async fn auto_escalate_overdue(
        &self,
        caller: &AuthUser,
        now_epoch: u64,
        reason: &str,
    ) -> Result<Vec<Escalation>, DomainError> {
        rbac::require_any_role(caller, &["admin", "supervisor", "dispatcher"])?;
        let overdue = self.list_overdue_requests(now_epoch, DataScope::All).await?;
        let mut created = Vec::new();
        for req in overdue {
            let existing = self.escalations.list_by_request(&req.id, DataScope::All).await?;
            let has_open = existing.iter().any(|e| e.state == EscalationState::Open);
            if has_open {
                continue;
            }
            let esc_id = format!("esc-worker-{}-{}", now_epoch, req.id);
            let escalation = Escalation::new(esc_id, req.id.clone(), reason.to_string(), req.owner_user_id.clone())?;
            self.escalations.save(escalation.clone(), DataScope::All).await?;
            let payload = make_event(
                "escalation.created",
                escalation.id.clone(),
                escalation.owner_user_id.clone(),
                serde_json::json!({
                    "escalation_id": escalation.id,
                    "request_id": escalation.request_id,
                    "reason": escalation.reason,
                    "state": format!("{:?}", escalation.state),
                    "source": "sla_worker"
                }),
            )?;
            self.events
                .publish("escalation.created", &payload)
                .await?;
            created.push(escalation);
        }
        Ok(created)
    }
}
