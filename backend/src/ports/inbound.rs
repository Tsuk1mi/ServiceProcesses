use async_trait::async_trait;

use crate::domain::errors::DomainError;
use crate::domain::value_objects::RequestStatus;
use crate::ports::data_scope::DataScope;

#[derive(Debug, Clone)]
pub struct CreateRequestCommand {
    pub request_id: String,
    pub asset_id: String,
    pub description: String,
}

#[async_trait]
pub trait ServiceRequestUseCase: Send + Sync {
    async fn create_request(&self, command: CreateRequestCommand, scope: DataScope) -> Result<(), DomainError>;
    async fn update_status(
        &self,
        request_id: &str,
        next: RequestStatus,
        scope: DataScope,
    ) -> Result<(), DomainError>;
}
