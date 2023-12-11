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
    status: Status,
    dependencies: Vec<DependencyStatusPayload>,
}
impl AsPayload<ServiceStatusPayload> for ServiceStatus {
    fn as_payload(&self) -> ServiceStatusPayload {
        ServiceStatusPayload {
            version: self.version().as_payload(),
            status: self.status().clone(),
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

impl Serialize for Status {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(match self {
            Status::Ok => "Ok",
            Status::Degraded => "Degraded",
        })
    }
}

impl Serialize for Dependency {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(match self {
            Dependency::Auth0 => "auth0",
            Dependency::Database => "database",
            Dependency::Snitch => "snitch",
        })
    }
}

// A custom Deserializer for enums combined with contract tests will make sure contracts are kept

impl<'de> Deserialize<'de> for Status {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "Ok" => Ok(Status::Ok),
            "Degraded" => Ok(Status::Degraded),
            unknown => Err(serde::de::Error::custom(format!(
                "Invalid Status: `{}`",
                unknown
            ))),
        }
    }
}

impl<'de> Deserialize<'de> for Dependency {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "database" => Ok(Dependency::Database),
            unknown => Err(serde::de::Error::custom(format!(
                "Invalid Dependency: `{}`",
                unknown
            ))),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct DependencyStatusPayload {
    dependency: Dependency,
    status: Status,
}
impl AsPayload<DependencyStatusPayload> for DependencyStatus {
    fn as_payload(&self) -> DependencyStatusPayload {
        DependencyStatusPayload {
            dependency: self.dependency().clone(),
            status: self.status().clone(),
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
