pub mod model;

use std::future::Future;

use crate::health_check::service_status::{Dependency, DependencyStatus, ServiceStatus, Status};
use crate::health_check::version::{Environment, Version, VersionLoadError};
use crate::health_status::model::payload_converters::AsPayload;
use warp::reject::{self, Rejection};
use warp::reply::Reply;
use warp::Filter;

pub fn routes() -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    ping().or(health_check(do_check_health))
}

fn ping() -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("ping").map(|| warp::reply::json(&"pong"))
}

fn health_check<F, S>(
    check_health: F,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone
where
    F: Fn() -> S + Clone + Send + Sync,
    S: Future<Output = Result<ServiceStatus, VersionLoadError>> + Send,
{
    warp::path("status").and_then(move || {
        let fnn = check_health.clone();
        async move {
            match fnn().await {
                Ok(service_health) => Ok(warp::reply::json(&service_health.as_payload())),
                Err(e) => Err(reject::custom(e)),
            }
        }
    })
}

async fn do_check_health() -> Result<ServiceStatus, VersionLoadError> {
    // TODO Hardcoded for now
    let env = Environment::new("dev".to_string());
    let version = Version::current_version(env, &"rustic.version".to_string())?;
    let dependencies = vec![DependencyStatus::new(Dependency::Database, Status::Ok)];
    Ok(ServiceStatus::new(version, dependencies))
}

// TODO Wrap a json payload?
//struct CustomJsonRejection(reply::Json);

impl warp::reject::Reject for VersionLoadError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::health_check::version;
    use crate::health_check::version::{Build, Commit, Environment};
    use crate::health_status::model::ServiceStatusPayload;
    use serde_json::Value;

    // See https://docs.rs/warp/latest/warp/test/index.html
    #[tokio::test]
    async fn ping_returns_pong() {
        let filter = ping();

        let result = warp::test::request()
            .method("GET")
            .path("/ping")
            .reply(&filter)
            .await;

        assert_eq!(result.status(), 200);
        assert_eq!(
            serde_json::from_slice::<Value>(result.body()).unwrap(),
            serde_json::json!("pong")
        );
    }

    #[tokio::test]
    async fn status_returns_version() {
        // TODO Property-based tests
        let version = version::tests::current_version(
            Environment::new("test".to_string()),
            Build::new("fjadljfald;sjfsaf".to_string()),
            Commit::new("fjdfljafj;asfdsf".to_string()),
        )
        .unwrap();
        let service_status = ServiceStatus::new(version, Vec::new());
        let primed_service_status = service_status.clone();
        let check_health_returns_ok = move || {
            let c = primed_service_status.clone();
            async { Ok::<ServiceStatus, VersionLoadError>(c) }
        };
        let status = health_check(check_health_returns_ok);

        let result = warp::test::request()
            .method("GET")
            .path("/status")
            .reply(&status)
            .await;

        assert_eq!(result.status(), 200);
        let obtained: ServiceStatusPayload = serde_json::from_slice(result.body()).unwrap();
        assert_eq!(obtained, service_status.as_payload());
    }
}
