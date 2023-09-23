use std::process::{Command, Output};

/// Collect arguments and forward them to `cargo test`.
///
/// Note: This filters some arguments that mess up the output, like
/// `--nocapture` which prints in the status part and hinders parsing.
pub fn cargo_test() -> Output {
    let passin: Vec<_> = std::env::args().collect();
    let forward = if passin
        .get(..2)
        .is_some_and(|v| v[0].ends_with("cargo-pretty-test") && v[1] == "pretty-test")
    {
        // `cargo pretty-test` yields ["path-to-cargo-pretty-test", "pretty-test", rest]
        &passin[2..]
    } else {
        // `cargo-pretty-test` yields ["path-to-cargo-pretty-test", rest]
        &passin[1..]
    };
    let args = forward.iter().filter(|arg| *arg != "--nocapture");
    Command::new("cargo")
        .arg("test")
        .args(args)
        .output()
        .expect("`cargo test` failed")
}
