use regex_lite::Regex;
use std::{
    path::{Component, Path},
    time::Duration,
};

// Lazily initialize a global variable.
#[doc(hidden)]
#[macro_export]
macro_rules! lazy_static {
    ($f:ident, $t:ty, $e:block $(;)?) => {
        lazy_static! { $f -> &'static $t, $t, $e }
    };
    ($f:ident -> $ret:ty, $t:ty, $e:block $(;)?) => {
        #[allow(dead_code)]
        fn $f() -> $ret {
            static TMP: ::std::sync::OnceLock<$t> = ::std::sync::OnceLock::new();
            TMP.get_or_init(|| $e)
        }
    };
    ($( $f:ident, $t:ty, $e:block );+ $(;)?) => {
        $( lazy_static! { $f, $t, $e } )+
    };
    ($( $f:ident -> $ret:ty, $t:ty, $e:block );+ $(;)?) => {
        $( lazy_static! { $f -> $ret, $t, $e } )+
    };
}

const RE_ERROR: &str = "regex pattern error";

pub struct Re {
    pub ty: Regex,
    pub head: Regex,
    pub tree: Regex,
    pub stats: Regex,
}

pub fn re() -> Re {
    Re {
        // Running unittests src/lib.rs (target/debug/deps/cargo_pretty_test-9b4400a4dee777d5)
        // Running unittests src/main.rs (target/debug/deps/cargo_pretty_test-269f1bfba2d44b88)
        // Running tests/golden_master_test.rs (target/debug/deps/golden_master_test-4deced585767cf11)
        // Running tests/mocking_project.rs (target/debug/deps/mocking_project-bd11dfdabc9464fa)
        // Doc-tests cargo-pretty-test
        ty: Regex::new(r"(?m)^\s+(Running (?P<is_unit>unittests )?(?P<path>\S+) \((?P<pkg>.*)\))|(Doc-tests (?P<doc>\S+))$")
            .expect(RE_ERROR),
        // running 0 tests; running 1 test; running 2 tests; ...
        head: Regex::new(r"running (?P<amount>\d+) tests?").expect(RE_ERROR),
        // Common test info:
        // test submod::normal_test ... ok
        // test submod::ignore ... ignored, reason
        // test submod::ignore_without_reason ... ignored
        // test submod::panic::should_panic - should panic ... ok
        // test submod::panic::should_panic_without_reanson - should panic ... ok
        tree: Regex::new(r"(?m)^test \S+( - should panic)? \.\.\. \S+(, .*)?$").expect(RE_ERROR),
        // test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
        stats: Regex::new(r"(?mx)
            ^test\ result:\ (?P<ok>\S+)\.
            \ (?P<passed>\d+)\ passed;
            \ (?P<failed>\d+)\ failed;
            \ (?P<ignored>\d+)\ ignored;
            \ (?P<measured>\d+)\ measured;
            \ (?P<filtered>\d+)\ filtered\ out;
            \ finished\ in\ (?P<time>\S+)s$").expect(RE_ERROR),
      }
}

/// All the information reported by a test runner.
#[derive(Debug)]
pub struct TestInfo<'s> {
    pub ty: TestType,
    /// Path to test & pkg name
    pub src: Src<'s>,
    /// Raw test information from stdout
    pub raw: Text<'s>,
    pub stat: Stats,
    pub parsed: ParsedCargoTestOutput<'s>,
}

impl TestInfo<'_> {
    pub fn parse_cargo_test<'s>(stderr: &'s str, stdout: &'s str) -> Vec<TestInfo<'s>> {
        let parsed_stderr = parse_stderr(stderr);
        let parsed_stdout = parse_stdout(stdout);
        assert_eq!(
            parsed_stderr.len(),
            parsed_stdout.len(),
            "the amount of test runners from stderr should equal to that from stdout"
        );
        parsed_stderr
            .into_iter()
            .zip(parsed_stdout)
            .map(|(a, b)| TestInfo {
                ty: a.0,
                src: a.1,
                raw: b.2,
                stat: b.1,
                parsed: b.0,
            })
            .collect()
    }
}

pub type Text<'s> = &'s str;

#[derive(Debug)]
pub struct Src<'s> {
    pub path: Text<'s>,
    pub pkg: Text<'s>,
}

/// Type of test.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TestType {
    UnitLib,
    UnitBin,
    Doc,
    Tests,
    Examples,
    Benches,
}

/// Statistics of test.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Stats {
    pub ok: bool,
    pub total: u32,
    pub passed: u32,
    pub ignored: u32,
    pub measured: u32,
    pub filtered_out: u32,
    pub finished_in: Duration,
}

impl std::ops::Add<Stats> for Stats {
    type Output = Stats;

    fn add(self, rhs: Stats) -> Self::Output {
        Stats {
            ok: self.ok && rhs.ok,
            total: self.total + rhs.total,
            passed: self.passed + rhs.passed,
            ignored: self.ignored + rhs.ignored,
            measured: self.measured + rhs.measured,
            filtered_out: self.filtered_out + rhs.filtered_out,
            finished_in: self.finished_in + rhs.finished_in,
        }
    }
}

