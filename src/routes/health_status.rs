use std::future::Future;

use crate::health_check::service_status::{Dependency, DependencyStatus, ServiceStatus, Status};
use crate::health_check::version::{Environment, Version, VersionLoadError};
use warp::reject::{self, Rejection};
use warp::reply::Reply;
use warp::Filter;

use self::payload_converters::AsPayload;

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

mod payload_converters {
    pub trait AsPayload {
        type Payload;
        fn as_payload(&self) -> Self::Payload;
    }
}

mod model {
    use super::payload_converters::AsPayload;
    use crate::health_check::{
        service_status::{Dependency, DependencyStatus, ServiceStatus, Status},
        version::Version,
    };
    use serde::{ser::SerializeMap, Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    pub struct ServiceStatusPayload {
        #[serde(flatten)]
        version: VersionPayload,
        status: String,
        dependencies: Vec<DependencyStatusPayload>,
    }

    impl AsPayload for ServiceStatus {
        type Payload = ServiceStatusPayload;
        fn as_payload(&self) -> Self::Payload {
            ServiceStatusPayload {
                version: self.version().as_payload(),
                status: self.status().as_payload().to_owned(),
                dependencies: self.dependencies().iter().map(|d| d.as_payload()).collect(),
            }
        }
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    pub struct VersionPayload {
        env: String,
        build: String,
        commit: String,
    }
    impl AsPayload for Version {
        type Payload = VersionPayload;
        fn as_payload(&self) -> Self::Payload {
            VersionPayload {
                env: self.env().to_string(),
                build: self.build().to_string(),
                commit: self.commit().to_string(),
            }
        }
    }

    impl AsPayload for Dependency {
        type Payload = String;
        fn as_payload(&self) -> Self::Payload {
            match self {
                Dependency::Database => "database".to_string(),
            }
        }
    }

    impl AsPayload for Status {
        type Payload = String;
        fn as_payload(&self) -> Self::Payload {
            match self {
                Status::Ok => "Ok".to_string(),
                Status::Degraded => "Degraded".to_string(),
            }
        }
    }

    #[derive(Debug, PartialEq)]
    pub struct DependencyStatusPayload {
        dependency: String,
        status: String,
    }
    impl AsPayload for DependencyStatus {
        type Payload = DependencyStatusPayload;
        fn as_payload(&self) -> Self::Payload {
            DependencyStatusPayload {
                dependency: self.dependency().as_payload(),
                status: self.status().as_payload(),
            }
        }
    }
    impl Serialize for DependencyStatusPayload {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut map = serializer.serialize_map(Some(1))?;
            map.serialize_entry(&self.dependency, &self.status)?;
            map.end()
        }
    }
    impl<'de> Deserialize<'de> for DependencyStatusPayload {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct DependencyStatusVisitor;

            impl<'de> serde::de::Visitor<'de> for DependencyStatusVisitor {
                // An example of an Associated Type
                // Only one possible implementation for a given type. Compare it with generics, where there can be many.
                type Value = DependencyStatusPayload;

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    formatter.write_str("an instance of DependencyStatus")
                }

                fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::MapAccess<'de>,
                {
                    let (dependency, status) = access
                        .next_entry()?
                        .ok_or_else(|| serde::de::Error::missing_field("dependency"))?;
                    Ok(DependencyStatusPayload { dependency, status })
                }
            }

            deserializer.deserialize_struct(
                "DependencyStatusPayload",
                &["dependency", "status"],
                DependencyStatusVisitor,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::health_check::version;
    use crate::health_check::version::{Build, Commit, Environment};
    use crate::routes::health_status::model::ServiceStatusPayload;

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
        let status = health_check(move || check_health_returns_ok());

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
