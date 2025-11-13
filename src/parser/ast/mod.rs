use crate::parser::lexer::types::Token;
use miette::{NamedSource, SourceSpan};
use types::{Error, Expr};

pub mod types;

// AST Implementations
mod array;
mod binary_op;
mod expr;
mod r#fn;
mod ident;
mod interpolated_path;
mod interpolated_string;
mod r#let;
mod object;

mod pretty_print;

#[derive(Debug)]
pub struct Parser {
    // Input
    tokens: Vec<Token>,
    source: NamedSource<String>,

    // State
    pos: usize,
    context: Context,
}

#[derive(Debug, PartialEq, Clone)]
enum Context {
    TopLevel,
    Function,
    Object,
}

pub type ExprResult = Result<Expr, Error>;

impl Parser {
    pub fn new(tokens: Vec<Token>, source: NamedSource<String>) -> Self {
        Self {
            tokens,
            source,

            pos: 0,
            context: Context::TopLevel,
        }
    }

    /// Return a span that contains the current line the parser is on.
    fn closest_span(&self) -> SourceSpan {
        if let Some(token) = self.tokens.get(self.pos) {
            token.span
        } else if let Some(token) = self.tokens.get(self.pos.saturating_sub(1)) {
            token.span
        } else {
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

/// Internal macro for the AST.
macro_rules! advance {
    ($self:expr) => {{
        let token = $self.tokens.get($self.pos);
        if token.is_some() {
            $self.pos = $self.pos.saturating_add(1);
        }
        token
    }};
}

/// Internal macro for the AST.
macro_rules! consume {
    ($self:expr, $expected:ident) => {
        $crate::parser::ast::consume!(no_propagate $self, $expected)?
    };
    ($self:expr, $expected:ident($($value:expr),*)) => {
        $crate::parser::ast::consume!(no_propagate $self, $expected($($value),*))?
    };
    (no_propagate $self:expr, $expected:ident) => {{
        use $crate::parser::lexer::types::TokenKind;
        use $crate::parser::ast::types::{Error, ErrorKind};

        match $crate::parser::ast::advance!($self) {
            Some(token) => {
                if token.kind == TokenKind::$expected {
                    Ok(token.clone())
                } else {
                    Err(Error::new(
                        ErrorKind::ExpectedToken {
                            expected: stringify!($expected).to_lowercase(),
                            found: Some(token.kind.clone())
                        },
                        $self.source.clone(),
                        token.span,
                    ))
                }
            }
            _ => Err(Error::new(
                ErrorKind::ExpectedToken {
                    expected: stringify!($expected).to_lowercase(),
                    found: None
                },
                $self.source.clone(),
                $self.closest_span(),
            ))
        }
    }};
    (no_propagate $self:expr, $expected:ident($($value:expr),*)) => {{
        use $crate::parser::lexer::types::TokenKind;
        use $crate::parser::ast::types::{Error, ErrorKind};

        match $crate::parser::ast::advance!($self) {
            Some(token) => {
                if let TokenKind::$expected($($value),*) = token.kind {
                    Ok(token.clone())
                } else {
                    Err(Error::new(
                        ErrorKind::ExpectedToken(TokenKind::$expected($($value.clone()),*), Some(token.kind.clone())),
                        $self.source.clone(),
                        token.span,
                    ))
                }
            }
            _ => Err(Error::new(
                ErrorKind::ExpectedToken(TokenKind::$expected($($value.clone()),*), None),
                $self.source.clone(),
                $self.closest_span(),
            ))
        }
    }};
}

pub(crate) use {advance, consume};
