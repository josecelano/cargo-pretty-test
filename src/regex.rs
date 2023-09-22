use regex_lite::Regex;
use std::sync::OnceLock;

fn re_lines_of_tests() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?m)^test \S+ \.\.\. \S+$").expect("regex pattern error"))
}

/// Get the lines of tests from `cargo test`.
pub fn get_lines_of_tests(text: &str) -> impl Iterator<Item = &str> {
    re_lines_of_tests().find_iter(text).map(|m| m.as_str())
}
