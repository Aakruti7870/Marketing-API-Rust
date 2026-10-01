use crate::config::Config;
use redis::aio::ConnectionManager;
use reqwest::Client;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
    pub http_client: Client,
    pub redis: Option<ConnectionManager>,
}

impl AppState {
    pub fn new(pool: PgPool, config: Config) -> Self {
        Self {
            pool,
            config: Arc::new(config),
            http_client: Client::builder().build().unwrap_or_default(),
            redis: None,
        }
    }

    pub fn with_redis(mut self, redis: Option<ConnectionManager>) -> Self {
        self.redis = redis;
        self
    }
}
