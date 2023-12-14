# Workstation

## Required Tools

- [Git](https://git-scm.com/)
- [Docker](https://www.docker.com/)
- [Docker Compose](https://docs.docker.com/compose/)
- [Rust](https://rustup.rs/)

## Recommended Tools

- [Visual Studio Code](https://code.visualstudio.com/)
- [rust-analyzer](https://code.visualstudio.com/docs/languages/rust#_2-install-the-rustanalyzer-extension)

## Useful Commands

#### Docker:
 - `docker run --rm -p3030:3030 rafaelfiume/rustic-sketch:latest --name rustic`

### rustc & rustup

- `rustc --version [--verbose]
- `rustup doc`
- `rustup update`

### cargo

- `cargo build [--release]`
- `cargo fix --edition`
- `cargo fmt`
- `cargo new hello_world`
- `cargo run`
- `cargo test -- --show-output`    // Show output of sucessful tests
- `cargo test --test version_test` // Integration test + specific test

## Useful Resources

Build-dependencies:
 - [git2-rs](https://crates.io/crates/git2)

Dependencies:
 - [Serde](https://serde.rs/)
 - [Serde-json - crates.io](https://crates.io/crates/serde_json)
 - [SQLx - crates.io](https://crates.io/crates/sqlx)
 - [derive(Error) - crates.io](https://crates.io/crates/thiserror) // Shall I use it?
 - [Tokio](https://tokio.rs/tokio/tutorial)
 - [Tokio - crates.io](https://crates.io/crates/tokio)
 - [warp](https://docs.rs/warp/latest/warp/test/index.html)
 - [warp - crates.io](https://crates.io/crates/warp)
 - [warp - Examples](https://github.com/seanmonstar/warp/blob/master/examples/todos.rs)

Dev-dependencies:
 - [claims - crates.io](https://crates.io/crates/claims)
 - [Proptest](https://proptest-rs.github.io/proptest/proptest/getting-started.html)
 - [Proptest - crates.io](https://crates.io/crates/proptest)

Docker:
  - [5x Faster Rust Docker Builds with cargo-chef](https://www.lpalmieri.com/posts/fast-rust-docker-builds/)
  - [cargo-chef](https://github.com/LukeMathWalker/cargo-chef)
  - [Simplify Your Deployments Using the Rust Official Image](https://www.docker.com/blog/simplify-your-deployments-using-the-rust-official-image/)

Other resources:
 - [An Introduction To Property-Based Testing In Rust](https://www.lpalmieri.com/posts/an-introduction-to-property-based-testing-in-rust/)
 - [Zero to Production in Rust](https://github.com/LukeMathWalker/zero-to-production)

Feito com ❤️ por Artigiani.
