use std::sync::Arc;

use async_trait::async_trait;

use crate::application::rbac;
use crate::auth::AuthUser;
use crate::domain::entities::ServiceRequest;
use crate::domain::errors::DomainError;
use crate::domain::value_objects::RequestStatus;
use crate::ports::data_scope::DataScope;
use crate::ports::inbound::{CreateRequestCommand, ServiceRequestUseCase};
use crate::ports::outbound::{
    AssetRepository, EventPublisherPort, PriorityPolicyPort, ServiceRequestRepository, SlaPolicyPort,
};

#[derive(Clone)]
pub struct ServiceRequestAppService {
    pub assets: Arc<dyn AssetRepository>,
    pub requests: Arc<dyn ServiceRequestRepository>,
    pub sla: Arc<dyn SlaPolicyPort>,
    pub priority: Arc<dyn PriorityPolicyPort>,
    pub events: Arc<dyn EventPublisherPort>,
}

#[async_trait]
impl ServiceRequestUseCase for ServiceRequestAppService {
    async fn create_request(
        &self,
        caller: &AuthUser,
        command: CreateRequestCommand,
        scope: DataScope,
    ) -> Result<(), DomainError> {
        rbac::require_any_role(caller, &["admin", "dispatcher", "supervisor", "user"])?;
        let asset = self
            .assets
            .get_by_id(&command.asset_id, scope.clone())
            .await?
            .ok_or(DomainError::NotFound("asset"))?;

        let priority = self.priority.resolve_priority(&command.description)?;
        let sla_minutes = self.sla.resolve_sla_minutes(priority)?;

        let request = ServiceRequest::new(
            command.request_id,
            asset.id,
            command.description,
            priority,
            sla_minutes,
            asset.owner_user_id,
        )?;

        self.requests.save(request.clone(), scope.clone()).await?;
        tracing::info!(request_id = %request.id, asset_id = %request.asset_id, "service_request created");
        self.events
            .publish(
                "service_request.created",
                &format!(
                    "id={},asset_id={},priority={:?},sla={}",
                    request.id, request.asset_id, request.priority, request.sla_minutes
                ),
            )
            .await?;

        Ok(())
    }

    async fn update_status(
        &self,
        caller: &AuthUser,
        request_id: &str,
        next: RequestStatus,
        scope: DataScope,
    ) -> Result<(), DomainError> {
        rbac::require_any_role(caller, &["admin", "dispatcher", "supervisor"])?;
        let mut request = self
            .requests
            .get_by_id(request_id, scope.clone())
            .await?
            .ok_or(DomainError::NotFound("service_request"))?;

        request.transition_to(next)?;
        self.requests.update(request.clone(), scope.clone()).await?;
        self.events
            .publish(
                "service_request.status_changed",
                &format!("id={},status={:?}", request.id, request.status),
            )
            .await?;

        Ok(())
    }
}
