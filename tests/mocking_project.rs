use cargo_pretty_test::{
    lazy_static,
    parsing::{parse_stdout, ParsedCargoTestOutput, TestInfo},
    prettify::make_pretty,
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
        let modified_time = Regex::new(r"(?<raw>; finished in) (\S+)")
            .unwrap()
            .replace(text.trim(), "$raw 0.00s");
        let strip_backtrace = Regex::new("note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace\n")
            .unwrap()
            .replace_all(&modified_time, "");
        strip_backtrace.into_owned()
    };
}
lazy_static! {
    parsed_cargo_test, Vec<TestInfo<'static>>, {
        parse_stdout(cargo_test())
    };
}

fn is_nightly() -> bool {
    String::from_utf8(Command::new("rustc").arg("-V").output().unwrap().stdout)
        .unwrap()
        .contains("nightly")
}

#[test]
fn snapshot_testing_for_parsed_output() {
    let ParsedCargoTestOutput { head, tree, detail } = &parsed_cargo_test()[0].parsed;
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

    if is_nightly() {
        shot!(detail, @r###"
        failures:

        ---- submod::panic::panicked stdout ----
        thread 'submod::panic::panicked' panicked at tests/integration/src/lib.rs:9:13:
        explicit panic

        ---- submod::panic::should_panic_but_didnt stdout ----
        note: test did not panic as expected

        failures:
            submod::panic::panicked
            submod::panic::should_panic_but_didnt
        "###);
    } else {
        shot!(detail, @r###"
        failures:

        ---- submod::panic::panicked stdout ----
        thread 'submod::panic::panicked' panicked at 'explicit panic', tests/integration/src/lib.rs:9:13

        ---- submod::panic::should_panic_but_didnt stdout ----
        note: test did not panic as expected

        failures:
            submod::panic::panicked
            submod::panic::should_panic_but_didnt
        "###);
    }
}

#[test]
fn snapshot_testing_for_pretty_output() {
    let lines = parsed_cargo_test()[0].parsed.tree.iter().copied();
    shot!(make_pretty("test", lines).unwrap(), @r###"
    test
    ├── submod
    │   ├─ 🔕 ignore
    │   ├─ 🔕 ignore_without_reason
    │   ├─ ✅ normal_test
    │   └── panic
    │       ├─ ❌ panicked
    │       ├─ ✅ should_panic - should panic
    │       ├─ ❌ should_panic_but_didnt - should panic
    │       └─ ✅ should_panic_without_reanson - should panic
    └─ ✅ works
    "###);
}
