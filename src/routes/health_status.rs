use self::model::ServiceStatusPayload;
use crate::health_check::{HealthCheckError, HealthChecker};
use std::sync::Arc;
use warp::reject::{self, Rejection};
use warp::reply::Reply;
use warp::Filter;

pub mod model;

pub fn routes(
    health_checker: Arc<dyn HealthChecker + Send + Sync>,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    ping().or(check_health(health_checker))
}

fn ping() -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("ping").map(|| warp::reply::json(&"pong"))
}

fn check_health(
    health_checker: Arc<dyn HealthChecker + Send + Sync>,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("status").and_then(move || {
        let fnn = health_checker.clone();
        async move {
            match fnn.check().await {
                Ok(service_status) => Ok(warp::reply::json(&Into::<ServiceStatusPayload>::into(
                    service_status,
                ))),
                Err(e) => Err(reject::custom(e)),
            }
        }
    })
}

// TODO Wrap a json payload?
//struct CustomJsonRejection(reply::Json);

impl warp::reject::Reject for HealthCheckError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::health_check::service_status::ServiceStatus;
    use crate::health_check::test_kit::StubHealthChecker;
    use crate::health_check::version::test_kit::StubVersion;
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
    async fn status_checks_service_health() {
        let version = StubVersion::new(
            Environment::new("dev".to_string()),
            Build::new("feat.branch.108".to_string()),
            Commit::new("c11e2d041c9b4ca66e241f8429e9a2876a8e0b18".to_string()),
        )
        .into();
        let service_status = ServiceStatus::new(version, Vec::new());
        let health_checker = Arc::new(StubHealthChecker::new(Ok(service_status.clone())));

        let status = check_health(health_checker);
        let result = warp::test::request()
            .method("GET")
            .path("/status")
            .reply(&status)
            .await;

        assert_eq!(result.status(), 200);
        let obtained: ServiceStatusPayload = serde_json::from_slice(result.body()).unwrap();
        assert_eq!(obtained, service_status.into());
    }

    #[tokio::test]
    async fn status_fails_with_error() {
        let health_checker = Arc::new(StubHealthChecker::new(Err(HealthCheckError::new(
            "something went wrong".to_string(),
        ))));

        let status = check_health(health_checker);
        let result = warp::test::request()
            .method("GET")
            .path("/status")
            .reply(&status)
            .await;

        assert_eq!(result.status(), 500);
        // TODO Error payload
    }
}
