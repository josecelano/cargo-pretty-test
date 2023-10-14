use cargo_pretty_test::{
    lazy_static,
    parsing::{parse_stdout, ParsedCargoTestOutput, TestInfo},
    prettify::make_pretty,
};
use insta::{assert_debug_snapshot as snap, assert_display_snapshot as shot};
use regex_lite::Regex;
use std::process::Command;

struct Cache {
    /// Output from `cargo test`, but with unimportant texts modified.
    #[allow(dead_code)]
    raw_output: &'static str,
    /// Parsed information.
    info: Vec<TestInfo<'static>>,
}
lazy_static! {
    parsed_cargo_test, Cache, {
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
        let raw_output = strip_backtrace.into_owned().leak();
        let info = parse_stdout(raw_output).unwrap();
        Cache { raw_output, info }
    };
}

#[test]
fn snapshot_testing_for_parsed_output() {
    let ParsedCargoTestOutput { head, tree, detail } = &parsed_cargo_test().info[0].parsed;
    shot!(head, @"running 8 tests");
    // tree is sorted when parsing
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

    // test order is in random, so sort failure details here
    let failure_tests = Regex::new(r"(?m)^failures:$")
        .unwrap()
        .split(detail)
        .nth(1)
        .unwrap()
        .trim();
    let pos: Vec<_> = Regex::new(r"(?m)^---- \S+ stdout ----$")
        .unwrap()
        .find_iter(failure_tests)
        .map(|cap| cap.start())
        .collect();
    let mut failure_info: Vec<_> = pos
        .iter()
        .copied()
        .zip(pos.iter().copied().skip(1).chain(Some(failure_tests.len())))
        .map(|(a, b)| &failure_tests[a..b])
        .collect();
    failure_info.sort_unstable();
    snap!(failure_info, @r###"
    [
        "---- submod::panic::panicked stdout ----\nthread 'submod::panic::panicked' panicked at tests/integration/src/lib.rs:11:13:\nexplicit panic\n\n",
        "---- submod::panic::should_panic_but_didnt stdout ----\nnote: test did not panic as expected",
    ]
    "###);
}

#[test]
fn snapshot_testing_for_pretty_output() {
    let lines = parsed_cargo_test().info[0].parsed.tree.iter().copied();
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
