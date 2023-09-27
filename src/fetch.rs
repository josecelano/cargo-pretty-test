use crate::{
    parsing::{parse_cargo_test, Stats},
    prettify::{make_pretty, TestTree, ICON_NOTATION},
    regex::re,
};
use colored::{control::set_override, Colorize};
use console::{set_colors_enabled, set_colors_enabled_stderr, strip_ansi_codes, Key, Term};
use std::{
    io::{self, BufRead, BufReader, Write},
    process::{Child, Command, ExitCode, Stdio},
    thread,
};
use termtree::Tree;

/// Output from `cargo test`
pub struct Emit {
    /// Raw output.
    child: Child,
    /// Don't parse the output. Forward the output instead.
    help: bool,
    /// Don't forward raw output.
    quite: bool,
}

impl Emit {
    pub fn run(self) -> ExitCode {
        let Emit { child, help, quite } = self;
        let [stderr, stdout] = match forward_cargo_test_content(child, quite) {
            Ok(res) => res,
            Err(err) => {
                eprintln!("{err:?}");
                return ExitCode::FAILURE;
            }
        };
        let stderr = strip_ansi_codes(&stderr);
        let stdout = strip_ansi_codes(&stdout);
        if help {
            println!(
                "{phelp}\n{ICON_NOTATION}\n{sep}\n\n{help}",
                phelp = "cargo pretty-test help:".blue().bold(),
                sep = re().separator,
                help = "cargo test help:".blue().bold()
            );
            eprintln!("{stderr}");
            println!("{stdout}");
        } else {
            let (tree, stats) = parse_cargo_test_output(&stderr, &stdout);
            println!("{tree}\n{stats}");
            if !stats.ok {
                return ExitCode::FAILURE;
            }
        }
        ExitCode::SUCCESS
    }
}

fn forward_cargo_test_content(mut child: Child, quite: bool) -> io::Result<[String; 2]> {
    fn forward_and_save<R: BufRead, W: Write>(
        mut reader: R,
        mut writer: Option<W>,
    ) -> io::Result<String> {
        let mut buf = String::with_capacity(1024 * 2);
        let mut pos = 0;
        while let Ok(len) = reader.read_line(&mut buf) {
            if len == 0 {
                return Ok(buf);
            }
            let new = pos + len;
            writer
                .as_mut()
                .map(|w| w.write_all(buf[pos..new].as_bytes()))
                .transpose()?;
            pos = new;
        }
        Ok(buf)
    }

    let term_stdout = (!quite).then(Term::stdout);

    // Forward raw output from `cargo test`, keeping the exact display order.
    let forward_stderr = thread::spawn({
        let child_stderr = BufReader::new(child.stderr.take().unwrap());
        let term_stdout = term_stdout.clone();
        move || forward_and_save(child_stderr, term_stdout).unwrap()
    });
    let forward_stdout = thread::spawn({
        let child_stdout = BufReader::new(child.stdout.take().unwrap());
        let term_stdout = term_stdout.clone();
        move || forward_and_save(child_stdout, term_stdout).unwrap()
    });
    let stderr = forward_stderr.join().unwrap();
    let stdout = forward_stdout.join().unwrap();

    if let Some(term) = term_stdout {
        writeln!(
            &term,
            "{}",
            "Press <Enter> key to clear the screen and show the Test Tree.\n\
             Press any other key to show the Test Tree, keeping the raw input.\n"
                .yellow()
                .bold(),
        )?;
        if matches!(term.read_key(), Ok(Key::Enter)) {
            term.clear_last_lines(1)?;
            term.clear_screen()?;
        } else {
            term.clear_last_lines(3)?;
            writeln!(
                &term,
                "\n{}\n{}\n{}\n",
                "Raw output from `cargo test` 👆".yellow().bold(),
                re().separator,
                "Parsed output from `cargo pretty-test` 👇".yellow().bold()
            )?;
        }
    }

    Ok([stderr, stdout])
}

/// entrypoint for main.rs
pub fn run() -> ExitCode {
    cargo_test().run()
}

