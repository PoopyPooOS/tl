#![feature(let_chains, stmt_expr_attributes)]
// Lints
#![allow(clippy::unused_self, clippy::too_many_lines)]
#![deny(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    arithmetic_overflow,
    clippy::float_arithmetic,
    clippy::arithmetic_side_effects
)]
#![warn(clippy::unimplemented, clippy::todo)]

// Tests
#[cfg(test)]
mod tests;

mod error;
mod source;
pub use error::Error;
pub use source::Source;

// Parsers
pub mod parser;

// Runtime
pub mod runtime;

// Utils
pub mod utils;
