use super::{
    types::{Expr, ExprType},
    Context, ExprResult,
};
use crate::parser::{
    ast::{consume, err, raw_err},
    tokenizer::types::TokenType,
};
use logger::location::Section;

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
                        ExpectedOneOfTokens(vec![TokenType::Identifier("identifier".to_string())]),
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
        let end = consume!(self, RBrace);
        self.context = last_context;

        Ok(Expr::new(
            ExprType::FnDecl { args, body },
            Section::merge_start_end(&start.section, &end.section),
        ))
    }
}
