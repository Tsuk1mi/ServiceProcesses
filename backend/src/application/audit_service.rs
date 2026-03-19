use std::time::{SystemTime, UNIX_EPOCH};

use crate::domain::entities::AuditRecord;
use crate::domain::errors::DomainError;
use crate::ports::outbound::AuditRepository;

#[derive(Clone)]
pub struct AuditAppService<A>
where
    A: AuditRepository,
{
    pub audit: A,
}

impl<A> AuditAppService<A>
where
    A: AuditRepository,
{
    pub fn record(
        &self,
        request_id: Option<String>,
        entity: &str,
        action: &str,
        actor_role: &str,
        actor_id: Option<String>,
        details: String,
    ) -> Result<(), DomainError> {
        let id = format!(
            "aud-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
        let created_at_utc = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .to_string();

        let record = AuditRecord::new(
            id,
            request_id,
            entity.to_string(),
            action.to_string(),
            actor_role.to_string(),
            actor_id,
            details,
            created_at_utc,
        )?;
        self.audit.save(record)
    }

    pub fn list_by_request(&self, request_id: &str) -> Result<Vec<AuditRecord>, DomainError> {
        self.audit.list_by_request(request_id)
    }
}
