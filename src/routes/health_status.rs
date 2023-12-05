use std::future::Future;

use crate::health_check::service_status::{Dependency, DependencyStatus, ServiceStatus, Status};
use crate::health_check::version::{Environment, Version, VersionLoadError};
use warp::reject::{self, Rejection};
use warp::reply::Reply;
use warp::Filter;

pub fn routes() -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    ping().or(health_check(|| do_check_health()))
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
                Ok(service_health) => Ok(warp::reply::json(&service_health)),
                Err(e) => Err(reject::custom(e)),
            }
        }
    })
}

async fn do_check_health() -> Result<ServiceStatus, VersionLoadError> {
    // TODO Hardcoded
    let env = Environment::new("dev".to_string());
    let version = Version::current_version(env, &"rustic.version".to_string())?;
    // dbg!(&version);
    let dependencies = vec![DependencyStatus {
        dependency: Dependency::Database,
        status: Status::Ok,
    }];
    Ok(ServiceStatus::new(version, dependencies))
}

// TODO Wrap a json payload?
//struct CustomJsonRejection(reply::Json);

impl warp::reject::Reject for VersionLoadError {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    // See https://docs.rs/warp/latest/warp/test/index.html
    #[tokio::test]
    async fn ping_returns_pong() {
        let filter = ping();

        let res = warp::test::request()
            .method("GET")
            .path("/ping")
            .reply(&filter)
            .await;

        assert_eq!(res.status(), 200);
        assert_eq!(
            serde_json::from_slice::<Value>(res.body()).unwrap(),
            serde_json::json!("pong")
        );
    }
}
