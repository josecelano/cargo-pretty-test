use colored::{ColoredString, Colorize};
use regex_lite::Regex;

// Lazily initialize a global variable.
#[doc(hidden)]
#[macro_export]
macro_rules! lazy_static {
    ($v:vis $f:ident, $t:ty, $e:block $(;)?) => {
        lazy_static! { $v $f -> &'static $t, $t, $e }
    };
    ($v:vis $f:ident -> $ret:ty, $t:ty, $e:block $(;)?) => {
        #[allow(dead_code)]
        $v fn $f() -> $ret {
            static TMP: ::std::sync::OnceLock<$t> = ::std::sync::OnceLock::new();
            TMP.get_or_init(|| $e)
        }
    };
}

const RE_ERROR: &str = "regex pattern error";

pub struct Re {
    pub ty: Regex,
    pub head: Regex,
    pub tree: Regex,
    pub stats: Regex,
    pub separator: ColoredString,
}

lazy_static!(pub re, Re, {
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
        //
        // Doc Test: ^test (?P<file>\S+) - (?P<item>\S+) \(line \d+\)( - compile( fail)?)? ... (?P<status>\S+(, .*)?)$
        // test src/doc.rs - doc (line 3) ... ok
        // test tests/integration/src/lib.rs - attribute::edition2018 (line 100) ... ok
        // test tests/integration/src/lib.rs - attribute::ignore (line 76) ... ignored
        // test tests/integration/src/lib.rs - attribute::no_run (line 86) - compile ... ok
        // test tests/integration/src/lib.rs - attribute::should_compile_fail (line 90) - compile fail ... ok
        // test tests/integration/src/lib.rs - attribute::should_compile_fail_but_didnt (line 96) - compile fail ... FAILED
        // test tests/integration/src/lib.rs - attribute::should_panic (line 80) ... ok
        // test tests/integration/src/lib.rs - empty_doc_mod (line 41) ... ok
        // test tests/integration/src/lib.rs - empty_doc_mod::Item (line 48) ... ok
        // test tests/integration/src/lib.rs - empty_doc_mod::private_mod (line 44) ... ok
        tree: Regex::new(r"(?m)^test (?P<split>\S+( - should panic)?(?<doctest> - \S+ \(line \d+\)( - compile( fail)?)?)?) \.\.\. (?P<status>\S+(, .*)?)$").expect(RE_ERROR),
        // test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
        stats: Regex::new(r"(?mx)
            ^test\ result:\ (?P<ok>\S+)\.
            \ (?P<passed>\d+)\ passed;
            \ (?P<failed>\d+)\ failed;
            \ (?P<ignored>\d+)\ ignored;
            \ (?P<measured>\d+)\ measured;
            \ (?P<filtered>\d+)\ filtered\ out;
            \ finished\ in\ (?P<time>\S+)s$").expect(RE_ERROR),
        separator: "────────────────────────────────────────────────────────────────────────".yellow().bold()
    }
});
