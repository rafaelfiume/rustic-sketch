use self::{
    service_status::{DependencyStatus, ServiceStatus},
    version::Versioned,
};
use async_trait::async_trait;
use derive_more::Constructor;
use derive_more::Display;
use futures::future::join_all;
use std::error::Error;

pub mod service_status;
pub mod version;

#[async_trait]
pub trait HealthChecker {
    async fn check(&self) -> Result<ServiceStatus, HealthCheckError>;
}

#[async_trait]
pub trait DependencyHealthChecker {
    async fn check(&self) -> DependencyStatus;
}

#[derive(Constructor)]
pub struct RusticSketchHealthChecker {
    versioned: Box<dyn Versioned + Send + Sync>,
    dependency_health_checkers: Vec<Box<dyn DependencyHealthChecker + Sync + Send>>,
}

#[async_trait]
impl HealthChecker for RusticSketchHealthChecker {
    async fn check(&self) -> Result<ServiceStatus, HealthCheckError> {
        let version = self.versioned.version().map_err(|err| HealthCheckError {
            message: format!("Failed to load version: {}", err.message),
        })?;

        let futures: Vec<_> = self
            .dependency_health_checkers
            .iter()
            .map(|checker| checker.check())
            .collect();
        let dependencies = join_all(futures).await;

        Ok(ServiceStatus::new(version, dependencies))
    }
}

#[derive(Clone, Debug, Display)]
pub struct HealthCheckError {
    pub message: String,
}
impl Error for HealthCheckError {}

#[cfg(test)]
mod tests {
    use super::version::test_kit;
    use super::*;
    use crate::health_check::service_status::{Dependency, Status};
    use crate::health_check::test_kit::StubDependencyHealthChecker;
    use crate::health_check::version::{Build, Commit, Environment};

    #[tokio::test]
    async fn service_status_is_ok() {
        let versioned = test_kit::StubVersion {
            env: Environment::new("dev".to_string()),
            build: Build::new("feat.branch.108".to_string()),
            commit: Commit::new("c11e2d041c9b4ca66e241f8429e9a2876a8e0b18".to_string()),
        };
        let database_health_checker = StubDependencyHealthChecker {
            dependency: Dependency::Database,
            status: Status::Ok,
        };
        let snitch_health_checker = StubDependencyHealthChecker {
            dependency: Dependency::Snitch,
            status: Status::Ok,
        };

        let health_checker = RusticSketchHealthChecker {
            versioned: Box::new(versioned),
            dependency_health_checkers: vec![
                Box::new(database_health_checker),
                Box::new(snitch_health_checker),
            ],
        };
        let result = health_checker.check().await.unwrap();

        assert_eq!(result.status().clone(), Status::Ok);
    }

    #[tokio::test]
    async fn service_status_includes_version() {
        let env = Environment::new("dev".to_string());
        let build = Build::new("feat.branch.108".to_string());
        let commit = Commit::new("c11e2d041c9b4ca66e241f8429e9a2876a8e0b18".to_string());
        let versioned = test_kit::StubVersion {
            env: env.clone(),
            build: build.clone(),
            commit: commit.clone(),
        };
        let database_health_checker = StubDependencyHealthChecker {
            dependency: Dependency::Database,
            status: Status::Ok,
        };

        let health_checker = RusticSketchHealthChecker {
            versioned: Box::new(versioned),
            dependency_health_checkers: vec![Box::new(database_health_checker)],
        };
        let service_status = health_checker.check().await.unwrap();
        let result = service_status.version();

        assert_eq!(result.env().clone(), env);
        assert_eq!(result.build().clone(), build);
        assert_eq!(result.commit().clone(), commit);
    }
}

#[cfg(test)]
pub(crate) mod test_kit {
    use super::{
        service_status::{Dependency, Status},
        *,
    };

    /* Stubs */

    pub struct StubHealthChecker {
        pub service_status: Result<ServiceStatus, HealthCheckError>,
    }
    #[async_trait]
    impl HealthChecker for StubHealthChecker {
        async fn check(&self) -> Result<ServiceStatus, HealthCheckError> {
            self.service_status.clone()
        }
    }

    pub struct StubDependencyHealthChecker {
        pub dependency: Dependency,
        pub status: Status,
    }
    #[async_trait]
    impl DependencyHealthChecker for StubDependencyHealthChecker {
        async fn check(&self) -> DependencyStatus {
            DependencyStatus::new(self.dependency.clone(), self.status.clone())
        }
    }
}
