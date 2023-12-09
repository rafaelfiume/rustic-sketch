use crate::health_check::{
    service_status::{Dependency, DependencyStatus, ServiceStatus, Status},
    version::Version,
};
use serde::{ser::SerializeMap, Deserialize, Deserializer, Serialize, Serializer};

use self::payload_converters::AsPayload;

// TODO Move it to a separate file
pub mod payload_converters {
    pub trait AsPayload<T> {
        fn as_payload(&self) -> T;
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ServiceStatusPayload {
    #[serde(flatten)]
    version: VersionPayload,
    status: String,
    dependencies: Vec<DependencyStatusPayload>,
}
impl AsPayload<ServiceStatusPayload> for ServiceStatus {
    fn as_payload(&self) -> ServiceStatusPayload {
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
impl AsPayload<VersionPayload> for Version {
    fn as_payload(&self) -> VersionPayload {
        VersionPayload {
            env: self.env().to_string(),
            build: self.build().to_string(),
            commit: self.commit().to_string(),
        }
    }
}

impl AsPayload<String> for Dependency {
    fn as_payload(&self) -> String {
        match self {
            Dependency::Database => "database".to_string(),
        }
    }
}

impl AsPayload<String> for Status {
    fn as_payload(&self) -> String {
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
impl AsPayload<DependencyStatusPayload> for DependencyStatus {
    fn as_payload(&self) -> DependencyStatusPayload {
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
