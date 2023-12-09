use std::error::Error;

use serde::{Deserialize, Serialize};

pub type TestResult = Result<(), Box<dyn Error>>;

pub fn assert_bijective_relationship_between_encoder_and_decoder<'a, A>(
    original: &'a str,
) -> TestResult
where
    A: Serialize + Deserialize<'a> + PartialEq + std::fmt::Debug,
{
    let decoded = serde_json::from_str::<A>(&original)?;
    dbg!(&decoded);
    let roundtrip = serde_json::to_string(&decoded)?;

    Ok(assert_eq!(
        original
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>(),
        roundtrip
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>()
    ))
}
