#![feature(let_chains, stmt_expr_attributes)]
#![allow(clippy::unused_self)]
// Panics are not allowed
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

// Tests
#[cfg(test)]
mod tests;

// Parsers
pub mod parser;

mod source;
pub use source::Source;
