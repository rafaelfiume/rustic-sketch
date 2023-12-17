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

#[derive(Clone, Debug, Display, Eq, Hash, PartialEq)]
pub enum Status {
    Ok,
    Degraded,
}

#[derive(Clone, Debug, Display, Eq, Hash, PartialEq)]
pub enum Dependency {
    Auth0,
    Database,
    Snitch,
}

#[derive(Clone, Constructor, Debug, Eq, Hash, PartialEq)]
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
    use crate::health_check::version::test_kit::arb_version;
    use proptest::collection::vec;
    use proptest::proptest;

    proptest! {
        #[test]
        fn establish_service_good_health(
            version in arb_version(),
            dependencies in vec(arb_healthy_dependency(), 0..4),
        ) {
            let result = ServiceStatus::new(version, dependencies);
            assert_eq!(result.status, Status::Ok)
       }
    }

    proptest! {
        #[test]
        fn establish_service_poor_health(
            version in arb_version(),
            dependencies in arb_unhealthy_dependencies(),
        ) {
            let result = ServiceStatus::new(version, dependencies);
            assert_eq!(result.status, Status::Degraded)
       }
    }

    proptest! {
        #[test]
        fn track_version_and_dependencies(
            version in arb_version(),
            dependencies in vec(arb_service_dependency(), 0..4)
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
    use proptest::collection::vec;
    use proptest::prelude::*;
    use std::collections::HashSet;

    // ** Generators ** //

    pub fn arb_healthy_dependency() -> impl Strategy<Value = DependencyStatus> {
        arb_service_dependency().prop_filter("Ok", |d| d.status == Status::Ok)
    }

    pub fn arb_service_dependency() -> impl Strategy<Value = DependencyStatus> {
        // manually composing strategies with `prop_map` instead of using `prop_compose!`
        (arb_dependency(), arb_status())
            .prop_map(|(dependencies, status)| DependencyStatus::new(dependencies, status))
    }

    pub fn arb_unhealthy_dependencies() -> impl Strategy<Value = Vec<DependencyStatus>> {
        vec(arb_healthy_dependency(), 1..4)
            .prop_flat_map(|vec| {
                let unique: HashSet<_> = vec.into_iter().collect();
                let len = unique.len();
                let vec: Vec<_> = unique.into_iter().collect();
                (Just(vec), 0..len)
            })
            .prop_flat_map(|(mut vec, idx)| {
                vec[idx].status = Status::Degraded;
                Just(vec)
            })
    }

    fn arb_status() -> impl Strategy<Value = Status> {
        prop_oneof![Just(Status::Ok), Just(Status::Degraded)]
    }

    fn arb_dependency() -> impl Strategy<Value = Dependency> {
        prop_oneof![
            Just(Dependency::Auth0),
            Just(Dependency::Database),
            Just(Dependency::Snitch)
        ]
    }
}
