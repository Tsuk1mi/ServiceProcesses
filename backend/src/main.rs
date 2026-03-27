use service_processes_core::application::audit_service::AuditAppService;
use service_processes_core::application::escalation_service::EscalationAppService;
use service_processes_core::application::analytics_snapshot_service::AnalyticsSnapshotAppService;
use service_processes_core::application::reporting_service::ReportingAppService;
use service_processes_core::application::service_request_service::ServiceRequestAppService;
use service_processes_core::application::sla_service::SlaAppService;
use service_processes_core::application::technician_service::TechnicianAppService;
use service_processes_core::application::work_order_service::WorkOrderAppService;
use service_processes_core::domain::entities::{Asset, Technician};
use service_processes_core::domain::errors::DomainError;
use service_processes_core::infrastructure::in_memory::{
    BasicSlaPolicy, InMemoryAssetRepository, InMemoryEscalationRepository, InMemoryRequestRepository,
    InMemoryAnalyticsQuery, InMemoryAnalyticsSnapshotRepository, InMemoryAuditRepository,
    InMemoryTechnicianRepository,
    InMemoryWorkOrderRepository, KeywordPriorityPolicy, StdoutEventPublisher,
};
use service_processes_core::interfaces::http::{router, AppState};
use service_processes_core::ports::outbound::{AssetRepository, TechnicianRepository};
use std::env;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), DomainError> {
    let mode = env::var("APP_MODE").unwrap_or_else(|_| "api".to_string());
    if mode.eq_ignore_ascii_case("worker") {
        return run_sla_worker().await;
    }
    run_api().await
}

async fn run_api() -> Result<(), DomainError> {
    let state = build_state()?;
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

async fn run_sla_worker() -> Result<(), DomainError> {
    let state = build_state()?;
    let interval_sec = env::var("WORKER_INTERVAL_SEC")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(30);

    println!("SLA worker started, interval={}s", interval_sec);
    loop {
        let created = state
            .sla_service
            .auto_escalate_overdue(now_epoch(), "Automatic SLA overdue escalation by worker")?;
        for esc in created {
            let _ = state.audit_service.record(
                Some(esc.request_id.clone()),
                "escalation",
                "auto_create_overdue_worker",
                "system",
                Some("sla-worker".to_string()),
                format!("escalation_id={}", esc.id),
            );
        }

        // Update cached analytics snapshot for UI dashboards.
        state
            .analytics_snapshot_service
            .refresh(now_epoch())?;
        sleep(Duration::from_secs(interval_sec)).await;
    }
}

fn build_state() -> Result<AppState, DomainError> {
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
    let audit = InMemoryAuditRepository::new();
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

    let analytics_query = InMemoryAnalyticsQuery {
        requests: requests.clone(),
        work_orders: work_orders.clone(),
        escalations: escalations.clone(),
        technicians: technicians.clone(),
    };
    let analytics_snapshots = InMemoryAnalyticsSnapshotRepository::new();
    let analytics_snapshot_service = AnalyticsSnapshotAppService {
        analytics: analytics_query.clone(),
        snapshots: analytics_snapshots,
    };

    let state = AppState {
        assets,
        requests: requests.clone(),
        service,
        work_orders: work_orders.clone(),
        work_order_service: WorkOrderAppService {
            requests: requests.clone(),
            work_orders: work_orders.clone(),
            technicians: technicians.clone(),
            events: StdoutEventPublisher,
        },
        escalations: escalations.clone(),
        escalation_service: EscalationAppService {
            requests: requests.clone(),
            escalations: escalations.clone(),
            events: StdoutEventPublisher,
        },
        technicians: technicians.clone(),
        technician_service: TechnicianAppService {
            technicians: technicians.clone(),
        },
        audit: audit.clone(),
        audit_service: AuditAppService { audit },
        sla_service: SlaAppService {
            requests: requests.clone(),
            escalations: escalations.clone(),
            events: StdoutEventPublisher,
        },
        reporting_service: ReportingAppService {
            analytics: analytics_query.clone(),
        },
        analytics_snapshot_service,
    };
    Ok(state)
}

fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
