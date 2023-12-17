use claims::assert_err;
use rustic_sketch::health_check::version::{
    Build, Commit, Environment, VersionFromFile, Versioned,
};
use std::error::Error;
use std::fs;

#[tokio::test]
async fn retrieves_service_version() {
    let env = Environment::new("dev".to_string());
    let build = Build::new("snapshot".to_string());
    let commit = Commit::new("d1a1efeba1806cd2d0fe4164162272afb0f121f4".to_string());
    let version_file_path = version_file_exists_in_location(&build, &commit).unwrap();

    let versioned = VersionFromFile::new(env.clone(), version_file_path.clone());
    let result = versioned.version().await.unwrap();

    assert_eq!(result.env(), &env);
    assert_eq!(result.build(), &build);
    assert_eq!(result.commit(), &commit);
    fs::remove_file(&version_file_path).expect("error when removing version file")
}

#[tokio::test]
async fn version_returns_error_when_there_is_no_version_file() {
    let env = Environment::new("dev".to_string());
    let version_file_path = "unknown.version.file".to_string();

    let versioned = VersionFromFile::new(env.clone(), version_file_path);
    let result = versioned.version().await;

    // TODO Check actual error
    assert_err!(result);
}

#[tokio::test]
async fn version_returns_error_when_version_file_is_empty() {
    let env = Environment::new("dev".to_string());
    let version_file_path = empty_version_file_path("rustic.version".to_string()).unwrap();

    let versioned = VersionFromFile::new(env.clone(), version_file_path.clone());
    let result = versioned.version().await;

    assert_err!(result);
    fs::remove_file(&version_file_path).unwrap()
}

fn version_file_exists_in_location(
    build: &Build,
    commit: &Commit,
) -> Result<String, Box<dyn Error>> {
    let version_file_path = empty_version_file_path("empty.version.file".to_string())?;
    let valid_content = format!("{build}\n{commit}");
    // Passing a reference of `version_file_path`, so `write` borrows the string for the
    // write operation instead of moving it.
    fs::write(&version_file_path, valid_content)?;
    Ok(version_file_path)
}

// tests run in parallel, so we need to specify different names and avoid them clashing with each other
fn empty_version_file_path(filename: String) -> Result<String, Box<dyn Error>> {
    let version_file_path = format!("tests/{}", filename);
    fs::write(&version_file_path, "")?;
    Ok(version_file_path)
}
