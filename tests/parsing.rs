use cargo_pretty_test::regex::{TestInfo, TestType};
use pretty_assertions::assert_eq;
use std::time::Duration;

const STDERR: &str = "\
    Finished test [unoptimized + debuginfo] target(s) in 0.00s
     Running unittests src/lib.rs (target/debug/deps/cargo_pretty_test-9b4400a4dee777d5)
     Running unittests src/main.rs (target/debug/deps/cargo_pretty_test-269f1bfba2d44b88)
     Running tests/golden_master_test.rs (target/debug/deps/golden_master_test-4deced585767cf11)
     Running tests/mocking_project.rs (target/debug/deps/mocking_project-bd11dfdabc9464fa)
   Doc-tests cargo-pretty-test\
";

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


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
";

#[test]
fn parse_stderr_stdout() {
    use TestType::*;

    let parsed_tests_info = TestInfo::parse_cargo_test_with_empty_ones(STDERR, STDOUT);
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

    let total_time = 0.03;
    assert!(
        (parsed_tests_info
            .iter()
            .map(|(_, v)| v.stat.finished_in)
            .sum::<Duration>()
            .as_secs_f32()
            - total_time)
            .abs()
            < f32::EPSILON,
        "total time in running all tests should be {total_time}"
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
                &i.stat
            ))
            .collect::<Vec<_>>(),
    );
}
