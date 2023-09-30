use crate::{regex::re, Result};
use colored::{ColoredString, Colorize};
use indexmap::IndexMap;
use std::{
    path::{Component, Path},
    time::Duration,
};

/// The core parsing function that extracts all the information from `cargo test`
/// but filters out empty tests.
pub fn parse_cargo_test<'s>(stderr: &'s str, stdout: &'s str) -> Result<TestRunners<'s>> {
    use TestType::*;

    let mut pkg = None;
    Ok(TestRunners::new(
        parse_cargo_test_with_empty_ones(stderr, stdout)?
            .filter_map(|(runner, info)| {
                match runner.ty {
                    UnitLib | UnitBin => pkg = Some(runner.src.bin_name),
                    Doc => pkg = Some("Doc Tests"),
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
    ))
}

/// The core parsing function that extracts all the information from `cargo test`.
pub fn parse_cargo_test_with_empty_ones<'s>(
    stderr: &'s str,
    stdout: &'s str,
) -> Result<impl Iterator<Item = (TestRunner<'s>, TestInfo<'s>)>> {
    let parsed_stderr = parse_stderr(stderr)?;
    let parsed_stdout = parse_stdout(stdout)?;
    let err_len = parsed_stderr.len();
    let out_len = parsed_stdout.len();
    if err_len != out_len {
        return Err(format!(
            "{err_len} (the amount of test runners from stderr) should \
         equal to {out_len} (that from stdout)\n\
         stderr = {stderr:?}\nstdout = {stdout:?}"
        ));
    }
    Ok(parsed_stderr.into_iter().zip(parsed_stdout))
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
    pub stats: Stats,
}

impl<'s> PkgTest<'s> {
    pub fn new(runner: TestRunner<'s>, info: TestInfo<'s>) -> PkgTest<'s> {
        let stats = info.stats.clone();
        PkgTest {
            inner: vec![Data { runner, info }],
            stats,
        }
    }
    pub fn push(&mut self, runner: TestRunner<'s>, info: TestInfo<'s>) {
        self.stats += &info.stats;
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
    /// so it's possible to see same name from different crates.
    pub bin_name: Text<'s>,
}

/// Statistics of test.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Stats {
    pub ok: bool,
    pub total: u32,
    pub passed: u32,
    pub failed: u32,
    pub ignored: u32,
    pub measured: u32,
    pub filtered_out: u32,
    pub finished_in: Duration,
}

/// Summary text on the bottom.
impl std::fmt::Display for Stats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Stats {
            ok,
            total,
            passed,
            failed,
            ignored,
            measured,
            filtered_out,
            finished_in,
        } = *self;
        let time = finished_in.as_secs_f32();
        let fail = if failed == 0 {
            format!("{failed} failed")
        } else {
            format!("{failed} failed").red().bold().to_string()
        };
        write!(
            f,
            "Status: {}; total {total} tests in {time:.2}s: \
            {passed} passed; {fail}; {ignored} ignored; \
            {measured} measured; {filtered_out} filtered out",
            status(ok)
        )
    }
}

fn status(ok: bool) -> ColoredString {
    if ok {
        "OK".green().bold()
    } else {
        "FAIL".red().bold()
    }
}

impl Stats {
    /// Summary text at the end of root node.
    /// If the metric is zero, it won't be shown.
    pub fn inlay_summary_string(&self) -> String {
        let Stats {
            total,
            passed,
            failed,
            ignored,
            filtered_out,
            finished_in,
            ..
        } = *self;
        let time = finished_in.as_secs_f32();
        let mut metrics = Vec::with_capacity(4);
        if passed != 0 {
            metrics.push(format!("✅ {passed}"));
        };
        if failed != 0 {
            metrics.push(format!("❌ {failed}").red().to_string());
        };
        if ignored != 0 {
            metrics.push(format!("🔕 {ignored}"));
        };
        if filtered_out != 0 {
            metrics.push(format!("✂️ {filtered_out}"));
        };
        format!("{total} tests in {time:.2}s: {}", metrics.join("; "))
    }

