use std::collections::HashMap;

use getset::Getters;
use rustic_sketch::health_check::{service_status::Status, DependencyHealthChecker};
use rustic_sketch::store::postgres;
use testcontainers::{
    clients::{self},
    core::{Image, WaitFor},
};

#[tokio::test]
async fn dependency_status_is_ok_when_database_is_available() {
    let docker = clients::Cli::default();
    let postgres = Postgres::default();
    let container = docker.run(postgres.clone());
    let config = postgres::DatabaseConfig::new(
        "0.0.0.0".to_string(),
        container.get_host_port_ipv4(*postgres.port()),
        postgres.name().to_string(),
        postgres.user().to_string(),
        postgres.password().to_string(),
        5,
    );

    let store = postgres::PostgresStore::new(config).await.unwrap();
    let result = store.check().await;

    assert_eq!(*result.status(), Status::Ok);
}

#[tokio::test]
async fn dependency_status_is_degraded_when_database_is_not_available() {
    let docker = clients::Cli::default();
    let postgres = Postgres::default();
    let container = docker.run(postgres.clone());
    let config = postgres::DatabaseConfig::new(
        "0.0.0.0".to_string(),
        container.get_host_port_ipv4(*postgres.port()),
        postgres.name().to_string(),
        postgres.user().to_string(),
        postgres.password().to_string(),
        5,
    );

    let store = postgres::PostgresStore::new(config).await.unwrap();
    container.stop();
    let result = store.check().await;

    assert_eq!(*result.status(), Status::Degraded);
}

#[derive(Clone, Getters)]
struct Postgres {
    tag: String,

    #[getset(get)]
    port: u16,

    #[getset(get)]
    name: String,

    #[getset(get)]
    user: String,

    #[getset(get)]
    password: String,

    env_vars: HashMap<String, String>,
}
impl Postgres {
    fn new(tag: String, port: u16, name: String, user: String, password: String) -> Self {
        let mut env_vars = HashMap::new();
        env_vars.insert("POSTGRES_DB".to_string(), name.to_string());
        env_vars.insert("POSTGRES_USER".to_string(), user.to_string());
        env_vars.insert("POSTGRES_PASSWORD".to_string(), password.to_string());

        Postgres {
            tag,
            port,
            name,
            user,
            password,
            env_vars,
        }
    }
}

impl Default for Postgres {
    fn default() -> Self {
        Postgres::new(
            "16.2-bullseye".to_string(),
            5432,
            "rustic.sketch".to_string(),
            "rustic.sketch.dev".to_string(),
            "rustic.sketch.pw".to_string(),
        )
    }
}

impl Image for Postgres {
    // see https://github.com/testcontainers/testcontainers-rs-modules-community/blob/main/src/postgres/mod.rs
    type Args = ();

    fn name(&self) -> String {
        "postgres".to_string()
    }

    fn tag(&self) -> String {
        self.tag.to_string()
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        )]
    }

    fn env_vars(&self) -> Box<dyn Iterator<Item = (&String, &String)> + '_> {
        Box::new(self.env_vars.iter())
    }

    fn expose_ports(&self) -> Vec<u16> {
        vec![self.port]
    }
}
