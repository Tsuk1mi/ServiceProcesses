use crate::domain::errors::DomainError;
use crate::domain::value_objects::{
    AssetState, EscalationState, Priority, RequestStatus, WorkOrderStatus,
};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Asset {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub location: String,
    pub state: AssetState,
    /// Владелец записи (UUID пользователя). RBAC: не-admin видит только свои сущности.
    pub owner_user_id: String,
}

impl Asset {
    pub fn new(
        id: String,
        kind: String,
        title: String,
        location: String,
        owner_user_id: String,
    ) -> Result<Self, DomainError> {
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
        if owner_user_id.trim().is_empty() {
            return Err(DomainError::EmptyField("owner_user_id"));
        }

        Ok(Self {
            id,
            kind,
            title,
            location,
            state: AssetState::Active,
            owner_user_id,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ServiceRequest {
    pub id: String,
    pub asset_id: String,
    pub description: String,
    pub priority: Priority,
    pub status: RequestStatus,
    pub sla_minutes: u32,
    pub created_at_epoch_sec: u64,
    pub owner_user_id: String,
}

impl ServiceRequest {
    pub fn new(
        id: String,
        asset_id: String,
        description: String,
        priority: Priority,
        sla_minutes: u32,
        owner_user_id: String,
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
        if owner_user_id.trim().is_empty() {
            return Err(DomainError::EmptyField("owner_user_id"));
        }

        Ok(Self {
            id,
            asset_id,
            description,
            priority,
            status: RequestStatus::New,
            sla_minutes,
            created_at_epoch_sec: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            owner_user_id,
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

    pub fn is_terminal(&self) -> bool {
        matches!(self.status, RequestStatus::Resolved | RequestStatus::Closed)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WorkOrder {
    pub id: String,
    pub request_id: String,
    pub assignee: Option<String>,
    pub status: WorkOrderStatus,
    pub owner_user_id: String,
}

impl WorkOrder {
    pub fn new(id: String, request_id: String, owner_user_id: String) -> Result<Self, DomainError> {
        if id.trim().is_empty() {
            return Err(DomainError::EmptyField("id"));
        }
        if request_id.trim().is_empty() {
            return Err(DomainError::EmptyField("request_id"));
        }
        if owner_user_id.trim().is_empty() {
            return Err(DomainError::EmptyField("owner_user_id"));
        }

        Ok(Self {
            id,
            request_id,
            assignee: None,
            status: WorkOrderStatus::Created,
            owner_user_id,
        })
    }

    pub fn assign(&mut self, assignee: String) -> Result<(), DomainError> {
        if assignee.trim().is_empty() {
            return Err(DomainError::EmptyField("assignee"));
        }
        self.assignee = Some(assignee);
        self.status = WorkOrderStatus::Assigned;
        Ok(())
    }

    pub fn start(&mut self) -> Result<(), DomainError> {
        if self.status != WorkOrderStatus::Assigned {
            return Err(DomainError::InvalidTransition);
        }
        self.status = WorkOrderStatus::InProgress;
        Ok(())
    }

    pub fn complete(&mut self) -> Result<(), DomainError> {
        if self.status != WorkOrderStatus::InProgress {
            return Err(DomainError::InvalidTransition);
        }
        self.status = WorkOrderStatus::Completed;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Escalation {
    pub id: String,
    pub request_id: String,
    pub reason: String,
    pub state: EscalationState,
    pub owner_user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Technician {
    pub id: String,
    pub full_name: String,
    pub skills: Vec<String>,
    pub is_active: bool,
    pub owner_user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AuditRecord {
    pub id: String,
    pub request_id: Option<String>,
    pub entity: String,
    pub action: String,
    pub actor_role: String,
    pub actor_id: Option<String>,
    pub details: String,
    pub created_at_utc: String,
    pub owner_user_id: String,
}

impl AuditRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        request_id: Option<String>,
        entity: String,
        action: String,
        actor_role: String,
        actor_id: Option<String>,
        details: String,
        created_at_utc: String,
        owner_user_id: String,
    ) -> Result<Self, DomainError> {
        if id.trim().is_empty() {
            return Err(DomainError::EmptyField("id"));
        }
        if entity.trim().is_empty() {
            return Err(DomainError::EmptyField("entity"));
        }
        if action.trim().is_empty() {
            return Err(DomainError::EmptyField("action"));
        }
        if actor_role.trim().is_empty() {
            return Err(DomainError::EmptyField("actor_role"));
        }
        if details.trim().is_empty() {
            return Err(DomainError::EmptyField("details"));
        }
        if created_at_utc.trim().is_empty() {
            return Err(DomainError::EmptyField("created_at_utc"));
        }
        if owner_user_id.trim().is_empty() {
            return Err(DomainError::EmptyField("owner_user_id"));
        }

        Ok(Self {
            id,
            request_id,
            entity,
            action,
            actor_role,
            actor_id,
            details,
            created_at_utc,
            owner_user_id,
        })
    }
}

impl Technician {
    pub fn new(
        id: String,
        full_name: String,
        skills: Vec<String>,
        owner_user_id: String,
    ) -> Result<Self, DomainError> {
        if id.trim().is_empty() {
            return Err(DomainError::EmptyField("id"));
        }
        if full_name.trim().is_empty() {
            return Err(DomainError::EmptyField("full_name"));
        }
        if skills.is_empty() {
            return Err(DomainError::EmptyField("skills"));
        }
        if owner_user_id.trim().is_empty() {
            return Err(DomainError::EmptyField("owner_user_id"));
        }

        Ok(Self {
            id,
            full_name,
            skills,
            is_active: true,
            owner_user_id,
        })
    }
}

impl Escalation {
    pub fn new(
        id: String,
        request_id: String,
        reason: String,
        owner_user_id: String,
    ) -> Result<Self, DomainError> {
        if id.trim().is_empty() {
            return Err(DomainError::EmptyField("id"));
        }
        if request_id.trim().is_empty() {
            return Err(DomainError::EmptyField("request_id"));
        }
        if reason.trim().is_empty() {
            return Err(DomainError::EmptyField("reason"));
        }
        if owner_user_id.trim().is_empty() {
            return Err(DomainError::EmptyField("owner_user_id"));
        }

        Ok(Self {
            id,
            request_id,
            reason,
            state: EscalationState::Open,
            owner_user_id,
        })
    }

    pub fn resolve(&mut self) -> Result<(), DomainError> {
        if self.state != EscalationState::Open {
            return Err(DomainError::InvalidTransition);
        }
        self.state = EscalationState::Resolved;
        Ok(())
    }
}
