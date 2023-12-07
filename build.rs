use std::env::{self, VarError};
use std::error::Error;
use std::fs::File;
use std::io::Write;

const OUT_DIR: &str = ".";

fn main() -> Result<(), Box<dyn Error>> {
    // Necessary?
    println!("cargo:rerun-if-changed=.git/HEAD");
    create_version_file()
}

fn create_version_file() -> Result<(), Box<dyn Error>> {
    let build = pipeline_release_version().unwrap_or_else(|_| "snapshot".to_string());
    let commit_hash = head_commit_hash()?;

    let version_file_path = format!("{}/rustic.version", OUT_DIR);
    let mut version_file = File::create(version_file_path)?;
    version_file.write_all(build.as_bytes())?;
    version_file.write_all(b"\n")?;
    version_file.write_all(commit_hash.as_bytes())?;

    Ok(())
}

fn pipeline_release_version() -> Result<String, VarError> {
    let build_num = env::var("CIRCLE_BUILD_NUM")?;
    let version = env::var("CIRCLE_BRANCH").map(|branch| {
        if branch == "main" {
            build_num
        } else {
            format!("{branch}.{build_num}")
        }
    })?;
    Ok(version)
}

fn head_commit_hash() -> Result<String, Box<dyn Error>> {
    let repo = git2::Repository::open(".")?;
    let head_commit = repo.head()?.peel_to_commit()?;
    Ok(head_commit.id().to_string())
}
