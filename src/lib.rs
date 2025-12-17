#![feature(if_let_guard, stmt_expr_attributes, iterator_try_collect)]
// Lints
#![deny(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    arithmetic_overflow,
    clippy::float_arithmetic,
    clippy::arithmetic_side_effects
)]
#![warn(clippy::unimplemented, clippy::todo, clippy::str_to_string)]
#![allow(clippy::result_large_err, clippy::ignored_unit_patterns)]

// Tests
#[cfg(test)]
mod tests;

mod error;
pub use error::Error;

// Parsers
pub mod parser;

// Runtime
pub mod runtime;

// Utils
mod utils;
pub use utils::*;

mod source;
pub use source::Source;

// Re-exports for macros to be able to work outside of this crate
pub use indexmap;
