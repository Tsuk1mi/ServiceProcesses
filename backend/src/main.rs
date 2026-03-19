use service_processes_core::application::escalation_service::EscalationAppService;
use service_processes_core::application::service_request_service::ServiceRequestAppService;
use service_processes_core::application::technician_service::TechnicianAppService;
use service_processes_core::application::work_order_service::WorkOrderAppService;
use service_processes_core::domain::entities::{Asset, Technician};
use service_processes_core::domain::errors::DomainError;
use service_processes_core::infrastructure::in_memory::{
    BasicSlaPolicy, InMemoryAssetRepository, InMemoryEscalationRepository, InMemoryRequestRepository,
    InMemoryTechnicianRepository, InMemoryWorkOrderRepository, KeywordPriorityPolicy,
    StdoutEventPublisher,
};
use service_processes_core::interfaces::http::{router, AppState};
use service_processes_core::ports::outbound::{AssetRepository, TechnicianRepository};

#[tokio::main]
async fn main() -> Result<(), DomainError> {
    let assets = InMemoryAssetRepository::new();
    assets.save(Asset::new(
        "asset-1".to_string(),
        "building".to_string(),
        "Склад N1".to_string(),
        "Москва".to_string(),
    )?)?;
    let requests = InMemoryRequestRepository::new();
    let work_orders = InMemoryWorkOrderRepository::new();
    let escalations = InMemoryEscalationRepository::new();
    let technicians = InMemoryTechnicianRepository::new();
    technicians.save(Technician::new(
        "tech-1".to_string(),
        "Иван Иванов".to_string(),
        vec!["electrical".to_string(), "inspection".to_string()],
    )?)?;

    let service = ServiceRequestAppService {
        assets: assets.clone(),
        requests: requests.clone(),
        sla: BasicSlaPolicy,
        priority: KeywordPriorityPolicy,
        events: StdoutEventPublisher,
    };

    let state = AppState {
        assets,
        requests: requests.clone(),
        service,
        work_orders: work_orders.clone(),
        work_order_service: WorkOrderAppService {
            requests: requests.clone(),
            work_orders,
            technicians: technicians.clone(),
            events: StdoutEventPublisher,
        },
        escalations: escalations.clone(),
        escalation_service: EscalationAppService {
            requests: requests.clone(),
            escalations,
            events: StdoutEventPublisher,
        },
        technicians: technicians.clone(),
        technician_service: TechnicianAppService { technicians },
    };

    let app = router(state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("failed to bind 0.0.0.0:8080");

    println!("Backend API started at http://0.0.0.0:8080");
    axum::serve(listener, app)
        .await
        .expect("server terminated unexpectedly");
    Ok(())
}
