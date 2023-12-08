extern crate derive_more;

use derive_more::Constructor;
use derive_more::Display;
use std::error::Error;
use std::fs;

#[derive(Clone, Debug, PartialEq)]
pub struct Version {
    env: Environment,
    build: Build,
    commit: Commit,
}
impl Version {
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

// TODO use anyhow or thiserror to deal with errors?
#[derive(Debug, Display)]
pub struct VersionLoadError {
    message: String,
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

#[cfg(test)]
pub mod tests {
    use super::*;

    // ** Stubs ** //

    pub fn current_version(
        env: Environment,
        build: Build,
        commit: Commit,
    ) -> Result<Version, VersionLoadError> {
        Ok(Version { env, build, commit })
    }
}
