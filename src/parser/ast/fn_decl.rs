use super::{
    types::{Error, ErrorType, Expr, ExprType},
    Context, ExprResult,
};
use crate::parser::tokenizer::types::TokenType;

impl super::Parser {
    pub(super) fn parse_fn_decl(&mut self) -> ExprResult {
        let start = self.tokens[self.position].clone();

        // Args
        self.consume(TokenType::LParen)?;
        let mut args = Vec::new();

        while let Some(next_token) = self.tokens.get(self.position) {
            if next_token.token_type == TokenType::RParen {
                break;
            }

            let name = match self.tokens.get(self.position) {
                Some(token) => match &token.token_type {
                    TokenType::Identifier(name) => name.clone(),
                    _ => {
                        return Err(Box::new(Error::new(
                            ErrorType::ExpectedTokenGot(TokenType::Identifier("identifier".to_string()), token.token_type.clone()),
                            self.location_from_token(token),
                        )))
                    }
                },
                _ => {
                    return Err(Box::new(Error::new(
                        ErrorType::ExpectedToken(TokenType::Identifier("identifier".to_string())),
                        self.location_from_token(&self.tokens[self.position]),
                    )))
                }
            };

            self.position += 1;

            self.consume(TokenType::Colon)?;
            let arg_type = match self.tokens.get(self.position) {
                Some(token) => match &token.token_type {
                    TokenType::Identifier(arg_type) => arg_type.clone(),
                    _ => {
                        return Err(Box::new(Error::new(
                            ErrorType::ExpectedTokenGot(TokenType::Identifier("identifier".to_string()), token.token_type.clone()),
                            self.location_from_token(token),
                        )))
                    }
                },
                _ => {
                    return Err(Box::new(Error::new(
                        ErrorType::ExpectedToken(TokenType::Identifier("identifier".to_string())),
                        self.location_from_token(&self.tokens[self.position]),
                    )))
                }
            };

            self.position += 1;

            args.push((name, arg_type));

            // Consume commas if they exist.
            if let Some(token) = self.tokens.get(self.position)
                && token.token_type == TokenType::Comma
            {
                self.position += 1;
            }
        }

        self.consume(TokenType::RParen)?;

        // Return type
        let return_type = {
            if self
                .tokens
                .get(self.position)
                .is_some_and(|token| token.token_type == TokenType::LBrace)
            {
                None
            } else {
                self.position += 1;

                match self.tokens.get(self.position) {
                    Some(token) => match &token.token_type {
                        TokenType::Identifier(arg_type) => {
                            self.position += 1;
                            Some(arg_type.clone())
                        }
                        _ => {
                            return Err(Box::new(Error::new(
                                ErrorType::ExpectedTokenGot(TokenType::Identifier("identifier".to_string()), token.token_type.clone()),
                                self.location_from_token(token),
                            )))
                        }
                    },
                    _ => None,
                }
            }
        };

        // Body
        self.consume(TokenType::LBrace)?;
        let last_context = self.context.clone();
        self.context = Context::Function;

        let body = self.parse()?;
        self.consume(TokenType::RBrace)?;
        self.context = last_context;

        // TODO: Remove the `line` field in locations and replace it with just a byte range.
        // This would make it harder to get the line number of the error in the logger.
        // But at the end of the day it's way more flexible and I wouldn't have to do this garbage.
        let end = &self.tokens.iter().filter(|token| token.line == start.line).collect::<Vec<_>>();
        end.clone().sort_by(|a, b| a.cols.end().cmp(b.cols.end()));
        let end = end.last().copied().unwrap_or(&self.tokens[self.position - 1]);

        Ok(Expr::new(
            ExprType::FnDecl { args, return_type, body },
            start.line,
            *start.cols.start()..=*end.cols.end(),
        ))
    }
}
