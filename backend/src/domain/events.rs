use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::errors::DomainError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEventEnvelope {
    pub event_id: Uuid,
    pub event_type: String,
    pub entity_id: String,
    pub owner_user_id: String,
    pub occurred_at_epoch_sec: u64,
    pub payload: serde_json::Value,
}

impl DomainEventEnvelope {
    pub fn new(
        event_type: &str,
        entity_id: String,
        owner_user_id: String,
        payload: serde_json::Value,
    ) -> Result<Self, DomainError> {
        if event_type.trim().is_empty() {
            return Err(DomainError::EmptyField("event_type"));
        }
        if entity_id.trim().is_empty() {
            return Err(DomainError::EmptyField("entity_id"));
        }
        if owner_user_id.trim().is_empty() {
            return Err(DomainError::EmptyField("owner_user_id"));
        }

        Ok(Self {
            event_id: Uuid::new_v4(),
            event_type: event_type.to_string(),
            entity_id,
            owner_user_id,
            occurred_at_epoch_sec: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            payload,
        })
    }

    pub fn json_string(&self) -> Result<String, DomainError> {
        serde_json::to_string(self).map_err(|_| DomainError::EmptyField("domain_event"))
    }
}

pub fn make_event(
    event_type: &str,
    entity_id: String,
    owner_user_id: String,
    payload: serde_json::Value,
) -> Result<String, DomainError> {
    DomainEventEnvelope::new(event_type, entity_id, owner_user_id, payload)?.json_string()
}
