use super::{
    types::{Expr, ExprType, Literal},
    Context, ExprResult,
};
use crate::parser::{
    ast::{advance, consume, err, raw_err},
    tokenizer::types::TokenType,
};
use logger::location::Section;
use std::collections::BTreeMap;

impl super::Parser {
    pub(super) fn parse_object(&mut self) -> ExprResult {
        let start = self
            .tokens
            .get(self.position)
            .ok_or_else(|| raw_err!(NoTokensLeft))?
            .clone();

        consume!(self, LBrace);
        let last_context = self.context.clone();
        self.context = Context::Object;

        let mut fields = BTreeMap::new();

        loop {
            let token = self
                .tokens
                .get(self.position)
                .ok_or_else(|| raw_err!(NoTokensLeft))?;

            if token.token_type == TokenType::RBrace {
                consume!(self, RBrace);
                break;
            }

            let key = match advance!(self) {
                Some(token) => {
                    if let TokenType::Identifier(name) = &token.token_type {
                        name.clone()
                    } else {
                        return err!(
                            ExpectedTokenGot(
                                TokenType::Identifier(String::new()),
                                token.token_type.clone()
                            ),
                            self.location_from_token(token)
                        );
                    }
                }
                _ => {
                    return err!(
                        ExpectedOneOfTokens(vec![TokenType::Identifier(String::new())]),
                        self.location_from_token(
                            self.tokens
                                .get(self.position)
                                .ok_or_else(|| raw_err!(NoTokensLeft))?,
                        )
                    );
                }
            };

            match advance!(self) {
                Some(token) => match token.token_type {
                    TokenType::Equals => (),
                    TokenType::Colon => {
                        return err!(UnexpectedColonInObjectKV, self.location_from_token(token));
                    }
                    _ => {}
                },
                _ => {
                    return err!(
                        ExpectedSeperatorInObjectKV,
                        #[allow(
                            clippy::unwrap_used,
                            reason = "`location_from_token` always returns `Some`"
                        )]
                        self.tokens
                            .get(self.position)
                            .map(|token| self.location_from_token(token).unwrap())
                    );
                }
            }

            let value = self.parse_expr()?;

            fields.insert(key, value);
        }

        self.context = last_context;
        let end = self
            .tokens
            .get(self.position.saturating_sub(1))
            .ok_or_else(|| raw_err!(NoTokensLeft))?;

        Ok(Expr::new(
            ExprType::Literal(Literal::Object(fields)),
            Section::merge_start_end(&start.section, &end.section),
        ))
    }
}
