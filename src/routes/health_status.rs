use crate::health_check::service_status::{Dependency, DependencyStatus, ServiceStatus, Status};
use crate::health_check::version::{Environment, Version, VersionLoadError};
use warp::reject::{self, Rejection};
use warp::reply::Reply;
use warp::Filter;

pub fn routes() -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    ping().or(health_check())
}

fn ping() -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("ping").map(|| warp::reply::json(&"pong"))
}

fn health_check() -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("status").and_then(check_health)
}

// TODO Wrap a json payload?
//struct CustomJsonRejection(reply::Json);

impl warp::reject::Reject for VersionLoadError {}

async fn check_health(/* db */) -> Result<impl warp::Reply, Rejection> {
    // TODO Hardcoded
    let env = Environment::new("dev".to_string());
    let version = match Version::current_version(env, &"rustic.version".to_string()) {
        Ok(v) => v,
        Err(e) => return Err(reject::custom(e)),
    };
    // dbg!(&version);
    let dependencies = vec![DependencyStatus {
        dependency: Dependency::Database,
        status: Status::Ok,
    }];
    let service_health = ServiceStatus::new(version, dependencies);
    Ok(warp::reply::json(&service_health))
}

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
