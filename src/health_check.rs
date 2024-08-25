use self::{
    service_status::{DependencyStatus, ServiceStatus},
    version::Versioned,
};
use async_trait::async_trait;
use derive_more::Constructor;
use derive_more::Display;
use derive_more::Error;
use futures::future::join_all;
use getset::Getters;

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
        let version = self
            .versioned
            .version()
            .await
            .map_err(|err| HealthCheckError {
                message: format!("Failed to load version: {}", err.message()),
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

#[derive(Clone, Constructor, Debug, Display, Error, Getters)]
pub struct HealthCheckError {
    #[getset(get)]
    message: String,
}

#[cfg(test)]
mod tests {
    use super::version::test_kit::StubVersion;
    use super::*;
    use crate::health_check::service_status::{Dependency, Status};
    use crate::health_check::test_kit::StubDependencyHealthChecker;
    use crate::health_check::version::{Build, Commit, Environment};

    #[tokio::test]
    async fn returns_service_status() {
        let versioned = StubVersion::new(
            Environment::new("dev".to_string()),
            Build::new("feat.branch.108".to_string()),
            Commit::new("c11e2d041c9b4ca66e241f8429e9a2876a8e0b18".to_string()),
        );
        let database_health_checker =
            StubDependencyHealthChecker::new(Dependency::Database, Status::Ok);
        let snitch_health_checker =
            StubDependencyHealthChecker::new(Dependency::Snitch, Status::Ok);

        let health_checker = RusticSketchHealthChecker {
            versioned: Box::new(versioned),
            dependency_health_checkers: vec![
                Box::new(database_health_checker),
                Box::new(snitch_health_checker),
            ],
        };
        let result = health_checker.check().await.unwrap();

        assert_eq!(*result.status(), Status::Ok);
    }

    #[tokio::test]
    async fn service_status_includes_version() {
        let env = Environment::new("dev".to_string());
        let build = Build::new("feat.branch.108".to_string());
        let commit = Commit::new("c11e2d041c9b4ca66e241f8429e9a2876a8e0b18".to_string());
        let versioned = StubVersion::new(env.clone(), build.clone(), commit.clone());
        let database_health_checker =
            StubDependencyHealthChecker::new(Dependency::Database, Status::Ok);

        let health_checker = RusticSketchHealthChecker {
            versioned: Box::new(versioned),
            dependency_health_checkers: vec![Box::new(database_health_checker)],
        };
        let service_status = health_checker.check().await.unwrap();
        let result = service_status.version();

        assert_eq!(*result.env(), env);
        assert_eq!(*result.build(), build);
        assert_eq!(*result.commit(), commit);
    }
}

#[cfg(test)]
pub(crate) mod test_kit {
    use super::{
        service_status::{Dependency, Status},
        *,
    };

    /* Stubs */

    #[derive(Constructor)]
    pub struct StubHealthChecker {
        service_status: Result<ServiceStatus, HealthCheckError>,
    }
    #[async_trait]
    impl HealthChecker for StubHealthChecker {
        async fn check(&self) -> Result<ServiceStatus, HealthCheckError> {
            self.service_status.clone()
        }
    }

    #[derive(Constructor)]
    pub struct StubDependencyHealthChecker {
        dependency: Dependency,
        status: Status,
    }
    #[async_trait]
    impl DependencyHealthChecker for StubDependencyHealthChecker {
        async fn check(&self) -> DependencyStatus {
            DependencyStatus::new(self.dependency.clone(), self.status.clone())
        }
    }
}
