use crate::domain::entities::WorkOrder;
use crate::domain::errors::DomainError;
use crate::ports::outbound::{EventPublisherPort, ServiceRequestRepository, WorkOrderRepository};

#[derive(Clone)]
pub struct WorkOrderAppService<R, W, E>
where
    R: ServiceRequestRepository,
    W: WorkOrderRepository,
    E: EventPublisherPort,
{
    pub requests: R,
    pub work_orders: W,
    pub events: E,
}

impl<R, W, E> WorkOrderAppService<R, W, E>
where
    R: ServiceRequestRepository,
    W: WorkOrderRepository,
    E: EventPublisherPort,
{
    pub fn create_work_order(&self, id: String, request_id: String) -> Result<WorkOrder, DomainError> {
        let _request = self
            .requests
            .get_by_id(&request_id)?
            .ok_or(DomainError::NotFound("service_request"))?;

        let work_order = WorkOrder::new(id, request_id)?;
        self.work_orders.save(work_order.clone())?;
        self.events.publish(
            "work_order.created",
            &format!("id={},request_id={}", work_order.id, work_order.request_id),
        )?;

        Ok(work_order)
    }

    pub fn list_by_request(&self, request_id: &str) -> Result<Vec<WorkOrder>, DomainError> {
        self.work_orders.list_by_request(request_id)
    }
}
