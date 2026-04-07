use std::env;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use service_processes_core::application::analytics_snapshot_service::AnalyticsSnapshotAppService;
use service_processes_core::application::audit_service::AuditAppService;
use service_processes_core::application::escalation_service::EscalationAppService;
use service_processes_core::application::reporting_service::ReportingAppService;
use service_processes_core::application::service_request_service::ServiceRequestAppService;
use service_processes_core::application::sla_service::SlaAppService;
use service_processes_core::application::technician_service::TechnicianAppService;
use service_processes_core::application::work_order_service::WorkOrderAppService;
use service_processes_core::auth::{InMemoryUserStore, UserStore};
use service_processes_core::domain::entities::{Asset, Technician};
use service_processes_core::domain::errors::DomainError;
use service_processes_core::infrastructure::in_memory::{
    BasicSlaPolicy, InMemoryAnalyticsQuery, InMemoryAnalyticsSnapshotRepository, InMemoryAssetRepository,
    InMemoryAuditRepository, InMemoryEscalationRepository, InMemoryRequestRepository,
    InMemoryTechnicianRepository, InMemoryWorkOrderRepository, KeywordPriorityPolicy, StdoutEventPublisher,
};
use service_processes_core::infrastructure::jobs::{run_worker, JobClient, JobClientEventPublisher};
use service_processes_core::infrastructure::postgres::{
    connect_and_migrate, seed_demo_domain_if_empty, seed_users_if_empty, PgAnalyticsSnapshotRepository,
    PgAssetRepository, PgAuditRepository, PgEscalationRepository, PgServiceRequestRepository,
    PgTechnicianRepository, PgUserStore, PgWorkOrderRepository,
};
use service_processes_core::infrastructure::redis_cache::RedisCache;
use service_processes_core::interfaces::http::{router, AppState};
use service_processes_core::ports::outbound::{
    AnalyticsSnapshotRepository, AssetRepository, AuditRepository, EscalationRepository, EventPublisherPort,
    ServiceRequestRepository, TechnicianRepository, WorkOrderRepository,
};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), DomainError> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
    dotenvy::dotenv().ok();
    let mode = env::var("APP_MODE").unwrap_or_else(|_| "api".to_string());
    if mode.eq_ignore_ascii_case("worker") {
        return run_sla_worker().await;
    }
    if mode.eq_ignore_ascii_case("queue_worker") {
        return run_queue_worker().await;
    }
    run_api().await
}

async fn run_api() -> Result<(), DomainError> {
    let state = build_state().await?;
    let app = router(state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("failed to bind 0.0.0.0:8080");

    tracing::info!("Backend API at http://0.0.0.0:8080");
    tracing::info!("OpenAPI JSON: http://0.0.0.0:8080/api-docs/openapi.json");
    tracing::info!("Swagger UI:  http://0.0.0.0:8080/swagger-ui/");
    axum::serve(listener, app)
        .await
        .expect("server terminated unexpectedly");
    Ok(())
}

async fn run_queue_worker() -> Result<(), DomainError> {
    let redis_url = env::var("REDIS_URL").map_err(|_| DomainError::EmptyField("REDIS_URL"))?;
    let amqp_url = env::var("RABBITMQ_URL").map_err(|_| DomainError::EmptyField("RABBITMQ_URL"))?;
    let queue_name = env::var("JOB_QUEUE_NAME").unwrap_or_else(|_| "service_jobs".to_string());
    tracing::info!(queue = %queue_name, "queue worker (RabbitMQ consumer) starting");
    run_worker(&redis_url, &amqp_url, &queue_name)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "queue worker terminated");
            DomainError::EmptyField("queue_worker")
        })
}

async fn run_sla_worker() -> Result<(), DomainError> {
    let state = build_state().await?;
    let interval_sec = env::var("WORKER_INTERVAL_SEC")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(30);

    tracing::info!(interval_sec, "SLA worker started");
    loop {
        let created = state
            .sla_service
            .auto_escalate_overdue(now_epoch(), "Automatic SLA overdue escalation by worker")
            .await?;
        for esc in created {
            let audit_owner = esc.owner_user_id.clone();
            let _ = state
                .audit_service
                .record(
                    Some(esc.request_id.clone()),
                    "escalation",
                    "auto_create_overdue_worker",
                    "system",
                    Some("sla-worker".to_string()),
                    format!("escalation_id={}", esc.id),
                    audit_owner,
                )
                .await;
        }

        state.analytics_snapshot_service.refresh(now_epoch()).await?;
        sleep(Duration::from_secs(interval_sec)).await;
    }
}

