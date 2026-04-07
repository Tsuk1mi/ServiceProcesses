use std::sync::Arc;

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
        id: String,
        full_name: String,
        skills: Vec<String>,
        owner_user_id: String,
    ) -> Result<Technician, DomainError> {
        let item = Technician::new(id, full_name, skills, owner_user_id)?;
        self.technicians.save(item.clone()).await?;
        Ok(item)
    }

    pub async fn list(&self, scope: DataScope) -> Result<Vec<Technician>, DomainError> {
        self.technicians.list(scope).await
    }
}