/// Output from one test runner.
#[derive(Debug)]
pub struct ParsedCargoTestOutput<'s> {
    pub head: Text<'s>,
    pub tree: Vec<Text<'s>>,
    pub detail: Text<'s>,
}

pub type ParsedStderr<'s> = (TestType, Src<'s>);

pub fn parse_stderr(stderr: &str) -> Vec<ParsedStderr> {
    fn parse_stderr_inner<'s>(cap: &regex_lite::Captures<'s>) -> (TestType, Src<'s>) {
        if let Some((path, pkg)) = cap.name("path").zip(cap.name("pkg")) {
            let path = path.as_str();
            let path_norm = Path::new(path);
            let ty = if cap.name("is_unit").is_some() {
                if path_norm
                    .components()
                    .take(2)
                    .map(Component::as_os_str)
                    .eq(["src", "lib.rs"])
                {
                    TestType::UnitLib
                } else {
                    TestType::UnitBin
                }
            } else {
                let Some(base_dir) = path_norm
                    .components()
                    .next()
                    .and_then(|p| p.as_os_str().to_str())
                else {
                    unimplemented!("failed to parse the type of test: {path:?}")
                };
                match base_dir {
                    "tests" => TestType::Tests,
                    "examples" => TestType::Examples,
                    "benches" => TestType::Benches,
                    _ => unimplemented!("failed to parse the type of test: {path:?}"),
                }
            };

            // e.g. target/debug/deps/cargo_pretty_test-xxxxxxxxxxxxxxxx
            let mut pkg_comp = Path::new(pkg.as_str()).components();
            match pkg_comp.next().map(|p| p.as_os_str() == "target") {
                Some(true) => (),
                _ => unimplemented!("failed to parse the location of test: {pkg:?}"),
            }
            let pkg = pkg_comp.nth(2).unwrap().as_os_str().to_str().unwrap();
            let pkg = &pkg[..pkg
                .find('-')
                .expect("pkg should be of `pkgname-hash` pattern")];
            (ty, Src { path, pkg })
        } else if let Some(s) = cap.name("doc").map(|m| m.as_str()) {
            (TestType::Doc, Src { path: s, pkg: s })
        } else {
            unimplemented!();
        }
    }
    re().ty
        .captures_iter(stderr)
        .map(|cap| parse_stderr_inner(&cap))
        .collect::<Vec<_>>()
}

pub type ParsedStdout<'s> = (ParsedCargoTestOutput<'s>, Stats, Text<'s>);

pub fn parse_stdout(stdout: &str) -> Vec<ParsedStdout> {
    fn parse_stdout_except_head(raw: &str) -> Option<(Vec<&str>, &str, Stats, &str)> {
        fn parse_tree_detail(text: &str) -> (Vec<Text>, Text) {
            let line: Vec<_> = re().tree.find_iter(text).collect();
            let tree_end = line.last().map_or(0, |cap| cap.end() + 1);
            let mut tree: Vec<_> = line.into_iter().map(|cap| cap.as_str()).collect();
            tree.sort_unstable();
            (tree, text[tree_end..].trim())
        }

        if raw.is_empty() {
            None
        } else {
            let (tree, detail) = parse_tree_detail(raw);
            let cap = re().stats.captures(detail)?;
            let stats = Stats {
                ok: cap.name("ok").map(|ok| ok.as_str() == "ok")?,
                total: tree.len().try_into().ok()?,
                passed: cap.name("passed")?.as_str().parse().ok()?,
                ignored: cap.name("ignored")?.as_str().parse().ok()?,
                measured: cap.name("measured")?.as_str().parse().ok()?,
                filtered_out: cap.name("filtered")?.as_str().parse().ok()?,
                finished_in: Duration::from_secs_f32(cap.name("time")?.as_str().parse().ok()?),
            };
            Some((tree, detail, stats, raw))
        }
    }

    let split: Vec<_> = re()
        .head
        .captures_iter(stdout)
        .filter_map(|cap| {
            let full = cap.get(0)?;
            Some((
                full.start(),
                full.as_str(),
                cap.name("amount")?.as_str().parse::<u32>().ok()?,
            ))
        })
        .collect();
    assert!(
        !split.is_empty(),
        "{stdout} should contain `running (?P<amount>\\d+) tests?` pattern"
    );
    let parsed_stdout = if split.len() == 1 {
        vec![parse_stdout_except_head(stdout).unwrap()]
    } else {
        let start = split.iter().map(|v| v.0);
        let end = start.clone().skip(1).chain([stdout.len()]);
        start
            .zip(end)
            .filter_map(|(a, b)| {
                let src = &stdout[a..b];
                parse_stdout_except_head(src)
            })
            .collect::<Vec<_>>()
    };
    assert!(
        {
            let parsed_amount_from_head = split.iter().map(|v| v.2);
            parsed_amount_from_head.eq(parsed_stdout.iter().map(|v| v.2.total))
        },
        "the parsed amount of running tests should equal to the number in stats.total"
    );
    split
        .iter()
        .zip(parsed_stdout)
        .map(|(head_info, v)| {
            (
                ParsedCargoTestOutput {
                    head: head_info.1,
                    tree: v.0,
                    detail: v.1,
                },
                v.2,
                v.3,
            )
        })
        .collect()
}
