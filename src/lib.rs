use std::sync::Arc;

use health_check::{
    version::{Environment, VersionFromFile},
    RusticSketchHealthChecker,
};
use routes::health_status; // TODO Do I like this name?
use warp::Filter;

pub mod health_check; // publicly re-exported so it can be used in integration tests
pub mod routes;

pub async fn run() {
    let hello_route = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));

    let versioned = VersionFromFile::new(
        Environment::new("dev".to_string()), // TODO Hardcoded for now
        "rustic.version".to_string(),
    );
    let health_checker = RusticSketchHealthChecker::new(
        Box::new(versioned),
        vec![], // TODO empty for now
    );
    let routes = health_status::routes(Arc::new(health_checker))
        .or(hello_route)
        // TODO any origins for now
        .with(warp::cors().allow_any_origin());

    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await
}
