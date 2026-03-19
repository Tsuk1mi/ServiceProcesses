use crate::domain::errors::DomainError;
use crate::domain::value_objects::RequestStatus;

#[derive(Debug, Clone)]
pub struct CreateRequestCommand {
    pub request_id: String,
    pub asset_id: String,
    pub description: String,
}

pub trait ServiceRequestUseCase {
    fn create_request(&self, command: CreateRequestCommand) -> Result<(), DomainError>;
    fn update_status(&self, request_id: &str, next: RequestStatus) -> Result<(), DomainError>;
}