async fn build_state() -> Result<AppState, DomainError> {
    let db_url = env::var("DATABASE_URL").unwrap_or_default();
    if !db_url.is_empty() {
        tracing::info!("хранилище: PostgreSQL (DATABASE_URL); Redis+RabbitMQ обязательны для API");
        return build_state_pg(&db_url).await;
    }
    tracing::info!("хранилище: in-memory (без DATABASE_URL)");
    build_state_memory().await
}

async fn build_state_memory() -> Result<AppState, DomainError> {
    let jwt_secret = Arc::new(
        env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-me-please".to_string()),
    );
    let users: Arc<dyn UserStore> = Arc::new(InMemoryUserStore::demo()?);
    let admin_owner = InMemoryUserStore::demo_admin_id().to_string();
    let tech_id = InMemoryUserStore::demo_technician_id().to_string();

    let assets_mem = Arc::new(InMemoryAssetRepository::new());
    let assets: Arc<dyn AssetRepository> = assets_mem.clone();
    assets_mem
        .save(Asset::new(
            "asset-1".to_string(),
            "building".to_string(),
            "Склад N1".to_string(),
            "Москва".to_string(),
            admin_owner.clone(),
        )?)
        .await?;

    let requests = Arc::new(InMemoryRequestRepository::new(Arc::clone(&assets_mem)));
    let requests_dyn: Arc<dyn ServiceRequestRepository> = requests.clone();
    let work_orders = Arc::new(InMemoryWorkOrderRepository::new());
    let work_orders_dyn: Arc<dyn WorkOrderRepository> = work_orders.clone();
    let escalations = Arc::new(InMemoryEscalationRepository::new());
    let escalations_dyn: Arc<dyn EscalationRepository> = escalations.clone();
    let technicians = Arc::new(InMemoryTechnicianRepository::new());
    let technicians_dyn: Arc<dyn TechnicianRepository> = technicians.clone();
    let audit = Arc::new(InMemoryAuditRepository::new());
    let audit_dyn: Arc<dyn AuditRepository> = audit.clone();

    technicians
        .save(Technician::new(
            tech_id.clone(),
            "Иван Иванов".to_string(),
            vec!["electrical".to_string(), "inspection".to_string()],
            admin_owner.clone(),
        )?)
        .await?;

    let queue_name = env::var("JOB_QUEUE_NAME").unwrap_or_else(|_| "service_jobs".to_string());
    let jobs = match (env::var("REDIS_URL"), env::var("RABBITMQ_URL")) {
        (Ok(ref redis_url), Ok(ref amqp_url)) => match JobClient::connect(redis_url, amqp_url, &queue_name).await {
            Ok(client) => {
                tracing::info!("Redis + RabbitMQ: задачи, кэш GET и доменные события включены");
                Some(Arc::new(client))
            }
            Err(e) => {
                tracing::warn!(error = %e, "Redis/RabbitMQ недоступны — без кэша, событий в AMQP и /api/v1/jobs");
                None
            }
        },
        _ => {
            tracing::info!("задайте REDIS_URL и RABBITMQ_URL для задач, кэша и событий в RabbitMQ");
            None
        }
    };

    let events_pub: Arc<dyn EventPublisherPort> = match &jobs {
        Some(j) => Arc::new(JobClientEventPublisher::new(j.clone())),
        None => Arc::new(StdoutEventPublisher),
    };
    let redis_cache = jobs
        .as_ref()
        .map(|j| Arc::new(RedisCache::from_manager(j.redis_conn())));

    let service = ServiceRequestAppService {
        assets: assets.clone(),
        requests: requests_dyn.clone(),
        sla: Arc::new(BasicSlaPolicy),
        priority: Arc::new(KeywordPriorityPolicy),
        events: events_pub.clone(),
    };

    let analytics_query = Arc::new(InMemoryAnalyticsQuery {
        requests: requests_dyn.clone(),
        work_orders: work_orders_dyn.clone(),
        escalations: escalations_dyn.clone(),
        technicians: technicians_dyn.clone(),
    });
    let snapshots: Arc<dyn AnalyticsSnapshotRepository> = Arc::new(InMemoryAnalyticsSnapshotRepository::new());
    let analytics_snapshot_service = AnalyticsSnapshotAppService {
        analytics: analytics_query.clone(),
        snapshots: snapshots.clone(),
    };

    Ok(AppState {
        assets,
        requests: requests_dyn.clone(),
        service,
        work_orders: work_orders_dyn.clone(),
        work_order_service: WorkOrderAppService {
            requests: requests_dyn.clone(),
            work_orders: work_orders_dyn.clone(),
            technicians: technicians_dyn.clone(),
            events: events_pub.clone(),
        },
        escalations: escalations_dyn.clone(),
        escalation_service: EscalationAppService {
            requests: requests_dyn.clone(),
            escalations: escalations_dyn.clone(),
            events: events_pub.clone(),
        },
        technicians: technicians_dyn.clone(),
        technician_service: TechnicianAppService {
            technicians: technicians_dyn.clone(),
        },
        audit: audit_dyn.clone(),
        audit_service: AuditAppService { audit: audit_dyn },
        sla_service: SlaAppService {
            requests: requests_dyn.clone(),
            escalations: escalations_dyn.clone(),
            events: events_pub.clone(),
        },
        reporting_service: ReportingAppService {
            analytics: analytics_query.clone(),
        },
        analytics_snapshot_service,
        jwt_secret,
        users,
        jobs,
        redis_cache,
    })
}

