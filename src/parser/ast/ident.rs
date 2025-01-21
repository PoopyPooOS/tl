use super::{
    types::{Expr, ExprType},
    ExprResult,
};
use crate::parser::{
    ast::{advance, consume, err, raw_err, Context},
    tokenizer::types::TokenType,
};
use logger::location::Section;

impl super::Parser {
    pub(super) fn parse_ident(&mut self) -> ExprResult {
        let mut is_call = false;
        let mut is_index = false;

        let last_context = self.context.clone();

        if let Some(next_token) = self.tokens.get(self.position.saturating_add(1)) {
            if next_token.token_type == TokenType::LParen {
                is_call = true;
                self.context = Context::CallArgs;
            }
            is_index = next_token.token_type == TokenType::LBracket;
        }

        let name = match advance!(self) {
            Some(token) => {
                if let TokenType::Identifier(name) = &token.token_type {
                    (token.clone(), name.clone())
                } else {
                    unreachable!();
                }
            }
            _ => unreachable!(),
        };

        if !is_call && !is_index {
            return Ok(Expr::new(ExprType::Identifier(name.1), name.0.section));
        }

        if is_call {
            let name = (name.0.clone(), name.1);
            let mut call_args = Vec::new();
            consume!(self, LParen);

            while let Some(next_token) = self.tokens.get(self.position) {
                if next_token.token_type == TokenType::RParen {
                    break;
                }

                let expr = self.parse_expr()?;
                call_args.push(expr);
            }

            let end = consume!(self, RParen);
            let start = &name.0;
            self.context = last_context;

            return Ok(Expr::new(
                ExprType::Call {
                    name: name.1,
                    args: call_args,
                },
                Section::merge_start_end(&start.section, &end.section),
            ));
        }

        let name_token = name.0.clone();
        consume!(self, LBracket);

        let index = match advance!(self) {
            Some(token) => match &token.token_type {
                TokenType::Int(v) => {
                    if *v < 0 {
                        return err!(NegativeArrayIndex, self.location_from_token(token));
                    }

                    usize::try_from(*v).map_err(|_| {
                        raw_err!(NegativeArrayIndex, self.location_from_token(token))
                    })?
                }
                _ => return err!(ExpectedToken(TokenType::Int(0))),
            },
            _ => return err!(NoTokensLeft),
        };

        let end = consume!(self, RBracket);

        Ok(Expr::new(
            ExprType::ArrayIndex(name.1, index),
            Section::merge_start_end(&name_token.section, &end.section),
        ))
    }
}
