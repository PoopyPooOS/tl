use crate::{
    parser::tokenizer::types::{Token, TokenType},
    Source,
};
use logger::Location;
use types::{Error, ErrorType, Expr, Statement, StatementType};

pub mod types;

// AST Implementations
mod array;
mod binary_op;
mod expr;
mod fn_decl;
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

    fn consume(&mut self, expected: TokenType) -> Result<(), Box<Error>> {
        let next_token = {
            let token = self.tokens.get(self.position);
            if token.is_some() {
                self.position = self.position.saturating_add(1);
            }
            token
        };

        match next_token {
            Some(token) => {
                if token.token_type == expected {
                    Ok(())
                } else {
                    Err(Box::new(Error::new(
                        ErrorType::ExpectedTokenGot(expected, token.token_type.clone()),
                        self.location_from_token(token),
                    )))
                }
            }
            _ => Err(Box::new(Error::new(ErrorType::ExpectedToken(expected), None))),
        }
    }

    fn advance(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.position);
        if token.is_some() {
            self.position = self.position.saturating_add(1);
        }
        token
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
