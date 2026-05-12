use std::sync::Arc;

use futures_util::StreamExt;
use lapin::{
    options::*,
    types::FieldTable,
    Connection, ConnectionProperties, ExchangeKind,
};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement};
use uuid::Uuid;

use crate::application::analytics_snapshot_service::AnalyticsSnapshotAppService;
use crate::domain::events::DomainEventEnvelope;
use crate::domain::errors::DomainError;
use crate::infrastructure::in_memory::InMemoryAnalyticsQuery;
use crate::infrastructure::jobs::DOMAIN_EVENTS_EXCHANGE;
use crate::infrastructure::metrics::AppMetrics;
use crate::infrastructure::postgres::{
    connect_and_migrate, PgAnalyticsSnapshotRepository, PgEscalationRepository, PgServiceRequestRepository,
    PgTechnicianRepository, PgWorkOrderRepository,
};
use crate::ports::outbound::{
    AnalyticsSnapshotRepository, EscalationRepository, ServiceRequestRepository, TechnicianRepository,
    WorkOrderRepository,
};

#[derive(Clone)]
pub struct ReadModelProjector {
    db: DatabaseConnection,
}

impl ReadModelProjector {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn apply_event(
        &self,
        event: &DomainEventEnvelope,
        analytics_snapshot_service: &AnalyticsSnapshotAppService,
        metrics: &AppMetrics,
    ) -> Result<(), DomainError> {
        metrics.record_worker(&format!("event:{}", event.event_type));
        if let Some(request_id) = event_request_id(event) {
            self.project_request(&request_id).await?;
        }
        analytics_snapshot_service.refresh(now_epoch()).await?;
        metrics.record_worker("projection_success");
        Ok(())
    }

    async fn project_request(&self, request_id: &str) -> Result<(), DomainError> {
        self.delete_existing(request_id, "read_request_list_item", "request_id").await?;
        self.delete_existing(request_id, "read_request_detail", "request_id").await?;
        self.delete_existing(request_id, "read_work_order_item", "request_id").await?;

        self.db
            .execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "INSERT INTO read_request_list_item (
                    request_id, asset_id, asset_title, asset_location, description, priority, status,
                    assignee, open_escalation_count, work_order_count, overdue, sla_deadline_epoch_sec,
                    created_at_epoch_sec, owner_user_id
                 )
                 SELECT
                    sr.id,
                    sr.asset_id,
                    a.title,
                    a.location,
                    sr.description,
                    sr.priority,
                    sr.status,
                    (
                        SELECT wo.assignee
                        FROM work_order wo
                        WHERE wo.request_id = sr.id
                        ORDER BY wo.id DESC
                        LIMIT 1
                    ) AS assignee,
                    (
                        SELECT COUNT(*)
                        FROM escalation esc
                        WHERE esc.request_id = sr.id AND esc.state = 'Open'
                    ) AS open_escalation_count,
                    (
                        SELECT COUNT(*)
                        FROM work_order wo
                        WHERE wo.request_id = sr.id
                    ) AS work_order_count,
                    CASE
                        WHEN sr.status IN ('Resolved', 'Closed') THEN FALSE
                        ELSE EXTRACT(EPOCH FROM NOW())::BIGINT > (sr.created_at_epoch_sec + (sr.sla_minutes * 60))
                    END AS overdue,
                    sr.created_at_epoch_sec + (sr.sla_minutes * 60) AS sla_deadline_epoch_sec,
                    sr.created_at_epoch_sec,
                    sr.owner_user_id
                 FROM service_request sr
                 JOIN asset a ON a.id = sr.asset_id
                 WHERE sr.id = $1",
                vec![request_id.to_string().into()],
            ))
            .await
            .map_err(|_| DomainError::EmptyField("read_model_projector"))?;

        self.db
            .execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "INSERT INTO read_request_detail (
                    request_id, asset_id, asset_title, asset_location, description, priority, status,
                    assignee, work_order_status, open_escalation_count, latest_escalation_reason,
                    overdue, sla_deadline_epoch_sec, created_at_epoch_sec, owner_user_id
                 )
                 SELECT
                    sr.id,
                    sr.asset_id,
                    a.title,
                    a.location,
                    sr.description,
                    sr.priority,
                    sr.status,
                    last_wo.assignee,
                    last_wo.status,
                    COALESCE(open_esc.open_count, 0) AS open_escalation_count,
                    last_esc.reason AS latest_escalation_reason,
                    CASE
                        WHEN sr.status IN ('Resolved', 'Closed') THEN FALSE
                        ELSE EXTRACT(EPOCH FROM NOW())::BIGINT > (sr.created_at_epoch_sec + (sr.sla_minutes * 60))
                    END AS overdue,
                    sr.created_at_epoch_sec + (sr.sla_minutes * 60) AS sla_deadline_epoch_sec,
                    sr.created_at_epoch_sec,
                    sr.owner_user_id
                 FROM service_request sr
                 JOIN asset a ON a.id = sr.asset_id
                 LEFT JOIN LATERAL (
                    SELECT wo.assignee, wo.status
                    FROM work_order wo
                    WHERE wo.request_id = sr.id
                    ORDER BY wo.id DESC
                    LIMIT 1
                 ) last_wo ON TRUE
                 LEFT JOIN LATERAL (
                    SELECT COUNT(*) AS open_count
                    FROM escalation esc
                    WHERE esc.request_id = sr.id AND esc.state = 'Open'
                 ) open_esc ON TRUE
                 LEFT JOIN LATERAL (
                    SELECT esc.reason
                    FROM escalation esc
                    WHERE esc.request_id = sr.id
                    ORDER BY esc.id DESC
                    LIMIT 1
                 ) last_esc ON TRUE
                 WHERE sr.id = $1",
                vec![request_id.to_string().into()],
            ))
            .await
            .map_err(|_| DomainError::EmptyField("read_model_projector"))?;

        self.db
            .execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "INSERT INTO read_work_order_item (
                    work_order_id, request_id, assignee, assignee_name, status, owner_user_id
                 )
                 SELECT
                    wo.id,
                    wo.request_id,
                    wo.assignee,
                    tech.full_name,
                    wo.status,
                    wo.owner_user_id
                 FROM work_order wo
                 LEFT JOIN technician tech ON tech.id = wo.assignee
                 WHERE wo.request_id = $1",
                vec![request_id.to_string().into()],
            ))
            .await
            .map_err(|_| DomainError::EmptyField("read_model_projector"))?;

        Ok(())
    }

    async fn delete_existing(
        &self,
        entity_id: &str,
        table: &str,
        key_column: &str,
    ) -> Result<(), DomainError> {
        self.db
            .execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                format!("DELETE FROM {table} WHERE {key_column} = $1"),
                vec![entity_id.to_string().into()],
            ))
            .await
            .map_err(|_| DomainError::EmptyField("read_model_projector"))?;
        Ok(())
    }

    pub async fn store_error(&self, event: &DomainEventEnvelope, message: &str) -> Result<(), DomainError> {
        let payload = serde_json::to_value(event).map_err(|_| DomainError::EmptyField("projection_error"))?;
        self.db
            .execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "INSERT INTO read_projection_error (
                    id, event_id, event_type, entity_id, error_message, payload, created_at_epoch_sec
                 ) VALUES ($1, $2, $3, $4, $5, $6, $7)",
                vec![
                    Uuid::new_v4().into(),
                    event.event_id.into(),
                    event.event_type.clone().into(),
                    event.entity_id.clone().into(),
                    message.to_string().into(),
                    payload.into(),
                    (now_epoch() as i64).into(),
                ],
            ))
            .await
            .map_err(|_| DomainError::EmptyField("projection_error"))?;
        Ok(())
    }
}

