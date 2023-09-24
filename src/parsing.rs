use crate::regex::re;
use indexmap::IndexMap;
use std::{
    path::{Component, Path},
    time::Duration,
};

/// The core parsing function that extracts all the information from `cargo test`
/// but filters out empty tests.
pub fn parse_cargo_test<'s>(stderr: &'s str, stdout: &'s str) -> TestRunners<'s> {
    use TestType::*;

    let mut pkg = None;
    TestRunners::new(
        parse_cargo_test_with_empty_ones(stderr, stdout)
            .filter_map(|(runner, info)| {
                match runner.ty {
                    UnitLib | UnitBin => pkg = Some(runner.src.bin_name),
                    Doc => pkg = Some("Doc"),
                    _ => (),
                }
                if info.stats.total == 0 {
                    // don't show test types that have no tests
                    None
                } else {
                    Some((pkg, runner, info))
                }
            })
            .collect(),
    )
}

/// The core parsing function that extracts all the information from `cargo test`.
pub fn parse_cargo_test_with_empty_ones<'s>(
    stderr: &'s str,
    stdout: &'s str,
) -> impl Iterator<Item = (TestRunner<'s>, TestInfo<'s>)> {
    let parsed_stderr = parse_stderr(stderr);
    let parsed_stdout = parse_stdout(stdout);
    assert_eq!(
        parsed_stderr.len(),
        parsed_stdout.len(),
        "the amount of test runners from stderr should equal to that from stdout"
    );
    parsed_stderr.into_iter().zip(parsed_stdout)
}

/// Pkg/crate name determined by the unittests.
/// It's possible to be None because unittests can be omitted in `cargo test`
/// and we can't determine which crate emits the tests.
/// This mainly affacts how the project structure looks like specifically the root node.
pub type Pkg<'s> = Option<Text<'s>>;

/// All the test runners with original display order but filtering empty types out.
#[derive(Debug, Default)]
pub struct TestRunners<'s> {
    pub pkgs: IndexMap<Pkg<'s>, PkgTest<'s>>,
}

impl<'s> TestRunners<'s> {
    pub fn new(v: Vec<(Pkg<'s>, TestRunner<'s>, TestInfo<'s>)>) -> TestRunners<'s> {
        let mut runners = TestRunners::default();
        for (pkg, runner, info) in v {
            match runners.pkgs.entry(pkg) {
                indexmap::map::Entry::Occupied(mut item) => {
                    item.get_mut().push(runner, info);
                }
                indexmap::map::Entry::Vacant(empty) => {
                    empty.insert(PkgTest::new(runner, info));
                }
            }
        }
        runners
    }
}

/// The raw output from `cargo test`.
pub type Text<'s> = &'s str;

/// Tests information in a pkg/crate.
/// For doc test type, tests from multiple crates are considered
/// to be under a presumed Doc pkg.
#[derive(Debug, Default)]
pub struct PkgTest<'s> {
    pub inner: Vec<Data<'s>>,
}

impl<'s> PkgTest<'s> {
    pub fn new(runner: TestRunner<'s>, info: TestInfo<'s>) -> PkgTest<'s> {
        PkgTest {
            inner: vec![Data { runner, info }],
        }
    }
    pub fn push(&mut self, runner: TestRunner<'s>, info: TestInfo<'s>) {
        self.inner.push(Data { runner, info });
    }
}

/// Information extracted from stdout & stderr.
#[derive(Debug)]
pub struct Data<'s> {
    pub runner: TestRunner<'s>,
    pub info: TestInfo<'s>,
}

/// A test runner determined by the type and binary & source path.
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct TestRunner<'s> {
    pub ty: TestType,
    pub src: Src<'s>,
}

/// All the information reported by a test runner.
#[derive(Debug)]
pub struct TestInfo<'s> {
    /// Raw test information from stdout.
    pub raw: Text<'s>,
    pub stats: Stats,
    pub parsed: ParsedCargoTestOutput<'s>,
}

/// Types of a test.
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum TestType {
    UnitLib,
    UnitBin,
    Doc,
    Tests,
    Examples,
    Benches,
}

/// Source location and binary name for a test runner.
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct Src<'s> {
    /// Path of source code (except Doc type) which is relative to its crate
    /// rather than root of project.
    ///
    /// This means it's possible to see same path from different crates.
    pub src_path: Text<'s>,
    /// Name from the path of test runner binary. The path usually starts with `target/`.
    ///
    /// But this field doesn't contain neither the `target/...` prefix nor hash postfix,
    /// so it's possible to see same path from different crates.
    pub bin_name: Text<'s>,
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

pub fn parse_stderr(stderr: &str) -> Vec<TestRunner> {
    fn parse_stderr_inner<'s>(cap: &regex_lite::Captures<'s>) -> TestRunner<'s> {
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
            TestRunner {
                ty,
                src: Src {
                    src_path: path,
                    bin_name: pkg,
                },
            }
        } else if let Some(s) = cap.name("doc").map(|m| m.as_str()) {
            TestRunner {
                ty: TestType::Doc,
                src: Src {
                    src_path: s,
                    bin_name: s,
                },
            }
        } else {
            unimplemented!();
        }
    }
    re().ty
        .captures_iter(stderr)
        .map(|cap| parse_stderr_inner(&cap))
        .collect::<Vec<_>>()
}

pub fn parse_stdout(stdout: &str) -> Vec<TestInfo> {
    fn parse_stdout_except_head(raw: &str) -> Option<(Vec<Text>, Text, Stats, Text)> {
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
            let stats_start = cap.get(0)?.start();
            Some((tree, detail[..stats_start].trim(), stats, raw))
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

    // check the amount of tests
    let parsed_amount_from_head: Vec<_> = split.iter().map(|v| v.2).collect();
    let stats_total: Vec<_> = parsed_stdout.iter().map(|v| v.2.total).collect();
    assert_eq!(
        parsed_amount_from_head, stats_total,
        "the parsed amount of running tests {parsed_amount_from_head:?} \
         should equal to the number in stats.total {stats_total:?}"
    );

    split
        .iter()
        .zip(parsed_stdout)
        .map(|(head_info, v)| TestInfo {
            parsed: ParsedCargoTestOutput {
                head: head_info.1,
                tree: v.0,
                detail: v.1,
            },
            stats: v.2,
            raw: v.3,
        })
        .collect()
}
