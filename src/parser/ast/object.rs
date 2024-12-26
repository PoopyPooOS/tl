use super::{
    types::{Error, ErrorType, Expr, ExprType, Literal},
    Context, ExprResult,
};
use crate::parser::tokenizer::types::TokenType;
use std::collections::BTreeMap;

impl super::Parser {
    pub(super) fn parse_object(&mut self) -> ExprResult {
        let start = self
            .tokens
            .get(self.position)
            .ok_or_else(|| Box::new(Error::new(ErrorType::NoTokensLeft, None)))?
            .clone();

        self.consume(TokenType::LBrace)?;
        let last_context = self.context.clone();
        self.context = Context::Object;

        let mut fields = BTreeMap::new();

        loop {
            let token = self
                .tokens
                .get(self.position)
                .ok_or_else(|| Box::new(Error::new(ErrorType::NoTokensLeft, None)))?;

            if token.token_type == TokenType::RBrace {
                self.consume(TokenType::RBrace)?;
                break;
            }

            let next_token = {
                let token = self.tokens.get(self.position);
                if token.is_some() {
                    self.position = self.position.saturating_add(1);
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
                        self.location_from_token(
                            self.tokens
                                .get(self.position)
                                .ok_or_else(|| Box::new(Error::new(ErrorType::NoTokensLeft, None)))?,
                        ),
                    )))
                }
            };

            let next_token = {
                let token = self.tokens.get(self.position);
                if token.is_some() {
                    self.position = self.position.saturating_add(1);
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
        let end = self
            .tokens
            .get(self.position.saturating_sub(1))
            .ok_or_else(|| Box::new(Error::new(ErrorType::NoTokensLeft, None)))?
            .cols
            .end();

        Ok(Expr::new(
            ExprType::Literal(Literal::Object(fields)),
            start.line,
            *start.cols.start()..=*end,
        ))
    }
}
