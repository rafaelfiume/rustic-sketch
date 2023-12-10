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

#[cfg(test)]
mod test {
    use super::*;
    use crate::health_check::service_status::test_kit::*;
    use crate::health_check::version::test_kit::arb_versions;
    use proptest::proptest;
    use proptest::collection::vec;

    proptest! {
        #[test]
        fn establishes_service_good_health(
            version in arb_versions(),
            dependencies in vec(arb_healthy_dependencies(), 0..2),
        ) {
            let result = ServiceStatus::new(version.clone(), dependencies.clone());
            assert_eq!(result.status, Status::Ok)
       }
    }

    // TODO Check combination of Ok and Degraded status

    proptest! {
        #[test]
        fn establishes_service_poor_health(
            version in arb_versions(),
            dependencies in vec(arb_unhealthy_dependencies(), 1..2),
        ) {
            let result = ServiceStatus::new(version.clone(), dependencies.clone());
            assert_eq!(result.status, Status::Degraded)
       }
    }

    proptest! {
        #[test]
        fn tracks_version_and_dependencies(
            version in arb_versions(),
            dependencies in vec(arb_service_dependencies(), 0..2)
        ) {
            let result = ServiceStatus::new(version.clone(), dependencies.clone());
            assert_eq!(result.version, version);
            assert_eq!(result.dependencies, dependencies);
       }
    }
}

#[cfg(test)]
pub(crate) mod test_kit {
    use super::*;
    use proptest::prelude::*;

    pub fn arb_healthy_dependencies() -> impl Strategy<Value = DependencyStatus> {
        // filtering in action
        arb_service_dependencies().prop_filter("Ok", |d| d.status == Status::Ok)
    }

    pub fn arb_unhealthy_dependencies() -> impl Strategy<Value = DependencyStatus> {
        arb_service_dependencies().prop_filter("Degraded", |d| d.status == Status::Degraded)
    }

    pub fn arb_service_dependencies() -> impl Strategy<Value = DependencyStatus> {
        // manually composing strategies with `prop_map` instead of using `prop_compose!`
        (arb_dependencies(), arb_status())
            .prop_map(|(dependencies, status)| DependencyStatus::new(dependencies, status))
    }

    fn arb_status() -> impl Strategy<Value = Status> {
        prop_oneof![Just(Status::Ok), Just(Status::Degraded),]
    }

    fn arb_dependencies() -> impl Strategy<Value = Dependency> {
        prop_oneof![Just(Dependency::Database),]
    }
}
