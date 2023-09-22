use cargo_pretty_test::{app::make_pretty, lazy_static, regex::parse_cargo_test};
use insta::{assert_debug_snapshot as snap, assert_display_snapshot as shot};
use regex_lite::Regex;
use std::{process::Command, sync::OnceLock};

lazy_static! {
    cargo_test -> &'static str, String, {
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
    };
}
lazy_static! {
    parsed_cargo_test, (&'static str, Vec<&'static str>, &'static str), {
        parse_cargo_test(cargo_test())
    };
}

#[test]
fn check_text() {
    let (head, tree, detail) = parsed_cargo_test();
    shot!(head, @"running 7 tests");
    snap!(tree, @r###"
    [
        "test submod::ignore_without_reason ... ignored",
        "test submod::normal_test ... ok",
        "test submod::panic::panicked ... FAILED",
        "test works ... ok",
    ]
    "###);

    #[cfg(RUSTC_IS_NIGHTLY)]
    shot!(detail, @r###"
    failures:

    ---- submod::panic::panicked stdout ----
    thread 'submod::panic::panicked' panicked at tests/integration/src/lib.rs:8:13:
    explicit panic
    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


    failures:
        submod::panic::panicked

    test result: FAILED. 4 passed; 1 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.00s
    "###);

    #[cfg(not(RUSTC_IS_NIGHTLY))]
    shot!(detail, @r###"
    failures:

    ---- submod::panic::panicked stdout ----
    thread 'submod::panic::panicked' panicked at 'explicit panic', tests/integration/src/lib.rs:8:13
    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


    failures:
        submod::panic::panicked

    test result: FAILED. 4 passed; 1 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.00s
    "###);
}

#[test]
fn test_tree() {
    let lines = parsed_cargo_test().1.iter().copied();
    shot!(make_pretty(lines).unwrap(), @r###"
    test
    ├── submod
    │   ├─ ❌ ignore_without_reason
    │   ├─ ✅ normal_test
    │   └── panic
    │       └─ ❌ panicked
    └─ ✅ works
    "###);
}
