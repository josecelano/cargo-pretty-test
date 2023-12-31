//! This crate can be used as a binary or a library.
//!
//! * library: [parse the output from `cargo test`][crate::parsing]
//! * binary: `cargo install cargo-pretty-test`
//!
//! ```text
//! $ cargo pretty-test --workspace --no-fail-fast
//!
//! Error details from `cargo test` if any ... (Omitted here)
//!
//! Generated by cargo-pretty-test
//! ├── (OK) cargo_pretty_test ... (4 tests in 0.16s: ✅ 4)
//! │   ├── (OK) tests/golden_master_test.rs ... (1 tests in 0.00s: ✅ 1)
//! │   │   └─ ✅ golden_master_test
//! │   ├── (OK) tests/mocking_project.rs ... (2 tests in 0.16s: ✅ 2)
//! │   │   ├─ ✅ snapshot_testing_for_parsed_output
//! │   │   └─ ✅ snapshot_testing_for_pretty_output
//! │   └── (OK) tests/parsing.rs ... (1 tests in 0.00s: ✅ 1)
//! │       └─ ✅ parse_stderr_stdout
//! ├── (FAIL) integration ... (10 tests in 0.00s: ✅ 6; ❌ 2; 🔕 2)
//! │   ├── (FAIL) src/lib.rs ... (8 tests in 0.00s: ✅ 4; ❌ 2; 🔕 2)
//! │   │   ├── submod
//! │   │   │   ├─ 🔕 ignore
//! │   │   │   ├─ 🔕 ignore_without_reason
//! │   │   │   ├─ ✅ normal_test
//! │   │   │   └── panic
//! │   │   │       ├─ ❌ panicked
//! │   │   │       ├─ ✅ should_panic - should panic
//! │   │   │       ├─ ❌ should_panic_but_didnt - should panic
//! │   │   │       └─ ✅ should_panic_without_reanson - should panic
//! │   │   └─ ✅ works
//! │   ├── (OK) src/main.rs ... (1 tests in 0.00s: ✅ 1)
//! │   │   └─ ✅ from_main_rs
//! │   └── (OK) tests/parsing.rs ... (1 tests in 0.00s: ✅ 1)
//! │       └─ ✅ from_integration
//! └── (OK) Doc Tests ... (2 tests in 0.41s: ✅ 2)
//!     ├── (OK) cargo-pretty-test ... (1 tests in 0.20s: ✅ 1)
//!     │   └─ ✅ src/doc.rs - doc (line 3)
//!     └── (OK) integration ... (1 tests in 0.21s: ✅ 1)
//!         └─ ✅ tests/integration/src/lib.rs - doc (line 41)
//!
//! Status: FAIL; total 16 tests in 0.57s: 12 passed; 2 failed; 2 ignored; 0 measured; 0 filtered out
//! ```
//!
//! ![](https://user-images.githubusercontent.com/25300418/270264132-89de6fd2-11f8-4e5b-b9dc-8475fa022a5f.png)
//!
//! [More screenshots.](https://github.com/josecelano/cargo-pretty-test/wiki/cargo%E2%80%90pretty%E2%80%90test-screenshots)

#![allow(
    clippy::missing_panics_doc,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::enum_glob_use
)]

#[doc(hidden)]
pub mod doc;

pub mod fetch;
pub mod parsing;
pub mod prettify;
pub mod regex;

pub type Error = String;
pub type Result<T, E = Error> = ::std::result::Result<T, E>;
