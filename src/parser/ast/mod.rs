use crate::{
    Source,
    parser::{
        ast::types::{error::Error, expr::Expr},
        lexer::types::token::Token,
    },
};
use miette::SourceSpan;
use tl_macro::peek;

pub mod types;

// AST Implementations
mod array;
mod binary_op;
mod expr;
mod r#fn;
mod ident;
mod interpolated_path;
mod interpolated_string;
mod object;
mod r#type;
mod with;

mod pretty_print;

#[derive(Debug)]
pub struct Parser {
    // Input
    tokens: Vec<Token>,
    source: Source,

    // State
    pos: usize,
}

pub type ExprResult = Result<Expr, Error>;

impl Parser {
    pub fn new(tokens: Vec<Token>, source: Source) -> Self {
        Self {
            tokens,
            source,

            pos: 0,
        }
    }

    /// Return a span that contains the current line the parser is on.
    fn closest_span(&self) -> SourceSpan {
        match () {
            _ if let Some(token) = peek!(0) => token.span,
            _ if let Some(last_token) = peek!(-1) => last_token.span,
            _ => {
                let length = self
                    .source
                    .inner()
                    .as_bytes()
                    .iter()
                    .position(|&b| b == b'\n')
                    .unwrap_or(self.source.inner().len());

                SourceSpan::new(0.into(), length)
            }
        }
    }
}
