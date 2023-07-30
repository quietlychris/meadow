use xshell::{cmd, Shell};

// Adapted from the Bevy CI pipeline

fn main() {
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

    // These tests are already run on the CI
    // Using a double-negative here allows end-users to have a nicer experience
    // as we can pass in the extra argument to the CI script
    let args: Vec<String> = std::env::args().collect();
    if args.get(1) != Some(&"nonlocal".to_string()) {
        // Run tests
        cmd!(sh, "cargo test --workspace -- --nocapture --test-threads=1")
            .run()
            .expect("Please fix failing tests in output above.");

        // Run doc tests: these are ignored by `cargo test`
        cmd!(sh, "cargo test --doc --workspace")
            .run()
            .expect("Please fix failing doc-tests in output above.");
    }
}
