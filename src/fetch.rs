use crate::{
    prettify::make_pretty,
    regex::{TestInfo, Text},
};
use std::process::{Command, Output};
use termtree::Tree;

/// Collect arguments and forward them to `cargo test`.
///
/// Note: This filters some arguments that mess up the output, like
/// `--nocapture` which prints in the status part and hinders parsing.
pub fn cargo_test() -> Output {
    let passin: Vec<_> = std::env::args().collect();
    let forward = if passin
        .get(..2)
        .is_some_and(|v| v[0].ends_with("cargo-pretty-test") && v[1] == "pretty-test")
    {
        // `cargo pretty-test` yields ["path-to-cargo-pretty-test", "pretty-test", rest]
        &passin[2..]
    } else {
        // `cargo-pretty-test` yields ["path-to-cargo-pretty-test", rest]
        &passin[1..]
    };
    let args = forward.iter().filter(|arg| *arg != "--nocapture");
    Command::new("cargo")
        .arg("test")
        .args(args)
        .output()
        .expect("`cargo test` failed")
}

pub fn parse_cargo_test_output<'s>(stderr: &'s str, stdout: &'s str) -> Tree<Text<'s>> {
    let mut tree = Tree::new("Generated by cargo-pretty-test");
    for (pkg, data) in TestInfo::parse_cargo_test(stderr, stdout).pkgs {
        tree.push(Tree::new(pkg.unwrap_or("tests")).with_leaves(
            data.inner.into_iter().filter_map(|data| {
                let parsed = data.info.parsed;
                let detail_without_stats = parsed.detail;
                if !detail_without_stats.is_empty() {
                    eprintln!(
                        "{detail_without_stats}\n\n\
                         *************************************************************\n"
                    );
                }
                make_pretty(data.runner.src.src_path, parsed.tree.into_iter())
            }),
        ));
    }
    tree
}
