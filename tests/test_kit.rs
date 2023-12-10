use serde::{Deserialize, Serialize};
use std::error::Error;

pub type TestResult = Result<(), Box<dyn Error>>;

/*
 * A note regarding bijective relationship an isomorphism.
 *
 * AFAIK bijective relationship and isomorphism are equivalent for ADTs, but not necessarily for other structures.
 *
 * Roughly, two structures are isomorphic if they have their structure preserved, which includes data and operations.
 * Serialisation might preserve the data but not the operations during encoding and decoding.
 *
 * Bijective relationship is a weaker condition, denoting a one-to-one mapping between two structures, with no loss of information.
 *
 * That said, encoding and decoding of ADTs does seem to form an isomorphism,
 * as the operations are preserved, e.g. equality, immutability.
 */
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