    /// Root of test tree node depending on the test type.
    pub fn root_string(&self, pkg_name: Text) -> String {
        format!(
            "({}) {:} ... ({})",
            status(self.ok),
            pkg_name.blue().bold(),
            self.inlay_summary_string().bold()
        )
    }

    /// Subroot of test tree node depending on the test type.
    /// Compared with `Stats::root_string`, texts except status are non-bold.
    pub fn subroot_string(&self, runner_name: Text) -> String {
        format!(
            "({}) {} ... ({})",
            status(self.ok),
            runner_name,
            self.inlay_summary_string()
        )
    }
}

impl Default for Stats {
    fn default() -> Self {
        Stats {
            ok: true,
            total: 0,
            passed: 0,
            failed: 0,
            ignored: 0,
            measured: 0,
            filtered_out: 0,
            finished_in: Duration::from_secs(0),
        }
    }
}

impl std::ops::Add<&Stats> for &Stats {
    type Output = Stats;

    fn add(self, rhs: &Stats) -> Self::Output {
        Stats {
            ok: self.ok && rhs.ok,
            total: self.total + rhs.total,
            passed: self.passed + rhs.passed,
            failed: self.failed + rhs.failed,
            ignored: self.ignored + rhs.ignored,
            measured: self.measured + rhs.measured,
            filtered_out: self.filtered_out + rhs.filtered_out,
            finished_in: self.finished_in + rhs.finished_in,
        }
    }
}

impl std::ops::AddAssign<&Stats> for Stats {
    fn add_assign(&mut self, rhs: &Stats) {
        *self = &*self + rhs;
    }
}

/// Output from one test runner.
#[derive(Debug)]
pub struct ParsedCargoTestOutput<'s> {
    pub head: Text<'s>,
    pub tree: Vec<Text<'s>>,
    pub detail: Text<'s>,
}

pub fn parse_stderr(stderr: &str) -> Result<Vec<TestRunner>> {
    fn parse_stderr_inner<'s>(cap: &regex_lite::Captures<'s>) -> Result<TestRunner<'s>> {
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
                    return Err(format!("failed to parse the type of test: {path:?}"));
                };
                match base_dir {
                    "tests" => TestType::Tests,
                    "examples" => TestType::Examples,
                    "benches" => TestType::Benches,
                    _ => return Err(format!("failed to parse the type of test: {path:?}")),
                }
            };

            // e.g. target/debug/deps/cargo_pretty_test-xxxxxxxxxxxxxxxx
            let mut pkg_comp = Path::new(pkg.as_str()).components();
            match pkg_comp.next().map(|p| p.as_os_str() == "target") {
                Some(true) => (),
                _ => return Err(format!("failed to parse the location of test: {pkg:?}")),
            }
            let pkg = pkg_comp
                .nth(2)
                .ok_or_else(|| format!("can't get the third component in {pkg:?}"))?
                .as_os_str()
                .to_str()
                .ok_or_else(|| format!("can't turn os_str into str in {pkg:?}"))?;
            let pkg = &pkg[..pkg
                .find('-')
                .ok_or_else(|| format!("pkg `{pkg}` should be of `pkgname-hash` pattern"))?];
            Ok(TestRunner {
                ty,
                src: Src {
                    src_path: path,
                    bin_name: pkg,
                },
            })
        } else if let Some(s) = cap.name("doc").map(|m| m.as_str()) {
            Ok(TestRunner {
                ty: TestType::Doc,
                src: Src {
                    src_path: s,
                    bin_name: s,
                },
            })
        } else {
            Err(format!("{cap:?} is not supported to be parsed"))
        }
    }
    re().ty
        .captures_iter(stderr)
        .map(|cap| parse_stderr_inner(&cap))
        .collect::<Result<Vec<_>>>()
}

