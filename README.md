# rustic-sketch

## Rust in Action

Examples of various Rust features in action.

### The Language

#### ? Operator
  - [#expect](tests/public_contracts.rs)
  - [#unwrap]
  - [Testing Api contracts](tests/test_kit.rs)

#### Async Closures
  - ???

#### Getters
  - [Version]()
  - For derived getters, see `getset` crait.

#### Iterators
 - [chars()](tests/test_kit.rs)
 - [try_for_each()](tests/test_kit.rs)

#### Lifecycle Management:
 - [Testing contracts](tests/test_kit.rs)

#### Macros:
 - [dbg!](tests/public_contracts.rs)
 - [format!](tests/public_contracts.rs)

#### Types
 - [Associated Types](src/routes/health_status/model.rs)
 - [Type Alias](tests/test_kit.rs)

### Interesting Crates

#### derive-more
 - [Clone]()
 - [Constructor]()
 - [Debug]()
 - [Display]()
 - [Eq]()
 - [Error]()
 - [Hash]()
 - [PartialEq]()

#### getset
 - [#[get = "pub"]]()

#### Serde
 - [Derived Serialization and Deserialization](src/routes/health_status/model.rs)
 - [Custom Deserialization](src/routes/health_status/model.rs)
 - [Custom Serialization](src/routes/health_status/model.rs)

#### warp
 - ???

### Tests
 - [Integration test](tests/version_test.rs)
 - Property-based test:
   - [prop_compose!](src/health_check/version.rs)
   - [proptest! macro doesn't play nice with tokio::test](https://github.com/proptest-rs/proptest/issues/179).

 - [Table-driven test](tests/public_contracts.rs)
 - [Unit test](src/routes/health_status.rs)
