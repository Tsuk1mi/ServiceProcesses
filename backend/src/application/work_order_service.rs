use std::sync::Arc;

use crate::application::rbac;
use crate::auth::AuthUser;
use crate::domain::events::make_event;
use crate::domain::entities::WorkOrder;
use crate::domain::errors::DomainError;
use crate::ports::data_scope::DataScope;
use crate::ports::outbound::{
    EventPublisherPort, ServiceRequestRepository, TechnicianRepository, WorkOrderRepository,
};

#[derive(Clone)]
pub struct WorkOrderAppService {
    pub requests: Arc<dyn ServiceRequestRepository>,
    pub work_orders: Arc<dyn WorkOrderRepository>,
    pub technicians: Arc<dyn TechnicianRepository>,
    pub events: Arc<dyn EventPublisherPort>,
}

impl WorkOrderAppService {
    pub async fn create_work_order(
        &self,
        caller: &AuthUser,
        id: String,
        request_id: String,
        scope: DataScope,
    ) -> Result<WorkOrder, DomainError> {
        rbac::require_any_role(caller, &["admin", "dispatcher", "supervisor"])?;
        let request = self
            .requests
            .get_by_id(&request_id, scope.clone())
            .await?
            .ok_or(DomainError::NotFound("service_request"))?;

        let work_order = WorkOrder::new(id, request_id, request.owner_user_id)?;
        self.work_orders.save(work_order.clone(), scope.clone()).await?;
        let payload = make_event(
            "work_order.created",
            work_order.id.clone(),
            work_order.owner_user_id.clone(),
            serde_json::json!({
                "work_order_id": work_order.id,
                "request_id": work_order.request_id,
                "status": format!("{:?}", work_order.status)
            }),
        )?;
        self.events
            .publish("work_order.created", &payload)
            .await?;

        Ok(work_order)
    }

    pub async fn list_by_request(
        &self,
        request_id: &str,
        scope: DataScope,
    ) -> Result<Vec<WorkOrder>, DomainError> {
        self.work_orders.list_by_request(request_id, scope).await
    }

    pub async fn assign(
        &self,
        caller: &AuthUser,
        work_order_id: &str,
        assignee: String,
        scope: DataScope,
    ) -> Result<WorkOrder, DomainError> {
        rbac::require_any_role(caller, &["admin", "dispatcher", "supervisor"])?;
        let _technician = self
            .technicians
            .get_by_id(&assignee, scope.clone())
            .await?
            .ok_or(DomainError::NotFound("technician"))?;

        let mut work_order = self
            .work_orders
            .get_by_id(work_order_id, scope.clone())
            .await?
            .ok_or(DomainError::NotFound("work_order"))?;
        work_order.assign(assignee)?;
        self.work_orders.update(work_order.clone(), scope.clone()).await?;
        let payload = make_event(
            "work_order.assigned",
            work_order.id.clone(),
            work_order.owner_user_id.clone(),
            serde_json::json!({
                "work_order_id": work_order.id,
                "request_id": work_order.request_id,
                "assignee": work_order.assignee.clone(),
                "status": format!("{:?}", work_order.status)
            }),
        )?;
        self.events
            .publish("work_order.assigned", &payload)
            .await?;
        Ok(work_order)
    }

    pub async fn start(&self, caller: &AuthUser, work_order_id: &str, scope: DataScope) -> Result<WorkOrder, DomainError> {
        rbac::require_any_role(caller, &["admin", "technician", "dispatcher", "supervisor"])?;
        let mut work_order = self
            .work_orders
            .get_by_id(work_order_id, scope.clone())
            .await?
            .ok_or(DomainError::NotFound("work_order"))?;
        work_order.start()?;
        self.work_orders.update(work_order.clone(), scope.clone()).await?;
        let payload = make_event(
            "work_order.started",
            work_order.id.clone(),
            work_order.owner_user_id.clone(),
            serde_json::json!({
                "work_order_id": work_order.id,
                "request_id": work_order.request_id,
                "status": format!("{:?}", work_order.status)
            }),
        )?;
        self.events
            .publish("work_order.started", &payload)
            .await?;
        Ok(work_order)
    }

    pub async fn start_by_actor(
        &self,
        caller: &AuthUser,
        work_order_id: &str,
        actor_id: &str,
        scope: DataScope,
    ) -> Result<WorkOrder, DomainError> {
        rbac::require_any_role(caller, &["admin", "technician", "dispatcher", "supervisor"])?;
        let mut work_order = self
            .work_orders
            .get_by_id(work_order_id, scope.clone())
            .await?
            .ok_or(DomainError::NotFound("work_order"))?;
        let assignee = work_order
            .assignee
            .as_deref()
            .ok_or(DomainError::Forbidden("work order has no assignee"))?;
        if assignee != actor_id {
            return Err(DomainError::Forbidden(
                "technician can start only their own work order",
            ));
        }
        work_order.start()?;
        self.work_orders.update(work_order.clone(), scope.clone()).await?;
        let payload = make_event(
            "work_order.started",
            work_order.id.clone(),
            work_order.owner_user_id.clone(),
            serde_json::json!({
                "work_order_id": work_order.id,
                "request_id": work_order.request_id,
                "status": format!("{:?}", work_order.status)
            }),
        )?;
        self.events
            .publish("work_order.started", &payload)
            .await?;
        Ok(work_order)
    }

    pub async fn complete(&self, caller: &AuthUser, work_order_id: &str, scope: DataScope) -> Result<WorkOrder, DomainError> {
        rbac::require_any_role(caller, &["admin", "technician", "dispatcher", "supervisor"])?;
        let mut work_order = self
            .work_orders
            .get_by_id(work_order_id, scope.clone())
            .await?
            .ok_or(DomainError::NotFound("work_order"))?;
        work_order.complete()?;
        self.work_orders.update(work_order.clone(), scope.clone()).await?;
        let payload = make_event(
            "work_order.completed",
            work_order.id.clone(),
            work_order.owner_user_id.clone(),
            serde_json::json!({
                "work_order_id": work_order.id,
                "request_id": work_order.request_id,
                "status": format!("{:?}", work_order.status)
            }),
        )?;
        self.events
            .publish("work_order.completed", &payload)
            .await?;
        Ok(work_order)
    }

    pub async fn complete_by_actor(
        &self,
        caller: &AuthUser,
        work_order_id: &str,
        actor_id: &str,
        scope: DataScope,
    ) -> Result<WorkOrder, DomainError> {
        rbac::require_any_role(caller, &["admin", "technician", "dispatcher", "supervisor"])?;
        let mut work_order = self
            .work_orders
            .get_by_id(work_order_id, scope.clone())
            .await?
            .ok_or(DomainError::NotFound("work_order"))?;
        let assignee = work_order
            .assignee
            .as_deref()
            .ok_or(DomainError::Forbidden("work order has no assignee"))?;
        if assignee != actor_id {
            return Err(DomainError::Forbidden(
                "technician can complete only their own work order",
            ));
        }
        work_order.complete()?;
        self.work_orders.update(work_order.clone(), scope.clone()).await?;
        let payload = make_event(
            "work_order.completed",
            work_order.id.clone(),
            work_order.owner_user_id.clone(),
            serde_json::json!({
                "work_order_id": work_order.id,
                "request_id": work_order.request_id,
                "status": format!("{:?}", work_order.status)
            }),
        )?;
        self.events
            .publish("work_order.completed", &payload)
            .await?;
        Ok(work_order)
    }
}