#[allow(clippy::too_many_lines)]
pub fn parse_stdout(stdout: &str) -> Result<Vec<TestInfo>> {
    fn parse_stdout_except_head(raw: &str) -> Result<(Vec<Text>, Text, Stats, Text)> {
        fn parse_tree_detail(text: &str) -> (Vec<Text>, Text) {
            let line: Vec<_> = re().tree.find_iter(text).collect();
            let tree_end = line.last().map_or(0, |cap| cap.end() + 1);
            let mut tree: Vec<_> = line.into_iter().map(|cap| cap.as_str()).collect();
            tree.sort_unstable();
            (tree, text[tree_end..].trim())
        }

        if raw.is_empty() {
            Err("raw stdout is empty".into())
        } else {
            let (tree, detail) = parse_tree_detail(raw);
            let cap = re()
                .stats
                .captures(detail)
                .ok_or_else(|| format!("`stats` is not found in {raw:?}"))?;
            let stats = Stats {
                ok: cap
                    .name("ok")
                    .ok_or_else(|| format!("`ok` is not found in {raw:?}"))?
                    .as_str()
                    == "ok",
                total: u32::try_from(tree.len()).map_err(|err| err.to_string())?,
                passed: cap
                    .name("passed")
                    .ok_or_else(|| format!("`passed` is not found in {raw:?}"))?
                    .as_str()
                    .parse::<u32>()
                    .map_err(|err| err.to_string())?,
                failed: cap
                    .name("failed")
                    .ok_or_else(|| format!("`failed` is not found in {raw:?}"))?
                    .as_str()
                    .parse::<u32>()
                    .map_err(|err| err.to_string())?,
                ignored: cap
                    .name("ignored")
                    .ok_or_else(|| format!("`ignored` is not found in {raw:?}"))?
                    .as_str()
                    .parse::<u32>()
                    .map_err(|err| err.to_string())?,
                measured: cap
                    .name("measured")
                    .ok_or_else(|| format!("`measured` is not found in {raw:?}"))?
                    .as_str()
                    .parse::<u32>()
                    .map_err(|err| err.to_string())?,
                filtered_out: cap
                    .name("filtered")
                    .ok_or_else(|| format!("`filtered` is not found in {raw:?}"))?
                    .as_str()
                    .parse::<u32>()
                    .map_err(|err| err.to_string())?,
                finished_in: Duration::from_secs_f32(
                    cap.name("time")
                        .ok_or_else(|| format!("`time` is not found in {raw:?}"))?
                        .as_str()
                        .parse::<f32>()
                        .map_err(|err| err.to_string())?,
                ),
            };
            let stats_start = cap
                .get(0)
                .ok_or_else(|| format!("can't get stats start in {raw:?}"))?
                .start();
            Ok((tree, detail[..stats_start].trim(), stats, raw))
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
    if split.is_empty() {
        return Err(format!(
            "{stdout:?} should contain `running (?P<amount>\\d+) tests?` pattern"
        ));
    }
    let parsed_stdout = if split.len() == 1 {
        vec![parse_stdout_except_head(stdout)?]
    } else {
        let start = split.iter().map(|v| v.0);
        let end = start.clone().skip(1).chain([stdout.len()]);
        start
            .zip(end)
            .map(|(a, b)| {
                let src = &stdout[a..b];
                parse_stdout_except_head(src)
            })
            .collect::<Result<Vec<_>>>()?
    };

    // check the amount of tests
    let parsed_amount_from_head: Vec<_> = split.iter().map(|v| v.2).collect();
    let stats_total: Vec<_> = parsed_stdout.iter().map(|v| v.2.total).collect();
    if parsed_amount_from_head != stats_total {
        return Err(format!(
            "the parsed amount of running tests {parsed_amount_from_head:?} \
             should equal to the number in stats.total {stats_total:?}\n\
             split = {split:#?}\nparsed_stdout = {parsed_stdout:#?}"
        ));
    }

    Ok(split
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
        .collect())
}
