//! Очередь RabbitMQ + статус задач в Redis (ключ `job:status:{uuid}`).

use std::sync::Arc;

use futures_util::StreamExt;
use lapin::{
    options::*,
    types::FieldTable,
    BasicProperties, Connection, ConnectionProperties,
};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;

const JOB_KEY_PREFIX: &str = "job:status:";
const JOB_TTL_SECS: i64 = 604800; // 7 дней

#[derive(Debug, Error)]
pub enum JobError {
    #[error("redis: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("amqp: {0}")]
    Amqp(#[from] lapin::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Message(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobEnvelope {
    pub id: Uuid,
    pub kind: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct JobStatusRecord {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub owner_user_id: String,
}

#[derive(Clone)]
pub struct JobClient {
    redis: redis::aio::ConnectionManager,
    rabbit: Arc<Connection>,
    queue_name: String,
}

impl JobClient {
    pub async fn connect(redis_url: &str, amqp_url: &str, queue_name: &str) -> Result<Self, JobError> {
        let client = redis::Client::open(redis_url)?;
        let redis = redis::aio::ConnectionManager::new(client).await?;
        let rabbit = Arc::new(Connection::connect(amqp_url, ConnectionProperties::default()).await?);
        let channel = rabbit.create_channel().await?;
        channel
            .queue_declare(
                queue_name,
                QueueDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await?;

        Ok(Self {
            redis,
            rabbit,
            queue_name: queue_name.to_string(),
        })
    }

    fn key(id: &Uuid) -> String {
        format!("{JOB_KEY_PREFIX}{id}")
    }

    async fn write_record(&self, id: &Uuid, record: &JobStatusRecord) -> Result<(), JobError> {
        let key = Self::key(id);
        let json = serde_json::to_string(record)?;
        let mut conn = self.redis.clone();
        let _: () = conn.set(&key, json).await?;
        let _: () = conn.expire(&key, JOB_TTL_SECS).await?;
        Ok(())
    }

    /// Ставит задачу в Redis (queued) и публикует в RabbitMQ. Мгновенный ответ API.
    pub async fn enqueue(
        &self,
        kind: String,
        payload: serde_json::Value,
        owner_user_id: String,
    ) -> Result<Uuid, JobError> {
        let id = Uuid::new_v4();
        let record = JobStatusRecord {
            status: "queued".to_string(),
            result: None,
            error: None,
            owner_user_id,
        };
        self.write_record(&id, &record).await?;

        let envelope = JobEnvelope {
            id,
            kind,
            payload,
        };
        let body = serde_json::to_vec(&envelope)?;

        let channel = self.rabbit.create_channel().await?;
        channel
            .basic_publish(
                "",
                &self.queue_name,
                BasicPublishOptions::default(),
                &body,
                BasicProperties::default().with_delivery_mode(2),
            )
            .await?
            .await?;

        Ok(id)
    }

    pub async fn get_status(&self, id: &Uuid) -> Result<Option<JobStatusRecord>, JobError> {
        let key = Self::key(id);
        let mut conn = self.redis.clone();
        let raw: Option<String> = conn.get(key).await?;
        let Some(s) = raw else {
            return Ok(None);
        };
        Ok(Some(serde_json::from_str(&s)?))
    }
}

/// Воркер: читает очередь, обновляет статусы в Redis.
pub async fn run_worker(redis_url: &str, amqp_url: &str, queue_name: &str) -> Result<(), JobError> {
    let client = redis::Client::open(redis_url)?;
    let redis = redis::aio::ConnectionManager::new(client).await?;
    let conn = Connection::connect(amqp_url, ConnectionProperties::default()).await?;
    let channel = conn.create_channel().await?;
    channel
        .queue_declare(
            queue_name,
            QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;

    let mut consumer = channel
        .basic_consume(
            queue_name,
            "service_jobs_worker",
            BasicConsumeOptions {
                no_ack: false,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;

    tracing::info!(queue = %queue_name, "RabbitMQ consumer started");

    while let Some(delivery) = consumer.next().await {
        let delivery = delivery?;
        match process_one_delivery(&redis, &delivery).await {
            Ok(()) => {
                delivery.ack(BasicAckOptions::default()).await?;
            }
            Err(e) => {
                tracing::warn!(error = %e, "job failed, nack without requeue");
                delivery
                    .nack(BasicNackOptions {
                        requeue: false,
                        ..Default::default()
                    })
                    .await?;
            }
        }
    }

    Ok(())
}

async fn patch_job_record(
    redis: &redis::aio::ConnectionManager,
    id: &Uuid,
    status: &str,
    result: Option<serde_json::Value>,
    error: Option<String>,
) -> Result<(), JobError> {
    let key = format!("{JOB_KEY_PREFIX}{id}");
    let mut conn = redis.clone();
    let raw: Option<String> = conn.get(&key).await?;
    let Some(raw) = raw else {
        return Err(JobError::Message("job record missing in redis".into()));
    };
    let mut rec: JobStatusRecord = serde_json::from_str(&raw)?;
    rec.status = status.to_string();
    rec.result = result;
    rec.error = error;
    let json = serde_json::to_string(&rec)?;
    let _: () = conn.set(&key, json).await?;
    let _: () = conn.expire(&key, JOB_TTL_SECS).await?;
    Ok(())
}

async fn process_one_delivery(
    redis: &redis::aio::ConnectionManager,
    delivery: &lapin::message::Delivery,
) -> Result<(), JobError> {
    let env: JobEnvelope = serde_json::from_slice(&delivery.data)?;
    let id = env.id;

    patch_job_record(redis, &id, "processing", None, None).await?;

    let outcome = match env.kind.as_str() {
        "echo" => {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            Ok(serde_json::json!({ "echo": env.payload }))
        }
        "simulate_slow" => {
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            Ok(serde_json::json!({ "done": true }))
        }
        other => Err(JobError::Message(format!("unknown job kind: {other}"))),
    };

    match outcome {
        Ok(val) => {
            patch_job_record(redis, &id, "completed", Some(val), None).await?;
        }
        Err(e) => {
            let msg = e.to_string();
            let _ = patch_job_record(redis, &id, "failed", None, Some(msg.clone())).await;
            return Err(e);
        }
    }

    Ok(())
}
