use super::{
    types::{Error, ErrorType, Expr, ExprType},
    ExprResult,
};
use crate::parser::tokenizer::types::TokenType;

impl super::Parser {
    pub(super) fn parse_ident(&mut self) -> ExprResult {
        let mut is_call = false;
        let mut is_access = false;
        let mut is_index = false;

        if let Some(next_token) = self.tokens.get(self.position.saturating_add(1)) {
            if next_token.token_type.is_binary_operator() {
                return self.parse_binary_op(0);
            }

            is_call = next_token.token_type == TokenType::LParen;
            is_access = next_token.token_type == TokenType::Dot;
            is_index = next_token.token_type == TokenType::LBracket;
        }

        let next_token = {
            let token = self.tokens.get(self.position).cloned();
            if token.is_some() {
                self.position = self.position.saturating_add(1);
            }
            token
        };

        let name = match next_token {
            Some(token) => {
                if let TokenType::Identifier(name) = &token.token_type {
                    (token.clone(), name.clone())
                } else {
                    unreachable!();
                }
            }
            _ => unreachable!(),
        };

        if !is_call && !is_access && !is_index {
            return Ok(Expr::new(ExprType::Identifier(name.1), name.0.line, name.0.cols.clone()));
        }

        if is_call {
            let name = (name.0.clone(), name.1);
            let mut call_args = Vec::new();
            self.consume(TokenType::LParen)?;

            // Empty args
            if let Some(token) = self.tokens.get(self.position) {
                if token.token_type == TokenType::RParen {
                    self.position = self.position.saturating_add(1);
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

            let start = &name.0;
            let end = &self.tokens.iter().filter(|token| token.line == start.line).collect::<Vec<_>>();
            end.clone().sort_by(|a, b| a.cols.end().cmp(b.cols.end()));
            let end = end.last().copied().unwrap_or(
                self.tokens
                    .get(self.position.saturating_sub(1))
                    .ok_or_else(|| Box::new(Error::new(ErrorType::NoTokensLeft, None)))?,
            );

            return Ok(Expr::new(
                ExprType::Call {
                    name: name.1,
                    args: call_args,
                },
                name.0.line,
                *start.cols.start()..=*end.cols.end(),
            ));
        }

        if is_access {
            // Clone otherwise it would require 2 mutable borrows.
            let name_token = name.0.clone();
            let mut paths: Vec<String> = vec![name.1];

            while let Some(token) = self.advance() {
                if let TokenType::Identifier(v) = &token.token_type {
                    paths.push(v.clone());
                }
            }

            let line = name_token.line;
            let token = self
                .tokens
                .get(self.position.saturating_sub(1))
                .ok_or_else(|| Box::new(Error::new(ErrorType::NoTokensLeft, None)))?;

            return Ok(Expr::new(
                ExprType::DotAccess(paths),
                line,
                *name_token.cols.start()..=*token.cols.end(),
            ));
        }

        let name_token = name.0.clone();
        self.consume(TokenType::LBracket)?;

        let next_token = {
            let token = self.tokens.get(self.position);
            if token.is_some() {
                self.position = self.position.saturating_add(1);
            }
            token
        };

        let index = match next_token {
            Some(token) => {
                dbg!(&token.token_type);
                match &token.token_type {
                    TokenType::Int(v) => {
                        if *v < 0 {
                            return Err(Box::new(Error::new(ErrorType::NegativeArrayIndex, self.location_from_token(token))));
                        }

                        usize::try_from(*v)
                            .map_err(|_| Box::new(Error::new(ErrorType::NegativeArrayIndex, self.location_from_token(token))))?
                    }
                    _ => return Err(Box::new(Error::new(ErrorType::ExpectedToken(TokenType::Int(0)), None))),
                }
            }
            None => return Err(Box::new(Error::new(ErrorType::NoTokensLeft, None))),
        };

        self.consume(TokenType::RBracket)?;

        let line = name_token.line;
        let token = self
            .tokens
            .get(self.position.saturating_sub(1))
            .ok_or_else(|| Box::new(Error::new(ErrorType::NoTokensLeft, None)))?;

        Ok(Expr::new(
            ExprType::ArrayIndex(name.1, index),
            line,
            *name_token.cols.start()..=*token.cols.end(),
        ))
    }
}
