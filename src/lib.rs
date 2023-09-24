#![allow(
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::enum_glob_use
)]

#[doc(hidden)]
pub mod doc;

pub mod fetch;
pub mod parsing;
pub mod prettify;
pub mod regex;
