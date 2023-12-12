extern crate derive_more;

use derive_more::Constructor;
use derive_more::Display;
use std::error::Error;
use std::fs;

// TODO Async
pub trait Versioned {
    fn version(&self) -> Result<Version, VersionLoadError>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct Version {
    env: Environment,
    build: Build,
    commit: Commit,
}
impl Version {
    // TODO make it async
    pub fn current_version(
        environment: Environment,
        path: &String,
    ) -> Result<Version, VersionLoadError> {
        let content = fs::read_to_string(path).map_err(|e| VersionLoadError {
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
            env: environment,
            build: Build::new(build.to_owned()),
            commit: Commit::new(commit.to_owned()),
        };
        Ok(version)
    }

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

struct VersionFromFile {
    env: Environment,
    path: String,
}

impl Versioned for VersionFromFile {
    fn version(&self) -> Result<Version, VersionLoadError> {
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
#[derive(Debug, Display)]
pub struct VersionLoadError {
    pub message: String,
}
impl Error for VersionLoadError {}

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

    pub struct StubVersion {
        pub env: Environment,
        pub build: Build,
        pub commit: Commit,
    }
    impl Versioned for StubVersion {
        fn version(&self) -> Result<Version, VersionLoadError> {
            Ok(Version {
                env: self.env.clone(),
                build: self.build.clone(),
                commit: self.commit.clone(),
            })
        }
    }

    pub fn current_version(
        env: Environment,
        build: Build,
        commit: Commit,
    ) -> Result<Version, VersionLoadError> {
        Ok(Version { env, build, commit })
    }

    // ** Generators **//

    // about boxing or not see: https://proptest-rs.github.io/proptest/proptest/tutorial/transforming-strategies.html

    prop_compose! {
        // The generated function will take the fst parameter list as arguments
        // Strategies parameters are defined in the snd argument list
        pub fn arb_version()(env in arb_env(), build in arb_build(), commit in arb_commit()) -> Version {
          current_version(env, build, commit).unwrap()
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
