//! Кэш JSON-ответов GET `/api/v1/*` в Redis (инвалидация при любой мутации).

use redis::AsyncCommands;
use tracing::warn;

#[derive(Clone)]
pub struct RedisCache {
    redis: redis::aio::ConnectionManager,
}

impl RedisCache {
    pub fn from_manager(redis: redis::aio::ConnectionManager) -> Self {
        Self { redis }
    }

    pub async fn get(&self, key: &str) -> Option<Vec<u8>> {
        let mut conn = self.redis.clone();
        match conn.get::<_, Option<Vec<u8>>>(key).await {
            Ok(v) => v,
            Err(e) => {
                warn!(error = %e, key = %key, "redis cache get failed");
                None
            }
        }
    }

    pub async fn set(&self, key: &str, value: &[u8], ttl_secs: u64) {
        let mut conn = self.redis.clone();
        let r: Result<(), redis::RedisError> = redis::cmd("SET")
            .arg(key)
            .arg(value)
            .arg("EX")
            .arg(ttl_secs)
            .query_async(&mut conn)
            .await;
        if let Err(e) = r {
            warn!(error = %e, key = %key, "redis cache set failed");
        }
    }

    /// Сброс кэша API после POST/PUT/DELETE.
    pub async fn invalidate_api_cache(&self) {
        let mut conn = self.redis.clone();
        let keys: Result<Vec<String>, _> = redis::cmd("KEYS")
            .arg("cache:api:v1:*")
            .query_async(&mut conn)
            .await;
        let Ok(keys) = keys else {
            return;
        };
        if keys.is_empty() {
            return;
        }
        if let Err(e) = conn.del::<_, ()>(&keys).await {
            warn!(error = %e, "redis cache invalidate failed");
        }
    }
}
