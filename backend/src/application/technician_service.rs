use std::sync::Arc;

use crate::application::rbac;
use crate::auth::AuthUser;
use crate::domain::entities::Technician;
use crate::domain::errors::DomainError;
use crate::ports::data_scope::DataScope;
use crate::ports::outbound::TechnicianRepository;

#[derive(Clone)]
pub struct TechnicianAppService {
    pub technicians: Arc<dyn TechnicianRepository>,
}

impl TechnicianAppService {
    pub async fn create(
        &self,
        caller: &AuthUser,
        id: String,
        full_name: String,
        skills: Vec<String>,
        owner_user_id: String,
    ) -> Result<Technician, DomainError> {
        rbac::require_any_role(caller, &["admin", "supervisor"])?;
        let scope = caller.data_scope();
        let item = Technician::new(id, full_name, skills, owner_user_id)?;
        self.technicians.save(item.clone(), scope).await?;
        Ok(item)
    }

    pub async fn list(&self, scope: DataScope) -> Result<Vec<Technician>, DomainError> {
        self.technicians.list(scope).await
    }
}
