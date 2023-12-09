pub mod health_check; // publicly re-exported so it can be used in integration tests
pub mod routes;

use routes::health_status;
use warp::Filter;

pub async fn run() {
    let hello_route = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));

    let routes = health_status::routes()
        .or(hello_route)
        // TODO any origins for now
        .with(warp::cors().allow_any_origin());

    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await
}
