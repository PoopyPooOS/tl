#![feature(stmt_expr_attributes)]
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
#![allow(clippy::result_large_err)]

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

pub use indexmap;
