use cargo_pretty_test::{app::make_pretty, regex::get_lines_of_tests};
use insta::{assert_debug_snapshot as snap, assert_display_snapshot as shot};
use regex_lite::Regex;
use std::{process::Command, sync::OnceLock};

fn cargo_test() -> &'static str {
    static TEXT: OnceLock<String> = OnceLock::new();
    TEXT.get_or_init(|| {
        let output = Command::new("cargo")
            .args(["test", "-p", "integration"])
            .output()
            .unwrap();
        let text = String::from_utf8_lossy(&output.stdout);
        // normalize
        Regex::new(r"(?<raw>; finished in) (\S+)")
            .unwrap()
            .replace(text.trim(), "$raw 0.00s")
            .to_string()
    })
}

#[test]
fn check_text() {
    shot!(cargo_test(), @r###"
    running 7 tests
    test submod::ignore ... ignored, reason
    test submod::ignore_without_reason ... ignored
    test submod::normal_test ... ok
    test submod::panic::panicked ... FAILED
    test submod::panic::should_panic - should panic ... ok
    test submod::panic::should_panic_without_reanson - should panic ... ok
    test works ... ok

    failures:

    ---- submod::panic::panicked stdout ----
    thread 'submod::panic::panicked' panicked at tests/integration/src/lib.rs:8:13:
    explicit panic
    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


    failures:
        submod::panic::panicked

    test result: FAILED. 4 passed; 1 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.00s
    "###);
}

#[test]
fn integration() {
    let text = cargo_test();
    let lines = get_lines_of_tests(text).collect::<Vec<_>>();
    snap!(lines, @r###"
    [
        "test submod::ignore_without_reason ... ignored",
        "test submod::normal_test ... ok",
        "test submod::panic::panicked ... FAILED",
        "test works ... ok",
    ]
    "###);
    shot!(make_pretty(lines.into_iter()).unwrap(), @r###"
    test
    ├── submod
    │   ├─ ❌ ignore_without_reason
    │   ├─ ✅ normal_test
    │   └── panic
    │       └─ ❌ panicked
    └─ ✅ works
    "###);
}
