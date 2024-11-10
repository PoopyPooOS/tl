#![allow(clippy::unused_self)]

use super::types::{BinaryOperator, Expr, Literal, Statement};
use crate::tokenizer::types::{Token, TokenType};
use logger::{make_error, Location, Log};
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct ParseResult {
    pub statements: Vec<Statement>,
    pub advance_by: usize,
}

#[derive(Debug, PartialEq)]
pub struct Parser {
    tokens: Vec<Token>,
    path: PathBuf,
    position: usize,
}

#[allow(clippy::unnecessary_wraps)]
impl Parser {
    pub fn new(tokens: Vec<Token>, path: impl Into<PathBuf>) -> Self {
        Self {
            tokens,
            path: path.into(),
            position: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Statement>, Box<Log>> {
        let mut statements = Vec::new();

        while let Some(token) = self.tokens.get(self.position) {
            let result = match token.token_type {
                TokenType::Let => self.parse_let(self.position),
                _ => self.parse_expr(self.position),
            }?;

            statements.extend(result.statements);
            self.position += result.advance_by;
        }

        Ok(statements)
    }

    fn parse_let(&self, starting_position: usize) -> Result<ParseResult, Box<Log>> {
        let mut result = ParseResult {
            advance_by: 1, // Let token
            ..Default::default()
        };

        let token = &self.tokens[starting_position];
        let mut name = String::new();

        {
            let next_token = self.tokens.get(starting_position + result.advance_by);

            if next_token.is_none() {
                return Err(Box::new(
                    make_error!("Expected identifier after 'let'", location: Location::new_with_section(&self.path, token.line..=token.line, token.column..=token.column + token.len)),
                ));
            }

            match &next_token.unwrap().token_type {
                TokenType::Identifier(ident) => {
                    name.clone_from(ident);
                    result.advance_by += 1;
                }
                _ => {
                    return Err(Box::new(
                        make_error!("Expected identifier after 'let'", location: Location::new_with_section(&self.path, token.line..=token.line, token.column..=token.column + token.len)),
                    ));
                }
            }
        }

        {
            let previous_token = &self.tokens[(starting_position + result.advance_by) - 1];
            let next_token = self.tokens.get(starting_position + result.advance_by);

            if next_token.is_none() {
                return Err(Box::new(
                    make_error!("Expected '=' after identifier in variable declaration", location: Location::new_with_section(&self.path, previous_token.line..=previous_token.line, previous_token.column..=previous_token.column + previous_token.len)),
                ));
            }

            match &next_token.unwrap().token_type {
                TokenType::Equals => result.advance_by += 1,
                _ => {
                    return Err(Box::new(
                        make_error!("Expected '=' after identifier in variable declaration", location: Location::new_with_section(&self.path, previous_token.line..=previous_token.line, previous_token.column..=previous_token.column + previous_token.len)),
                    ));
                }
            }
        }

        let value = self.parse_expr(starting_position + result.advance_by)?;
        result.advance_by += value.advance_by;

        if value.statements.len() != 1 {
            let token = &self.tokens[starting_position + result.advance_by - 1];
            return Err(Box::new(
                make_error!("Expected single expression in variable declaration", location: Location::new_with_section(&self.path, token.line..=token.line, token.column..=token.column + token.len)),
            ));
        }

        if let Statement::Expr(expr) = &value.statements[0] {
            result.statements.push(Statement::Let { name, value: expr.clone() });
        }

        Ok(result)
    }

    fn parse_expr(&self, starting_position: usize) -> Result<ParseResult, Box<Log>> {
        let mut result = ParseResult {
            advance_by: 1, // Expr
            ..Default::default()
        };

        let token = &self.tokens[starting_position];

        let next_token = self.tokens.get(starting_position + result.advance_by);

        if let Some(next_token) = next_token {
            match next_token.token_type {
                TokenType::Plus | TokenType::Minus | TokenType::Multiply | TokenType::Slash => {
                    dbg!(self.parse_binary_op(starting_position)?);
                }
                _ => (),
            }
        }

        let expr = match &token.token_type {
            TokenType::String(v) => Expr::Literal(Literal::String(v.clone())),
            TokenType::Number(v) => Expr::Literal(Literal::Number(*v)),
            TokenType::Float(v) => Expr::Literal(Literal::Float(*v)),
            TokenType::Bool(v) => Expr::Literal(Literal::Bool(*v)),
            TokenType::Identifier(v) => Expr::Identifier(v.clone()),
            other => {
                return Err(Box::new(
                    make_error!(format!("Unexpected token found in expression: {other}"), location: Location::new_with_section(&self.path, token.line..=token.line, token.column..=token.column + token.len)),
                ));
            }
        };

        result.statements.push(Statement::Expr(expr));

        Ok(result)
    }

    fn parse_binary_op(&self, starting_position: usize) -> Result<ParseResult, Box<Log>> {
        let mut result = ParseResult {
            advance_by: 3, // Binary OP
            ..Default::default()
        };

        let token = &self.tokens[starting_position];

        let expr = match &token.token_type {
            TokenType::Plus => Expr::BinaryOp(BinaryOperator::Plus),
            TokenType::Minus => Expr::BinaryOp(BinaryOperator::Minus),
            TokenType::Multiply => Expr::BinaryOp(BinaryOperator::Multiply),
            TokenType::Slash => Expr::BinaryOp(BinaryOperator::Divide),
            _ => {
                return Err(Box::new(
                    make_error!("Unexpected token found in expression", location: Location::new_with_section(&self.path, token.line..=token.line, token.column..=token.column + token.len)),
                ));
            }
        };

        Ok(result)
    }
}
