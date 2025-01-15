use super::{
    types::{Expr, ExprType},
    Context, ExprResult,
};
use crate::parser::{
    ast::{consume, err, raw_err},
    tokenizer::types::TokenType,
};

impl super::Parser {
    pub(super) fn parse_fn_decl(&mut self) -> ExprResult {
        let start = self
            .tokens
            .get(self.position)
            .ok_or_else(|| raw_err!(NoTokensLeft))?
            .clone();

        // Args
        consume!(self, LParen);
        let mut args = Vec::new();

        while let Some(next_token) = self.tokens.get(self.position) {
            if next_token.token_type == TokenType::RParen {
                break;
            }

            let name = match self.tokens.get(self.position) {
                Some(token) => match &token.token_type {
                    TokenType::Identifier(name) => name.clone(),
                    _ => {
                        return err!(
                            ExpectedTokenGot(
                                TokenType::Identifier("identifier".to_string()),
                                token.token_type.clone()
                            ),
                            self.location_from_token(token)
                        );
                    }
                },
                _ => {
                    return err!(
                        ExpectedToken(TokenType::Identifier("identifier".to_string())),
                        self.location_from_token(
                            self.tokens
                                .get(self.position)
                                .ok_or_else(|| raw_err!(NoTokensLeft))?
                        )
                    );
                }
            };

            self.position = self.position.saturating_add(1);

            args.push(name);
        }

        consume!(self, RParen);

        // Body
        consume!(self, LBrace);
        let last_context = self.context.clone();
        self.context = Context::Function;

        let body = self.parse()?;
        consume!(self, RBrace);
        self.context = last_context;

        // TODO: Remove the `line` field in locations and replace it with just a byte range.
        // This would make it harder to get the line number of the error in the logger.
        // But at the end of the day it's way more flexible and I wouldn't have to do this garbage.
        let end = &self
            .tokens
            .iter()
            .filter(|token| token.line == start.line)
            .collect::<Vec<_>>();
        end.clone().sort_by(|a, b| a.cols.end().cmp(b.cols.end()));
        let end = end.last().copied().unwrap_or(
            self.tokens
                .get(self.position.saturating_sub(1))
                .ok_or_else(|| raw_err!(NoTokensLeft))?,
        );

        Ok(Expr::new(
            ExprType::FnDecl { args, body },
            start.line,
            *start.cols.start()..=*end.cols.end(),
        ))
    }
}
