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

// Parsers
pub mod parser;

mod source;
pub use source::Source;
