use async_trait::async_trait;
use derive_more::Constructor;
use derive_more::Display;
use derive_more::Error;
use getset::Getters;
use std::fs;

#[async_trait]
pub trait Versioned {
    async fn version(&self) -> Result<Version, VersionLoadError>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct Version {
    env: Environment,
    build: Build,
    commit: Commit,
}
impl Version {
    // - we need to use the `&` in front of the self shorthand to indicate that this method borrows the Self instance
    // - `&self` is appropriate here since we don't want to take ownership, and only read the data in the struct
    // - `&mut self` would be appropriate if we wanted to change the instance we are calling the method from
    // - `self` takes ownership of the instance, which is rare and usually used...
    // ... when transforming the instance and preventing the original caller from using the original instance.
    pub fn env(&self) -> &Environment {
        &self.env
    }

    pub fn build(&self) -> &Build {
        &self.build
    }

    pub fn commit(&self) -> &Commit {
        &self.commit
    }
}

#[derive(Constructor)]
pub struct VersionFromFile {
    env: Environment,
    path: String,
}
#[async_trait]
impl Versioned for VersionFromFile {
    async fn version(&self) -> Result<Version, VersionLoadError> {
        let content = fs::read_to_string(&self.path).map_err(|e| VersionLoadError {
            message: e.to_string(),
        })?;
        let mut lines = content.lines();
        let build = lines.next().ok_or(VersionLoadError {
            message: "No build number specified in 'rustic.version'".to_string(),
        })?;
        let commit = lines.next().ok_or(VersionLoadError {
            message: "No commit hash specified in 'rustic.version'".to_string(),
        })?;
        let version = Version {
            env: self.env.to_owned(),
            build: Build::new(build.to_owned()),
            commit: Commit::new(commit.to_owned()),
        };
        Ok(version)
    }
}

// TODO use anyhow or thiserror to deal with errors?
#[derive(Debug, Display, Error, Getters)]
pub struct VersionLoadError {
    #[getset(get = "pub")]
    message: String,
}

#[derive(Clone, Constructor, Debug, Display, PartialEq)]
// The newtype pattern.
// Use owned String instead of slice &str: each instance of this struct own its own data,
// always valid for as long the entire struct is valid.
pub struct Environment(String);

#[derive(Clone, Constructor, Debug, Display, PartialEq)]
pub struct Build(String);

#[derive(Clone, Constructor, Debug, Display, PartialEq)]
pub struct Commit(String);

#[cfg(test)] // conditional compilation attr: item included only during tests.
pub(crate) mod test_kit {
    use super::*;
    use proptest::prelude::*;
    use proptest::prop_compose;

    use crate::health_check::version::{Environment, Version};

    // ** Stubs ** //

    #[derive(Clone, Constructor)]
    pub struct StubVersion {
        env: Environment,
        build: Build,
        commit: Commit,
    }
    impl From<StubVersion> for Version {
        fn from(stub: StubVersion) -> Self {
            Version {
                env: stub.env,
                build: stub.build,
                commit: stub.commit,
            }
        }
    }
    #[async_trait]
    impl Versioned for StubVersion {
        async fn version(&self) -> Result<Version, VersionLoadError> {
            Ok(self.clone().into())
        }
    }

    // ** Generators **//

    prop_compose! {
        // The generated function will take the fst parameter list as arguments
        // Strategies parameters are defined in the snd argument list
        pub fn arb_version()(env in arb_env(), build in arb_build(), commit in arb_commit()) -> Version {
          StubVersion::new(env, build, commit).into()
        }
    }

    fn arb_env() -> impl Strategy<Value = Environment> {
        prop_oneof![Just("dev".to_string()), Just("prd".to_string())].prop_map(Environment::new)
    }

    prop_compose! {
        fn arb_build()(build in "(snapshot|branch-name.[0-9]{1,6}|[0-9]{1,6})") -> Build {
           Build::new(build)
        }
    }

    prop_compose! {
        fn arb_commit()(commit in "[0-9a-f]{40}") -> Commit {
            Commit::new(commit)
        }
    }
}