async fn build_state_pg(database_url: &str) -> Result<AppState, DomainError> {
    let jwt_secret = Arc::new(
        env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-me-please".to_string()),
    );
    let redis_url = env::var("REDIS_URL").map_err(|_| DomainError::EmptyField("REDIS_URL"))?;
    let amqp_url = env::var("RABBITMQ_URL").map_err(|_| DomainError::EmptyField("RABBITMQ_URL"))?;
    let queue_name = env::var("JOB_QUEUE_NAME").unwrap_or_else(|_| "service_jobs".to_string());

    let jobs = Arc::new(
        JobClient::connect(&redis_url, &amqp_url, &queue_name)
            .await
            .map_err(|_| DomainError::EmptyField("redis_rabbit_connect"))?,
    );
    let redis_cache = Some(Arc::new(RedisCache::from_manager(jobs.redis_conn())));
    let events_pub: Arc<dyn EventPublisherPort> = Arc::new(JobClientEventPublisher::new(jobs.clone()));

    let db = connect_and_migrate(database_url).await?;
    seed_users_if_empty(&db).await?;
    let admin_owner = PgUserStore::demo_admin_subject().to_string();
    seed_demo_domain_if_empty(&db, admin_owner).await?;

    let assets: Arc<dyn AssetRepository> = Arc::new(PgAssetRepository::new(db.clone()));
    let requests: Arc<dyn ServiceRequestRepository> = Arc::new(PgServiceRequestRepository::new(db.clone()));
    let work_orders: Arc<dyn WorkOrderRepository> = Arc::new(PgWorkOrderRepository::new(db.clone()));
    let escalations: Arc<dyn EscalationRepository> = Arc::new(PgEscalationRepository::new(db.clone()));
    let technicians: Arc<dyn TechnicianRepository> = Arc::new(PgTechnicianRepository::new(db.clone()));
    let audit: Arc<dyn AuditRepository> = Arc::new(PgAuditRepository::new(db.clone()));
    let users: Arc<dyn UserStore> = Arc::new(PgUserStore::new(db.clone()));

    let service = ServiceRequestAppService {
        assets: assets.clone(),
        requests: requests.clone(),
        sla: Arc::new(BasicSlaPolicy),
        priority: Arc::new(KeywordPriorityPolicy),
        events: events_pub.clone(),
    };

    let analytics_query = Arc::new(InMemoryAnalyticsQuery {
        requests: requests.clone(),
        work_orders: work_orders.clone(),
        escalations: escalations.clone(),
        technicians: technicians.clone(),
    });
    let snapshots: Arc<dyn AnalyticsSnapshotRepository> = Arc::new(PgAnalyticsSnapshotRepository::new(db));
    let analytics_snapshot_service = AnalyticsSnapshotAppService {
        analytics: analytics_query.clone(),
        snapshots: snapshots.clone(),
    };

    Ok(AppState {
        assets,
        requests: requests.clone(),
        service,
        work_orders: work_orders.clone(),
        work_order_service: WorkOrderAppService {
            requests: requests.clone(),
            work_orders: work_orders.clone(),
            technicians: technicians.clone(),
            events: events_pub.clone(),
        },
        escalations: escalations.clone(),
        escalation_service: EscalationAppService {
            requests: requests.clone(),
            escalations: escalations.clone(),
            events: events_pub.clone(),
        },
        technicians: technicians.clone(),
        technician_service: TechnicianAppService {
            technicians: technicians.clone(),
        },
        audit: audit.clone(),
        audit_service: AuditAppService { audit: audit.clone() },
        sla_service: SlaAppService {
            requests: requests.clone(),
            escalations: escalations.clone(),
            events: events_pub.clone(),
        },
        reporting_service: ReportingAppService {
            analytics: analytics_query.clone(),
        },
        analytics_snapshot_service,
        jwt_secret,
        users,
        jobs: Some(jobs),
        redis_cache,
    })
}

fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
