extern crate derive_more;

use crate::health_check::version::*;
use derive_more::Constructor;
use derive_more::Display;

#[derive(Clone, Debug, PartialEq)]
pub struct ServiceStatus {
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
            version,
            status: overall_status,
            dependencies,
        }
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn status(&self) -> &Status {
        &self.status
    }

    pub fn dependencies(&self) -> &Vec<DependencyStatus> {
        &self.dependencies
    }
}

#[derive(Clone, Debug, Display, PartialEq)]
pub enum Status {
    Ok,
    Degraded,
}

#[derive(Clone, Debug, Display, PartialEq)]
pub enum Dependency {
    Database,
}

#[derive(Clone, Constructor, Debug, PartialEq)]
pub struct DependencyStatus {
    dependency: Dependency,
    status: Status,
}

impl DependencyStatus {
    pub fn dependency(&self) -> &Dependency {
        &self.dependency
    }
    pub fn status(&self) -> &Status {
        &self.status
    }
}
