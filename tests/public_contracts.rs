// Each file in the `tests` dir is treatead as a different crate.
// Thus we need to use `mod` to bring modules into scope.
mod test_kit;

use std::fs;

use rustic_sketch::routes::health_status::model::ServiceStatusPayload;

use test_kit::assert_bijective_relationship_between_encoder_and_decoder;
use test_kit::TestResult;

#[test]
fn status_ok_contract() -> TestResult {
    struct TestCase {
        sample: &'static str,
    }
    let test_cases = vec![
        TestCase { sample: "ok" },
        //TestCase { sample: "bum" }, // Try it to see the error
        TestCase { sample: "degraded" },
    ];
    // note that `try_for_each` will interrupt the tests on the first error
    test_cases.iter().try_for_each(|case| {
        let path_to_contract = format!(
            "tests/resources/contracts/health_check/status_{}.json",
            case.sample
        );
        let json = fs::read_to_string(&path_to_contract)
            .expect(&format!("Could not read file `{}`", &path_to_contract));

        assert_bijective_relationship_between_encoder_and_decoder::<ServiceStatusPayload>(&json)
    })
}
