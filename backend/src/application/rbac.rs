//! Проверка RBAC в слое application (дублирует границы HTTP и защищает воркер очереди).

use crate::auth::AuthUser;
use crate::domain::errors::DomainError;

pub fn require_any_role(caller: &AuthUser, allowed: &[&str]) -> Result<(), DomainError> {
    if caller.has_any_role(allowed) {
        Ok(())
    } else {
        Err(DomainError::Forbidden("operation is not allowed for current role"))
    }
}

/// Актор для фоновых процессов (SLA-воркер), которым нужен полный доступ к данным.
pub fn system_admin_actor() -> AuthUser {
    AuthUser {
        sub: uuid::Uuid::nil(),
        roles: vec!["admin".to_string()],
    }
}
