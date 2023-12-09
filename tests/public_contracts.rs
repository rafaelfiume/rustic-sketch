// Each file in the `tests` dir is treatead as a different crate.
// Thus we need to use `mod` to bring modules into scope.
mod test_kit;

use std::fs;

use rustic_sketch::routes::health_status::model::ServiceStatusPayload;

use test_kit::assert_bijective_relationship_between_encoder_and_decoder;
use test_kit::TestResult;

#[test]
fn status_ok_contract() -> TestResult {
    let json = fs::read_to_string("tests/resources/contracts/health_check/status_ok.json")?;

    assert_bijective_relationship_between_encoder_and_decoder::<ServiceStatusPayload>(&json)
}
