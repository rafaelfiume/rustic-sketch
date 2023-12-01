use crate::health_check::version::*;
use serde::{ser::SerializeStruct, Serialize, Serializer};

#[derive(Debug, Serialize)]
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

#[derive(Debug, PartialEq, Serialize)]
pub enum Status {
    Ok,
    Degraded,
}

#[derive(Debug, Serialize)]
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

#[derive(Debug)]
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
