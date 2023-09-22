use regex_lite::Regex;

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

lazy_static! {
    re_header, Regex, {
        Regex::new(r"running \d+ tests").expect(RE_ERROR)
    };
    re_lines_of_tests, Regex, {
        Regex::new(r"(?m)^test \S+ \.\.\. \S+$").expect(RE_ERROR)
    };
}

pub struct ParsedCargoTestOutput<'s> {
    pub head: &'s str,
    pub tree: Vec<&'s str>,
    pub detail: &'s str,
}

pub fn parse_cargo_test_output(text: &str) -> ParsedCargoTestOutput<'_> {
    let head = re_header()
        .find(text)
        .expect("`running \\d+ tests` not found");
    let head_end = head.end() + 1;
    let line: Vec<_> = re_lines_of_tests().find_iter(&text[head_end..]).collect();
    let tree_end = line.last().map_or(head_end, |cap| head_end + cap.end() + 1);
    let mut tree: Vec<_> = line.into_iter().map(|cap| cap.as_str()).collect();
    tree.sort_unstable();
    ParsedCargoTestOutput {
        head: head.as_str(),
        tree,
        detail: text[tree_end..].trim(),
    }
}

/// Get the lines of tests from `cargo test`.
pub fn get_lines_of_tests(text: &str) -> impl Iterator<Item = &str> {
    re_lines_of_tests().find_iter(text).map(|m| m.as_str())
}
