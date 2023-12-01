use claims::assert_err;
use rustic_sketch::health_check::version::{Build, Commit, Environment, Version};
use std::error::Error;
use std::fs;

// TODO Can I have property-based integration tests?
#[test]
fn current_service_version() {
    let env = Environment::new("dev".to_string());
    let build = Build::new("snapshot".to_string());
    let commit = Commit::new("d1a1efeba1806cd2d0fe4164162272afb0f121f4".to_string());
    let version_file_path =
        version_file_exists_in_location(&build, &commit).expect("no version file");

    let result = Version::current_version(env.clone(), &version_file_path)
        .expect("error when making version");

    assert_eq!(result.env(), &env);
    assert_eq!(result.build(), &build);
    assert_eq!(result.commit(), &commit);
    fs::remove_file(&version_file_path).expect("error when removing version file")
}

// Sad path

#[test]
fn current_service_version_returns_error_when_there_is_no_version_file() {
    let env = Environment::new("dev".to_string());
    let version_file_path = "unknown.version.file".to_string();

    let result = Version::current_version(env.clone(), &version_file_path);

    // TODO Check actual error
    assert_err!(result);
}

#[test]
fn current_service_version_returns_error_when_version_file_is_empty() {
    // look mama, no version file, it will boom!
    let env = Environment::new("dev".to_string());
    let version_file_path = empty_version_file_path("rustic.version".to_string()).unwrap();

    let result = Version::current_version(env.clone(), &version_file_path);

    //assert_ok!(result); // uncomment it to see the failing test with msg:
    // `assertion failed, expected Ok(..), got Err("No build number specified in 'rustic.version'")`
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