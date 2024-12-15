# rustic-sketch

[![CircleCI](https://dl.circleci.com/status-badge/img/gh/rafaelfiume/rustic-sketch/tree/main.svg?style=svg)](https://dl.circleci.com/status-badge/redirect/gh/rafaelfiume/rustic-sketch/tree/main) [<img src="https://img.shields.io/badge/dockerhub-images-blue.svg?logo=LOGO">](<https://hub.docker.com/repository/docker/rafaelfiume/rustic-sketch/general>)


## A Few Rust Features in Action

### Tests

 - [Integration test](tests/version_test.rs)
 - Property-based tests:
   - [prop_compose!](src/health_check/version.rs)
   - [proptest! macro doesn't play nice with tokio::test](https://github.com/proptest-rs/proptest/issues/179).
 - [Table-driven test](tests/public_contracts.rs)
 - [Testing Api contracts](tests/test_kit.rs)
 - [Unit test](src/routes/health_status.rs)

### The language

 - [#expect](tests/public_contracts.rs)
 - [Getters methods](src/health_check/version.rs)
   - For derived getters, see `getset` crait.
 - Iterators
   - [chars()](tests/test_kit.rs)
   - [try_for_each()](tests/public_contracts.rs)
 - [Lifecycle management](tests/test_kit.rs) (see `'a`)
 - Macros:
   - [dbg!](tests/test_kit.rs)
   - [format!](tests/public_contracts.rs)
 - [Question mark operator](src/health_check/version.rs), `?`
 - Types
   - [Associated Types](src/routes/health_status/model.rs) (see `type Value = DependencyStatusPayload`)
   - [Type Alias](tests/test_kit.rs) (see `TestResult`)
 - [#unwrap](build.rs)

### A few interesting crates

 - [async-trait](src/health_check.rs)
 - derive_more
   - [Clone](src/health_check.rs) (see `HealthCheckError`)
   - [Constructor](src/health_check.rs) (see `HealthCheckError`)
   - [Debug](src/health_check.rs) (see `HealthCheckError`)
   - [Display](src/health_check.rs) (see `HealthCheckError`)
   - [Eq](src/health_check/service_status.rs) (see `Status`)
   - [Error](src/health_check.rs) (see `HealthCheckError`)
   - [Hash](src/health_check/service_status.rs) (see `Status`)
   - [PartialEq](src/health_check/service_status.rs) (see `Status`)
 - getset
   - [#[getset(get)]]() (see `HealthCheckError`)
 - serde
   - [Derived Serialization and Deserialization](src/routes/health_status/model.rs)
   - [Custom Deserialization](src/routes/health_status/model.rs)
   - [Custom Serialization](src/routes/health_status/model.rs)
 - [warp](src/routes/health_status.rs)

### Passing an async closure to a function

This has challenged me: defining a function that takes an async closure as parameter. [This](https://tech.fpcomplete.com/blog/captures-closures-async/) article and [this](https://github.com/hyperium/hyper/blob/6d26ee0a97e008c8e50de79f5657e8240ab304b9/src/service/util.rs#L30) hyper source code helped me to get there.

```
fn check_health(
    health_checker: Arc<dyn HealthChecker + Send + Sync>,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("status").and_then(move || {
        let fnn = health_checker.clone(); // <- this was the trickiest part
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
```

From the article, this was key to me:

> "The formatting recommended by rustfmt hides away the fact that there are two different environments at play between the outer closure and the async block, by moving the two onto a single line with move |_conn| async move."
