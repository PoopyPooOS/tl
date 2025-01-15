use crate::{
    parser::tokenizer::types::{Token, TokenType},
    Source,
};
use logger::Location;
use types::{Error, Expr, Statement, StatementType};

pub mod types;

// AST Implementations
mod array;
mod binary_op;
mod expr;
mod r#fn;
mod ident;
mod interpolated_string;
mod r#let;
mod object;

#[derive(Debug)]
pub struct Parser {
    // Input
    tokens: Vec<Token>,
    source: Source,

    // State
    position: usize,
    context: Context,
}

#[derive(Debug, PartialEq, Clone)]
enum Context {
    TopLevel,
    Function,
    Object,
    DotAccessPath,
    CallArgs,
}

pub type ExprResult = Result<Expr, Box<Error>>;
pub type StatementResult = Result<Vec<Statement>, Box<Error>>;

impl Parser {
    pub fn new(tokens: Vec<Token>, source: impl Into<Source>) -> Self {
        if cfg!(debug_assertions) {
            logger::set_app_name!("AST");
        }

        Self {
            tokens,
            source: source.into(),

            position: 0,
            context: Context::TopLevel,
        }
    }

    /// Generates an AST based on the tokens of this [`Parser`].
    /// # Errors
    /// This function will return an error if a AST generation error occurs.
    pub fn parse(&mut self) -> Result<Vec<Statement>, Box<Error>> {
        let mut statements = Vec::new();

        while let Some(token) = self.tokens.get(self.position) {
            let parsed = match token.token_type {
                TokenType::Let => self.parse_let()?,
                TokenType::RBrace if self.context == Context::Function => break,
                _ => {
                    let expr = self.parse_expr()?;
                    let (line, cols) = (expr.line, expr.cols.clone());
                    vec![Statement::new(StatementType::Expr(expr), line, cols)]
                }
            };

            statements.extend(parsed);
        }

        Ok(statements)
    }

    #[allow(
        clippy::unnecessary_wraps,
        reason = "`Option<Location>` is more commonly used, this simplifies things"
    )]
    /// Always returns `Some`, safe to unwrap if needed.
    fn location_from_token(&self, token: &Token) -> Option<Location> {
        Some(Location {
            path: self.source.path.clone(),
            text: self.source.text.clone(),
            lines: token.line..=token.line,
            section: Some(token.cols.clone()),
        })
    }
}

/// Internal macro for the AST.
macro_rules! advance {
    ($self:expr) => {{
        let token = $self.tokens.get($self.position);
        if token.is_some() {
            $self.position = $self.position.saturating_add(1);
        }
        token
    }};
}

/// Internal macro for the AST.
macro_rules! consume {
    ($self:expr, $expected:ident) => {
        match $crate::parser::ast::advance!($self) {
            Some(token) => {
                if token.token_type == $crate::parser::tokenizer::types::TokenType::$expected {
                    Ok(token.clone())
                } else {
                    $crate::parser::ast::err!(
                        ExpectedTokenGot($crate::parser::tokenizer::types::TokenType::$expected, token.token_type.clone()),
                        $self.location_from_token(token)
                    )
                }
            }
            _ => $crate::parser::ast::err!(ExpectedToken($crate::parser::tokenizer::types::TokenType::$expected)),
        }?
    };
    ($self:expr, $expected:ident($($value:expr),*)) => {
        match $crate::parser::ast::advance!($self) {
            Some(token) => {
                if let $crate::parser::tokenizer::types::TokenType::$expected($($value),*) = token.token_type {
                    Ok(token.clone())
                } else {
                    $crate::parser::ast::err!(
                        ExpectedTokenGot(
                            $crate::parser::tokenizer::types::TokenType::$expected($($value.clone()),*),
                            token.token_type.clone()
                        ),
                        $self.location_from_token(token)
                    )
                }
            }
            _ => $crate::parser::ast::err!(
                ExpectedToken($crate::parser::tokenizer::types::TokenType::$expected($($value.clone()),*))
            ),
        }
    };
}

/// Internal macro for the AST.
#[allow(unused_macros, reason = "will be used soon probably")]
macro_rules! get_ident {
    ($self:expr) => {
        match advance!($self) {
            Some(token) => {
                if let TokenType::Identifier(v) = &token.token_type {
                    v.clone()
                } else {
                    return $crate::parser::ast::err!(
                        ExpectedToken(TokenType::Identifier("identifier".to_string())),
                        $self.location_from_token(
                            $self
                                .tokens
                                .get($self.position)
                                .ok_or_else(|| $crate::parser::ast::raw_err!(NoTokensLeft))?
                        )
                    );
                }
            }
            _ => return $crate::parser::ast::err!(
                ExpectedToken(TokenType::Identifier("identifier".to_string())),
                $self.location_from_token(
                    $self
                        .tokens
                        .get($self.position.saturating_sub(1))
                        .ok_or_else(|| $crate::parser::ast::raw_err!(NoTokensLeft))?
                )
            ),
        }
    };
    ($self:expr, $token:expr) => {
        let name = match advance!($self) {
            Some(token) => {
                if let TokenType::Identifier(name) = &token.token_type {
                    (token.clone(), name.clone())
                } else {
                    return $crate::parser::ast::err!(
                        ExpectedToken(TokenType::Identifier("identifier".to_string())),
                        $self.location_from_token(
                            $self
                                .tokens
                                .get($self.position)
                                .ok_or_else(|| $crate::parser::ast::raw_err!(NoTokensLeft))?
                        )
                    );
                }
            }
            _ => return $crate::parser::ast::err!(
                ExpectedToken(TokenType::Identifier("identifier".to_string())),
                $self.location_from_token(
                    $self
                        .tokens
                        .get($self.position)
                        .ok_or_else(|| $crate::parser::ast::raw_err!(NoTokensLeft))?
                )
            );
        };
    };
}

/// Internal macro for the AST.
macro_rules! err {
    ($err_type:ident) => {
        Err(Box::new(crate::parser::ast::types::Error::new(crate::parser::ast::types::ErrorType::$err_type, None)))
    };
    ($err_type:ident, $location:expr) => {
        Err(Box::new(crate::parser::ast::types::Error::new(crate::parser::ast::types::ErrorType::$err_type, $location)))
    };
    ($err_type:ident($($err_value:expr),*)) => {
        Err(Box::new(crate::parser::ast::types::Error::new(crate::parser::ast::types::ErrorType::$err_type($($err_value),*), None)))
    };
    ($err_type:ident($($err_value:expr),*), $location:expr) => {
        Err(Box::new(crate::parser::ast::types::Error::new(crate::parser::ast::types::ErrorType::$err_type($($err_value),*), $location)))
    };
}

/// Internal macro for the AST.
macro_rules! raw_err {
    ($err_type:ident) => {
        Box::new(crate::parser::ast::types::Error::new(crate::parser::ast::types::ErrorType::$err_type, None))
    };
    ($err_type:ident, $location:expr) => {
        Box::new(crate::parser::ast::types::Error::new(crate::parser::ast::types::ErrorType::$err_type, $location))
    };
    ($err_type:ident($($err_value:expr),*)) => {
        Box::new(crate::parser::ast::types::Error::new(crate::parser::ast::types::ErrorType::$err_type($($err_value),*), None))
    };
    ($err_type:ident($($err_value:expr),*), $location:expr) => {
        Box::new(crate::parser::ast::types::Error::new(crate::parser::ast::types::ErrorType::$err_type($($err_value),*), $location))
    };
}

#[allow(unused_imports, reason = "will be used soon probably")]
pub(crate) use {advance, consume, err, get_ident, raw_err};
