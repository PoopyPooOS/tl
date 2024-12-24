#![warn(clippy::todo)]

use crate::{
    parser::tokenizer::types::{Token, TokenType},
    Source,
};
use logger::Location;
use std::collections::BTreeMap;
use types::{BinaryOperator, Error, ErrorType, Expr, ExprType, Literal, Statement, StatementType};

pub mod types;

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
#[allow(dead_code)]
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
            #[allow(clippy::match_single_binding, clippy::single_match_else, reason = "TODO: Add more statements")]
            let parsed = match token.token_type {
                TokenType::Let => self.parse_let()?,
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

    fn parse_expr(&mut self) -> ExprResult {
        if let Some(next_token) = self.tokens.get(self.position + 1) {
            if next_token.token_type.is_binary_operator() {
                return self.parse_binary_op(0);
            }
        }

        let expr = match &self.tokens[self.position].token_type {
            TokenType::LBrace => Some(self.parse_object()?),
            TokenType::LBracket => Some(self.parse_array()?),
            TokenType::Identifier(_) => Some(self.parse_ident()?),
            TokenType::LParen => {
                if let Some(next_token) = self.tokens.get(self.position + 1) {
                    if next_token.token_type.is_number() {
                        return self.parse_binary_op(0);
                    }
                }
                None
            }
            TokenType::Not => {
                let token = self.tokens[self.position].clone();
                self.consume(TokenType::Not)?;
                let expr = self.parse_expr()?;
                let end = *expr.cols.end();
                Some(Expr::new(ExprType::Not(Box::new(expr)), token.line, *token.cols.start()..=end))
            }
            _ => None,
        };

        if let Some(expr) = expr {
            return Ok(expr);
        }

        self.parse_literal()
    }

    fn parse_literal(&mut self) -> ExprResult {
        let token = self.tokens[self.position].clone();

        let expr = match &token.token_type {
            TokenType::Null => Expr::new(ExprType::Literal(Literal::Null), token.line, token.cols),
            TokenType::String(v) => Expr::new(ExprType::Literal(Literal::String(v.clone())), token.line, token.cols),
            TokenType::InterpolatedString(v) => self.parse_interpolated_string(v)?,
            TokenType::Int(v) => Expr::new(ExprType::Literal(Literal::Int(*v)), token.line, token.cols),
            TokenType::Float(v) => Expr::new(ExprType::Literal(Literal::Float(*v)), token.line, token.cols),
            TokenType::Bool(v) => Expr::new(ExprType::Literal(Literal::Bool(*v)), token.line, token.cols),
            TokenType::Identifier(v) => Expr::new(ExprType::Identifier(v.clone()), token.line, token.cols),
            other => {
                let location = Location {
                    path: self.source.path.clone(),
                    text: self.source.text.clone(),
                    lines: token.line..=token.line,
                    section: Some(token.cols),
                };

                return Err(Box::new(Error::new(ErrorType::UnexpectedToken(other.clone()), Some(location))));
            }
        };

        self.position += 1;

        Ok(expr)
    }

    fn parse_interpolated_string(&mut self, v: &[Token]) -> ExprResult {
        let mut result: Vec<Expr> = Vec::new();
        let start = self.tokens[self.position].clone();

        for token in v {
            match &token.token_type {
                TokenType::String(v) => {
                    result.push(Expr::new(
                        ExprType::Literal(Literal::String(v.clone())),
                        token.line,
                        token.cols.clone(),
                    ));
                }
                TokenType::InterpolatedString(v) => {
                    let ast = Self::new(v.clone(), self.source.clone()).parse()?;
                    if let Some(first) = ast.first() {
                        let StatementType::Expr(first) = &first.statement_type else {
                            unreachable!()
                        };

                        result.push(first.clone());
                    }
                }
                _ => {
                    // FIXME: Cloning `self.source` is very inefficient.
                    let ast = Self::new(vec![token.clone()], self.source.clone()).parse()?;
                    if let Some(first) = ast.first() {
                        let StatementType::Expr(first) = &first.statement_type else {
                            unreachable!()
                        };

                        result.push(first.clone());
                    }
                }
            }
        }

        Ok(Expr::new(
            ExprType::Literal(Literal::InterpolatedString(result)),
            start.line,
            start.cols.clone(),
        ))
    }

    fn parse_object(&mut self) -> ExprResult {
        let start = self.tokens[self.position].clone();
        self.consume(TokenType::LBrace)?;
        let last_context = self.context.clone();
        self.context = Context::Object;

        let mut fields = BTreeMap::new();

        loop {
            if self.tokens[self.position].token_type == TokenType::RBrace {
                self.consume(TokenType::RBrace)?;
                break;
            }

            let next_token = {
                let token = self.tokens.get(self.position);
                if token.is_some() {
                    self.position += 1;
                }
                token
            };

            let key = match next_token {
                Some(token) => {
                    if let TokenType::Identifier(name) = &token.token_type {
                        name.clone()
                    } else {
                        return Err(Box::new(Error::new(
                            ErrorType::ExpectedTokenGot(TokenType::Identifier(String::new()), token.token_type.clone()),
                            self.location_from_token(token),
                        )));
                    }
                }
                _ => {
                    return Err(Box::new(Error::new(
                        ErrorType::ExpectedToken(TokenType::Identifier(String::new())),
                        self.location_from_token(&self.tokens[self.position]),
                    )))
                }
            };

            let next_token = {
                let token = self.tokens.get(self.position);
                if token.is_some() {
                    self.position += 1;
                }
                token
            };

            match next_token {
                Some(token) => if matches!(token.token_type, TokenType::Colon | TokenType::Equals) {},
                None =>
                {
                    #[allow(clippy::unwrap_used, reason = "`location_from_token` always returns `Some`")]
                    return Err(Box::new(Error::new(
                        ErrorType::ExpectedSeperatorInObjectKV,
                        self.tokens.get(self.position).map(|token| self.location_from_token(token).unwrap()),
                    )))
                }
            }

            let value = self.parse_expr()?;

            fields.insert(key, value);
        }

        self.context = last_context;
        let end = &self.tokens[self.position - 1].cols.end();

        Ok(Expr::new(
            ExprType::Literal(Literal::Object(fields)),
            start.line,
            *start.cols.start()..=**end,
        ))
    }

    fn parse_binary_op(&mut self, min_precedence: u8) -> ExprResult {
        let start = self.tokens[self.position].clone();
        let mut left = if self.tokens[self.position].token_type == TokenType::LParen {
            self.consume(TokenType::LParen)?;
            let expr = self.parse_binary_op(0)?;
            self.consume(TokenType::RParen)?;
            expr
        } else {
            self.parse_literal()?
        };

        while let Some(next_token) = self.tokens.get(self.position) {
            if !next_token.token_type.is_binary_operator() {
                break;
            }

            let precedence = BinaryOperator::from_token(next_token.token_type.clone())?.precedence();
            if precedence < min_precedence {
                break;
            }

            let operator_token = self.tokens[self.position].clone();
            self.position += 1;
            let operator = BinaryOperator::from_token(operator_token.token_type)?;

            let right = self.parse_binary_op(precedence + 1)?;

            let line = left.line;
            let end = *right.cols.end();
            left = Expr::new(
                ExprType::BinaryOp {
                    left: Box::new(left),
                    operator,
                    right: Box::new(right),
                },
                line,
                *start.cols.start()..=end,
            );
        }

        Ok(left)
    }

    fn parse_array(&mut self) -> ExprResult {
        let start = self.tokens[self.position].clone();
        self.consume(TokenType::LBracket)?;

        let mut array = Vec::new();
        while let Some(next_token) = self.tokens.get(self.position).cloned() {
            if next_token.token_type == TokenType::RBracket {
                self.consume(TokenType::RBracket)?;
                break;
            }

            let expr = self.parse_expr()?;
            array.push(expr);
        }

        let end = &self.tokens[self.position - 1].cols.end();
        Ok(Expr::new(
            ExprType::Literal(Literal::Array(array)),
            start.line,
            *start.cols.start()..=**end,
        ))
    }

    fn parse_ident(&mut self) -> ExprResult {
        let mut is_call = false;
        let mut is_access = false;

        if let Some(next_token) = self.tokens.get(self.position + 1) {
            if next_token.token_type.is_binary_operator() {
                return self.parse_binary_op(0);
            }

            is_call = next_token.token_type == TokenType::LParen;
            is_access = next_token.token_type == TokenType::Dot;
        }

        let next_token = {
            let token = self.tokens.get(self.position);
            if token.is_some() {
                self.position += 1;
            }
            token
        };

        let name = match next_token {
            Some(token) => {
                if let TokenType::Identifier(name) = &token.token_type {
                    (token, name.clone())
                } else {
                    unreachable!();
                }
            }
            _ => unreachable!(),
        };

        if !is_call && !is_access {
            return Ok(Expr::new(ExprType::Identifier(name.1), name.0.line, name.0.cols.clone()));
        }

        if is_call {
            let name = (name.0.clone(), name.1);
            let mut call_args = Vec::new();
            self.consume(TokenType::LParen)?;

            // Empty args
            if let Some(token) = self.tokens.get(self.position) {
                if token.token_type == TokenType::RParen {
                    self.position += 1;
                    let start = *name.0.cols.start();

                    return Ok(Expr::new(
                        ExprType::Call {
                            name: name.1,
                            args: call_args,
                        },
                        name.0.line,
                        start..=*token.cols.end(),
                    ));
                }
            }

            // 1 argument with no commas
            let token = self.parse_expr()?;
            call_args.push(token);

            // Handle multiple args
            while let Some(next_token) = self.tokens.get(self.position) {
                if next_token.token_type == TokenType::RParen {
                    break;
                }

                self.consume(TokenType::Comma)?;

                let token = self.parse_expr()?;
                call_args.push(token);
            }

            self.consume(TokenType::RParen)?;

            let start = *name.0.cols.start();
            let end = &self.tokens[self.position - 1].cols.end();

            return Ok(Expr::new(
                ExprType::Call {
                    name: name.1,
                    args: call_args,
                },
                name.0.line,
                start..=**end,
            ));
        }

        // Clone otherwise it would require 2 mutable borrows.
        let name_token = name.0.clone();
        let mut paths: Vec<String> = vec![name.1];

        let next_token = {
            let token = self.tokens.get(self.position);
            if token.is_some() {
                self.position += 1;
            }
            token
        };

        while let Some(token) = next_token {
            if let TokenType::Identifier(ident) = &token.token_type {
                paths.push(ident.clone());
            }
        }

        let line = name_token.line;
        let token = &self.tokens[self.position];

        Ok(Expr::new(
            ExprType::DotAccess(paths),
            line,
            *name_token.cols.start()..=*token.cols.end(),
        ))
    }

    fn parse_let(&mut self) -> StatementResult {
        let start = self.tokens[self.position].clone();
        self.consume(TokenType::Let)?;

        let next_token = {
            let token = self.tokens.get(self.position);
            if token.is_some() {
                self.position += 1;
            }
            token
        };

        let name = match next_token {
            Some(token) => {
                if let TokenType::Identifier(name) = &token.token_type {
                    (token.clone(), name.clone())
                } else {
                    return Err(Box::new(Error::new(
                        ErrorType::NoIdentifierAfterLet,
                        self.location_from_token(token),
                    )));
                }
            }
            _ => {
                return Err(Box::new(Error::new(
                    ErrorType::NoIdentifierAfterLet,
                    self.location_from_token(&self.tokens[self.position - 1]),
                )))
            }
        };

        self.consume(TokenType::Equals)?;
        let value = self.parse_expr()?;

        let line = value.line;
        let end = *value.cols.end();
        Ok(vec![Statement::new(
            StatementType::Let { name: name.1, value },
            line,
            *start.cols.start()..=end,
        )])
    }

    fn consume(&mut self, expected: TokenType) -> Result<(), Box<Error>> {
        let next_token = {
            let token = self.tokens.get(self.position);
            if token.is_some() {
                self.position += 1;
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
