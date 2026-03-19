use crate::domain::errors::DomainError;
use crate::domain::value_objects::{
    AssetState, EscalationState, Priority, RequestStatus, WorkOrderStatus,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub location: String,
    pub state: AssetState,
}

impl Asset {
    pub fn new(id: String, kind: String, title: String, location: String) -> Result<Self, DomainError> {
        if id.trim().is_empty() {
            return Err(DomainError::EmptyField("id"));
        }
        if kind.trim().is_empty() {
            return Err(DomainError::EmptyField("kind"));
        }
        if title.trim().is_empty() {
            return Err(DomainError::EmptyField("title"));
        }
        if location.trim().is_empty() {
            return Err(DomainError::EmptyField("location"));
        }

        Ok(Self {
            id,
            kind,
            title,
            location,
            state: AssetState::Active,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRequest {
    pub id: String,
    pub asset_id: String,
    pub description: String,
    pub priority: Priority,
    pub status: RequestStatus,
    pub sla_minutes: u32,
}

impl ServiceRequest {
    pub fn new(
        id: String,
        asset_id: String,
        description: String,
        priority: Priority,
        sla_minutes: u32,
    ) -> Result<Self, DomainError> {
        if id.trim().is_empty() {
            return Err(DomainError::EmptyField("id"));
        }
        if asset_id.trim().is_empty() {
            return Err(DomainError::EmptyField("asset_id"));
        }
        if description.trim().is_empty() {
            return Err(DomainError::EmptyField("description"));
        }
        if sla_minutes == 0 {
            return Err(DomainError::EmptyField("sla_minutes"));
        }

        Ok(Self {
            id,
            asset_id,
            description,
            priority,
            status: RequestStatus::New,
            sla_minutes,
        })
    }

    pub fn transition_to(&mut self, next: RequestStatus) -> Result<(), DomainError> {
        let valid = matches!(
            (self.status, next),
            (RequestStatus::New, RequestStatus::Planned)
                | (RequestStatus::Planned, RequestStatus::InProgress)
                | (RequestStatus::InProgress, RequestStatus::Resolved)
                | (RequestStatus::Resolved, RequestStatus::Closed)
                | (_, RequestStatus::Escalated)
        );

        if !valid {
            return Err(DomainError::InvalidTransition);
        }

        self.status = next;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkOrder {
    pub id: String,
    pub request_id: String,
    pub assignee: Option<String>,
    pub status: WorkOrderStatus,
}

impl WorkOrder {
    pub fn new(id: String, request_id: String) -> Result<Self, DomainError> {
        if id.trim().is_empty() {
            return Err(DomainError::EmptyField("id"));
        }
        if request_id.trim().is_empty() {
            return Err(DomainError::EmptyField("request_id"));
        }

        Ok(Self {
            id,
            request_id,
            assignee: None,
            status: WorkOrderStatus::Created,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Escalation {
    pub id: String,
    pub request_id: String,
    pub reason: String,
    pub state: EscalationState,
}

impl Escalation {
    pub fn new(id: String, request_id: String, reason: String) -> Result<Self, DomainError> {
        if id.trim().is_empty() {
            return Err(DomainError::EmptyField("id"));
        }
        if request_id.trim().is_empty() {
            return Err(DomainError::EmptyField("request_id"));
        }
        if reason.trim().is_empty() {
            return Err(DomainError::EmptyField("reason"));
        }

        Ok(Self {
            id,
            request_id,
            reason,
            state: EscalationState::Open,
        })
    }
}
