use cargo_pretty_test::{
    app::make_pretty,
    lazy_static,
    regex::{parse_cargo_test_output, ParsedCargoTestOutput},
};
use insta::{assert_debug_snapshot as snap, assert_display_snapshot as shot};
use regex_lite::Regex;
use std::process::Command;

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
    parsed_cargo_test, ParsedCargoTestOutput<'static>, {
        parse_cargo_test_output(cargo_test())
    };
}

#[test]
fn snapshot_testing_for_parsed_output() {
    let ParsedCargoTestOutput { head, tree, detail } = parsed_cargo_test();
    shot!(head, @"running 8 tests");
    snap!(tree, @r###"
    [
        "test submod::ignore ... ignored, reason",
        "test submod::ignore_without_reason ... ignored",
        "test submod::normal_test ... ok",
        "test submod::panic::panicked ... FAILED",
        "test submod::panic::should_panic - should panic ... ok",
        "test submod::panic::should_panic_but_didnt - should panic ... FAILED",
        "test submod::panic::should_panic_without_reanson - should panic ... ok",
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
    thread 'submod::panic::panicked' panicked at 'explicit panic', tests/integration/src/lib.rs:9:13
    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

    ---- submod::panic::should_panic_but_didnt stdout ----
    note: test did not panic as expected

    failures:
        submod::panic::panicked
        submod::panic::should_panic_but_didnt

    test result: FAILED. 4 passed; 2 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.00s
    "###);
}

#[test]
fn snapshot_testing_for_pretty_output() {
    let lines = parsed_cargo_test().tree.iter().copied();
    shot!(make_pretty(lines).unwrap(), @r###"
    test
    ├── submod
    │   ├─ 🔕 ignore
    │   ├─ 🔕 ignore_without_reason
    │   ├─ ✅ normal_test
    │   └── panic
    │       ├─ ❌ panicked
    │       ├─ ✅ should_panic
    │       ├─ ❌ should_panic_but_didnt
    │       └─ ✅ should_panic_without_reanson
    └─ ✅ works
    "###);
}