/// Collect arguments and forward them to `cargo test`.
///
/// Note: This filters some arguments that mess up the output, like
/// `--nocapture` which prints in the status part and hinders parsing.
pub fn cargo_test() -> Emit {
    let passin: Vec<_> = std::env::args().collect();
    let dashdash = passin.iter().any(|s| s == "--");
    let (mut pre, mut after) = (vec![], vec![]);
    let mut add_arg = |arg: &'static str| {
        if dashdash {
            after.push(arg);
        } else {
            after.extend(["--", arg]);
        }
    };
    match set_color(&passin) {
        SetColor::Unset => {
            pre.push("--color=always");
            add_arg("--color=always");
        }
        SetColor::Always => add_arg("--color=always"),
        SetColor::Never => add_arg("--color=never"),
        SetColor::Auto => add_arg("--color=auto"),
    };

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
    let help = forward.iter().any(|arg| arg == "--help" || arg == "-h");
    let mut quite = false;
    let args = forward.iter().filter(|&arg| {
        arg != "--nocapture" || {
            if arg == "-q" || arg == "--quite" {
                quite = arg == "-q" || arg == "--quite";
                false
            } else {
                true
            }
        }
    });
    Emit {
        child: Command::new("cargo")
            .arg("test")
            .args(pre)
            .args(args)
            .args(after)
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("`cargo test` failed"),
        help,
        quite,
    }
}

enum SetColor {
    Always,
    Never,
    Auto,
    Unset,
}

/// Reintepret `--color` and `CARGO_TERM_COLOR`:
/// * always (default): force `colored` to generate colored text and `console` to enable color
/// * nover: force `colored` not to generate colored text and `console` to disable color
/// * auto: let `colored`/`console` determin to generate/show colored text
///
/// Plus, add explicit `--color` argument to libtest runners, so don't pass `-- --color...` in
/// (didn't check this yet).
fn set_color(forward: &[String]) -> SetColor {
    use SetColor::*;

    let set = if let Some(pos) = forward.iter().position(|arg| arg.starts_with("--color")) {
        match (&*forward[pos], forward.get(pos + 1).map(|s| &**s)) {
            ("--color=always", _) | ("--color", Some("always")) => Always,
            ("--color=never", _) | ("--color", Some("never")) => Never,
            ("--color=auto", _) | ("--color", Some("auto")) => Auto,
            _ => unreachable!("--color only accepts one of always,never,auto"),
        }
    } else if let Some(set_color) = std::env::var_os("CARGO_TERM_COLOR") {
        match set_color.to_str().map(str::to_ascii_lowercase).as_deref() {
            Some("always") => Always,
            Some("never") => Never,
            Some("auto") => Auto,
            _ => unreachable!("--color only accepts one of always,never,auto"),
        }
    } else {
        Unset
    };
    match set {
        Unset | Always => {
            set_override(true);
            set_colors_enabled(true);
            set_colors_enabled_stderr(true);
        }
        Never => {
            set_override(false);
            set_colors_enabled(false);
            set_colors_enabled_stderr(false);
        }
        Auto => (),
    }
    set
}

pub fn parse_cargo_test_output<'s>(stderr: &'s str, stdout: &'s str) -> (TestTree<'s>, Stats) {
    let mut tree = Tree::new("Generated by cargo-pretty-test".bold().to_string().into());
    let mut stats = Stats::default();
    for (pkg, data) in parse_cargo_test(stderr, stdout).pkgs {
        stats += &data.stats;
        let root = data.stats.root_string(pkg.unwrap_or("tests")).into();
        tree.push(
            Tree::new(root).with_leaves(data.inner.into_iter().filter_map(|data| {
                let parsed = data.info.parsed;
                let detail_without_stats = parsed.detail;
                if !detail_without_stats.is_empty() {
                    eprintln!("{detail_without_stats}\n\n{}\n", re().separator);
                }
                let root = data.info.stats.subroot_string(data.runner.src.src_path);
                make_pretty(root, parsed.tree.into_iter())
            })),
        );
    }
    (tree, stats)
}
