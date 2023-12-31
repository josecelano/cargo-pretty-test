use cargo_pretty_test::{
    fetch::parse_cargo_test_output,
    parsing::{parse_cargo_test_with_empty_ones, TestType},
};
use insta::assert_display_snapshot;
use pretty_assertions::assert_eq;

const STDERR: &str = "\
    Finished test [unoptimized + debuginfo] target(s) in 0.00s
     Running unittests src/lib.rs (target/debug/deps/cargo_pretty_test-9b4400a4dee777d5)
     Running unittests src/main.rs (target/debug/deps/cargo_pretty_test-269f1bfba2d44b88)
     Running tests/golden_master_test.rs (target/debug/deps/golden_master_test-4deced585767cf11)
     Running tests/mocking_project.rs (target/debug/deps/mocking_project-bd11dfdabc9464fa)
   Doc-tests cargo-pretty-test\
";

// Note: the doc tests are from `test/integration`, but for simplicity, pretend they are
// for cargo-pretty-test.
const STDOUT: &str = "
running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
test golden_master_test ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s


running 2 tests
test snapshot_testing_for_parsed_output ... ok
test snapshot_testing_for_pretty_output ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s


running 14 tests
test tests/integration/src/lib.rs - (line 1) ... ok
test src/lib.rs - attribute::edition2018 (line 100) ... ok
test src/lib.rs - attribute::ignore (line 76) ... ignored
test src/lib.rs - attribute::no_run (line 86) - compile ... ok
test src/lib.rs - attribute::should_compile_fail (line 90) - compile fail ... ok
test src/lib.rs - attribute::should_compile_fail_but_didnt (line 96) - compile fail ... FAILED
test src/lib.rs - attribute::should_panic (line 80) ... ok
test src/lib.rs - empty_doc_mod (line 41) ... ok
test src/lib.rs - empty_doc_mod::Item (line 48) ... ok
test src/lib.rs - empty_doc_mod::private_mod (line 44) ... ok
test src/lib.rs - normal_doc_mod (line 55) ... ok
test src/lib.rs - normal_doc_mod::Item (line 69) ... ok
test src/lib.rs - normal_doc_mod::private_mod (line 59) ... ok
test src/lib.rs - normal_doc_mod::private_mod::Item (line 63) ... ok

test result: ok. 12 passed; 1 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s
";

#[test]
fn parse_stderr_stdout() {
    use TestType::*;

    let parsed_tests_info: Vec<_> = parse_cargo_test_with_empty_ones(STDERR, STDOUT)
        .unwrap()
        .collect();
    let parsed_stderr = parsed_tests_info
        .iter()
        .map(|(r, _)| (r.ty, r.src.src_path, r.src.bin_name))
        .collect::<Vec<_>>();
    assert_eq!(
        parsed_stderr,
        [
            (UnitLib, "src/lib.rs", "cargo_pretty_test",),
            (UnitBin, "src/main.rs", "cargo_pretty_test",),
            (Tests, "tests/golden_master_test.rs", "golden_master_test",),
            (Tests, "tests/mocking_project.rs", "mocking_project",),
            (Doc, "cargo-pretty-test", "cargo-pretty-test",),
        ]
    );

    println!(
        "{:#?}",
        parsed_tests_info
            .iter()
            .filter(|(_, info)| !info.parsed.tree.is_empty())
            .map(|(r, i)| (
                r.ty,
                r.src.src_path,
                r.src.bin_name,
                &i.parsed.tree,
                &i.stats
            ))
            .collect::<Vec<_>>(),
    );
}

#[test]
fn display_test_tree() {
    let (tree, stats) = parse_cargo_test_output(STDERR, STDOUT).unwrap();
    assert_display_snapshot!(tree, @r###"
    Generated by cargo-pretty-test
    ├── (OK) cargo_pretty_test ... (3 tests in 0.03s: ✅ 3)
    │   ├── (OK) tests/golden_master_test.rs ... (1 tests in 0.01s: ✅ 1)
    │   │   └─ ✅ golden_master_test
    │   └── (OK) tests/mocking_project.rs ... (2 tests in 0.02s: ✅ 2)
    │       ├─ ✅ snapshot_testing_for_parsed_output
    │       └─ ✅ snapshot_testing_for_pretty_output
    └── (OK) Doc Tests ... (14 tests in 0.00s: ✅ 12; ❌ 1; 🔕 1)
        └── (OK) cargo-pretty-test ... (14 tests in 0.00s: ✅ 12; ❌ 1; 🔕 1)
            ├── src/lib.rs - attribute
            │   ├─ ✅ edition2018 (line 100)
            │   ├─ 🔕 ignore (line 76)
            │   ├─ ✅ no_run (line 86) - compile
            │   ├─ ✅ should_compile_fail (line 90) - compile fail
            │   ├─ ❌ should_compile_fail_but_didnt (line 96) - compile fail
            │   └─ ✅ should_panic (line 80)
            ├── src/lib.rs - empty_doc_mod
            │   ├─ ✅ Item (line 48)
            │   └─ ✅ private_mod (line 44)
            ├─ ✅ src/lib.rs - empty_doc_mod (line 41)
            ├── src/lib.rs - normal_doc_mod
            │   ├─ ✅ Item (line 69)
            │   ├── private_mod
            │   │   └─ ✅ Item (line 63)
            │   └─ ✅ private_mod (line 59)
            ├─ ✅ src/lib.rs - normal_doc_mod (line 55)
            └─ ✅ tests/integration/src/lib.rs - (line 1)
    "###);

    let total_time = 0.03;
    assert!(
        (stats.finished_in.as_secs_f32() - total_time).abs() < f32::EPSILON,
        "total time in running all tests should be {total_time}"
    );
}
