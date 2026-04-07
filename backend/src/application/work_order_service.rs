use crate::domain::entities::WorkOrder;
use crate::domain::errors::DomainError;
use crate::ports::data_scope::DataScope;
use crate::ports::outbound::{
    EventPublisherPort, ServiceRequestRepository, TechnicianRepository, WorkOrderRepository,
};

#[derive(Clone)]
pub struct WorkOrderAppService<R, W, T, E>
where
    R: ServiceRequestRepository,
    W: WorkOrderRepository,
    T: TechnicianRepository,
    E: EventPublisherPort,
{
    pub requests: R,
    pub work_orders: W,
    pub technicians: T,
    pub events: E,
}

impl<R, W, T, E> WorkOrderAppService<R, W, T, E>
where
    R: ServiceRequestRepository + Send + Sync,
    W: WorkOrderRepository + Send + Sync,
    T: TechnicianRepository + Send + Sync,
    E: EventPublisherPort + Send + Sync,
{
    pub async fn create_work_order(
        &self,
        id: String,
        request_id: String,
        scope: DataScope,
    ) -> Result<WorkOrder, DomainError> {
        let request = self
            .requests
            .get_by_id(&request_id, scope.clone())
            .await?
            .ok_or(DomainError::NotFound("service_request"))?;

        let work_order = WorkOrder::new(id, request_id, request.owner_user_id)?;
        self.work_orders.save(work_order.clone()).await?;
        self.events
            .publish(
                "work_order.created",
                &format!("id={},request_id={}", work_order.id, work_order.request_id),
            )
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
        work_order_id: &str,
        assignee: String,
        scope: DataScope,
    ) -> Result<WorkOrder, DomainError> {
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
        self.work_orders.update(work_order.clone()).await?;
        self.events
            .publish(
                "work_order.assigned",
                &format!(
                    "id={},assignee={}",
                    work_order.id,
                    work_order.assignee.clone().unwrap_or_default()
                ),
            )
            .await?;
        Ok(work_order)
    }

    pub async fn start(&self, work_order_id: &str, scope: DataScope) -> Result<WorkOrder, DomainError> {
        let mut work_order = self
            .work_orders
            .get_by_id(work_order_id, scope.clone())
            .await?
            .ok_or(DomainError::NotFound("work_order"))?;
        work_order.start()?;
        self.work_orders.update(work_order.clone()).await?;
        self.events
            .publish("work_order.started", &format!("id={}", work_order.id))
            .await?;
        Ok(work_order)
    }

    pub async fn start_by_actor(
        &self,
        work_order_id: &str,
        actor_id: &str,
        scope: DataScope,
    ) -> Result<WorkOrder, DomainError> {
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
        self.work_orders.update(work_order.clone()).await?;
        self.events
            .publish("work_order.started", &format!("id={}", work_order.id))
            .await?;
        Ok(work_order)
    }

    pub async fn complete(&self, work_order_id: &str, scope: DataScope) -> Result<WorkOrder, DomainError> {
        let mut work_order = self
            .work_orders
            .get_by_id(work_order_id, scope.clone())
            .await?
            .ok_or(DomainError::NotFound("work_order"))?;
        work_order.complete()?;
        self.work_orders.update(work_order.clone()).await?;
        self.events
            .publish("work_order.completed", &format!("id={}", work_order.id))
            .await?;
        Ok(work_order)
    }

    pub async fn complete_by_actor(
        &self,
        work_order_id: &str,
        actor_id: &str,
        scope: DataScope,
    ) -> Result<WorkOrder, DomainError> {
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
        self.work_orders.update(work_order.clone()).await?;
        self.events
            .publish("work_order.completed", &format!("id={}", work_order.id))
            .await?;
        Ok(work_order)
    }
}
