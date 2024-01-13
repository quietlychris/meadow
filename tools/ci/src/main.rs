use xshell::{cmd, Shell};

// Adapted from the Bevy CI pipeline
#[cfg(not(feature = "quic"))]
fn main() {
    println!("Running CI without QUIC enabled");
    // When run locally, results may differ from actual CI runs triggered by
    // .github/workflows/ci.yml
    // - Official CI runs latest stable
    // - Local runs use whatever the default Rust is locally
    let sh = Shell::new().expect("Couldn't create new xshell Shell environment");

    // See if any code needs to be formatted
    cmd!(sh, "cargo fmt --all -- --check")
        .run()
        .expect("Please run 'cargo fmt --all' to format your code.");

    // See if clippy has any complaints.
    // - Type complexity must be ignored because we use huge templates for queries
    cmd!(sh, "cargo clippy --package meadow --all-targets -- -D warnings -A clippy::type_complexity -W clippy::doc_markdown")
        .run()
        .expect("Please fix clippy errors in output above.");

    sh.set_var("RUSTDOCFLAGS", "-D warnings");
    // Check the documentation format is valid
    cmd!(sh, "cargo doc --package meadow")
        .run()
        .expect("Please check that all documentation follows rustdoc standards");

    // Run tests
    cmd!(sh, "cargo test --workspace -- --nocapture --test-threads=1")
        .run()
        .expect("Please fix failing tests in output above.");

    // Run doc tests: these are ignored by `cargo test`
    cmd!(sh, "cargo test --doc --workspace")
        .run()
        .expect("Please fix failing doc-tests in output above.");
}

#[cfg(feature = "quic")]
fn main() {
    println!("- Running CI with QUIC enabled");
    // When run locally, results may differ from actual CI runs triggered by
    // .github/workflows/ci.yml
    // - Official CI runs latest stable
    // - Local runs use whatever the default Rust is locally
    let sh = Shell::new().expect("Couldn't create new xshell Shell environment");

    // See if any code needs to be formatted
    cmd!(sh, "cargo fmt --all -- --check")
        .run()
        .expect("Please run 'cargo fmt --all' to format your code.");

    // See if clippy has any complaints.
    // - Type complexity must be ignored because we use huge templates for queries
    cmd!(sh, "cargo clippy --package meadow --all-targets --all-features -- -D warnings -A clippy::type_complexity -W clippy::doc_markdown")
    .run()
    .expect("Please fix clippy errors in output above.");

    sh.set_var("RUSTDOCFLAGS", "-D warnings");
    // Check the documentation format is valid
    cmd!(sh, "cargo doc --package meadow")
        .run()
        .expect("Please check that all documentation follows rustdoc standards");

    // Run tests
    cmd!(
        sh,
        "cargo test --workspace --features=quic -- --nocapture --test-threads=1"
    )
    .run()
    .expect("Please fix failing tests in output above.");

    // Run doc tests: these are ignored by `cargo test`
    cmd!(sh, "cargo test --doc --workspace")
        .run()
        .expect("Please fix failing doc-tests in output above.");
}
