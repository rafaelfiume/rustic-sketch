use crate::health_check::version::*;
use serde::{ser::SerializeStruct, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ServiceStatus {
    #[serde(flatten)]
    version: Version,
    status: Status,
    dependencies: Vec<DependencyStatus>,
}
impl ServiceStatus {
    pub fn new(version: Version, dependencies: Vec<DependencyStatus>) -> Self {
        let overall_status = dependencies.iter().fold(Status::Ok, |acc, dependency| {
            if acc == Status::Ok && dependency.status == Status::Ok {
                Status::Ok
            } else {
                Status::Degraded
            }
        });
        ServiceStatus {
            version: version,
            status: overall_status,
            dependencies: dependencies,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum Status {
    Ok,
    Degraded,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum Dependency {
    Database,
}
impl Dependency {
    fn name(&self) -> &'static str {
        match self {
            Dependency::Database => "database",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DependencyStatus {
    pub dependency: Dependency,
    pub status: Status,
}
impl Serialize for DependencyStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("DependencyStatus", 2)?;
        state.serialize_field(&self.dependency.name(), &self.status)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for DependencyStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DependencyStatusVisitor;

        impl<'de> serde::de::Visitor<'de> for DependencyStatusVisitor {
            // An example of an Associated Type
            // Only one possible implementation for a given type. Compare it with generics, where there can be many.
            type Value = DependencyStatus;

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
                Ok(DependencyStatus { dependency, status })
            }
        }

        deserializer.deserialize_struct(
            "DependencyStatus",
            &["dependency", "status"],
            DependencyStatusVisitor,
        )
    }
}
