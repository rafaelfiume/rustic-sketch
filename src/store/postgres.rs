use crate::health_check::{
    service_status::{Dependency, DependencyStatus, Status},
    DependencyHealthChecker,
};
use async_trait::async_trait;
use sqlx::postgres::{PgPool, PgPoolOptions};

use derive_more::Constructor;
use derive_more::Display;
use derive_more::Error;
use getset::Getters;

#[derive(Clone, Constructor, Debug)]
pub struct DatabaseConfig {
    host: String,
    port: u16,
    name: String,
    user: String,
    password: String, // Secret type?
    db_pool_threads: u32,
}

pub struct PostgresStore {
    pool: PgPool,
}
impl PostgresStore {
    pub async fn new(config: DatabaseConfig) -> Result<Self, PostgresStoreError> {
        let connection_string = format!(
            "postgres://{}:{}@{}:{}/{}",
            config.user, config.password, config.host, config.port, config.name
        );

        let pool = PgPoolOptions::new()
            .max_connections(config.db_pool_threads)
            .connect(&connection_string)
            .await
            .map_err(|e| PostgresStoreError {
                message: e.to_string(),
            })?;

        Ok(PostgresStore { pool })
    }
}

// TODO Think about errors
#[derive(Debug, Display, Error, Getters)]
pub struct PostgresStoreError {
    #[getset(get)]
    message: String,
}

#[async_trait]
impl DependencyHealthChecker for PostgresStore {
    async fn check(&self) -> DependencyStatus {
        let status = match sqlx::query("SELECT 42").fetch_one(&self.pool).await {
            Ok(_) => Status::Ok,
            Err(_) => Status::Degraded,
        };
        DependencyStatus::new(Dependency::Database, status)
    }
}