pub async fn run_read_model_worker(
    database_url: &str,
    amqp_url: &str,
    queue_name: &str,
    metrics: AppMetrics,
) -> Result<(), DomainError> {
    let db = connect_and_migrate(database_url).await?;
    let projector = ReadModelProjector::new(db.clone());
    let requests: Arc<dyn ServiceRequestRepository> = Arc::new(PgServiceRequestRepository::new(db.clone()));
    let work_orders: Arc<dyn WorkOrderRepository> = Arc::new(PgWorkOrderRepository::new(db.clone()));
    let escalations: Arc<dyn EscalationRepository> = Arc::new(PgEscalationRepository::new(db.clone()));
    let technicians: Arc<dyn TechnicianRepository> = Arc::new(PgTechnicianRepository::new(db.clone()));
    let snapshots: Arc<dyn AnalyticsSnapshotRepository> = Arc::new(PgAnalyticsSnapshotRepository::new(db));
    let analytics_snapshot_service = AnalyticsSnapshotAppService {
        analytics: Arc::new(InMemoryAnalyticsQuery {
            requests: requests.clone(),
            work_orders: work_orders.clone(),
            escalations: escalations.clone(),
            technicians: technicians.clone(),
        }),
        snapshots,
    };

    let rabbit = Connection::connect(amqp_url, ConnectionProperties::default())
        .await
        .map_err(|_| DomainError::EmptyField("rabbitmq"))?;
    let channel = rabbit
        .create_channel()
        .await
        .map_err(|_| DomainError::EmptyField("rabbitmq"))?;
    channel
        .exchange_declare(
            DOMAIN_EVENTS_EXCHANGE,
            ExchangeKind::Topic,
            ExchangeDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await
        .map_err(|_| DomainError::EmptyField("rabbitmq"))?;
    channel
        .queue_declare(
            queue_name,
            QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await
        .map_err(|_| DomainError::EmptyField("rabbitmq"))?;
    channel
        .queue_bind(
            queue_name,
            DOMAIN_EVENTS_EXCHANGE,
            "#",
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(|_| DomainError::EmptyField("rabbitmq"))?;

    let mut consumer = channel
        .basic_consume(
            queue_name,
            "read_model_worker",
            BasicConsumeOptions {
                no_ack: false,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await
        .map_err(|_| DomainError::EmptyField("rabbitmq"))?;

    while let Some(delivery) = consumer.next().await {
        let delivery = delivery.map_err(|_| DomainError::EmptyField("rabbitmq"))?;
        metrics.record_worker("delivery_received");
        let parsed = serde_json::from_slice::<DomainEventEnvelope>(&delivery.data);
        match parsed {
            Ok(event) => {
                if let Err(error) = projector
                    .apply_event(&event, &analytics_snapshot_service, &metrics)
                    .await
                {
                    tracing::error!(event_type = %event.event_type, error = %error, "read model projection failed");
                    metrics.record_worker("projection_error");
                    let _ = projector.store_error(&event, &error.to_string()).await;
                }
            }
            Err(error) => {
                tracing::error!(error = %error, "failed to parse domain event");
                metrics.record_worker("event_parse_error");
            }
        }
        delivery
            .ack(BasicAckOptions::default())
            .await
            .map_err(|_| DomainError::EmptyField("rabbitmq"))?;
    }

    Ok(())
}

fn event_request_id(event: &DomainEventEnvelope) -> Option<String> {
    if event.event_type.starts_with("service_request.") {
        return Some(event.entity_id.clone());
    }
    event.payload
        .get("request_id")
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
}

fn now_epoch() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
