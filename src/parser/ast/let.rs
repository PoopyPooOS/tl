use super::{
    types::{Statement, StatementType},
    StatementResult,
};
use crate::parser::{
    ast::{advance, consume, err, raw_err},
    tokenizer::types::TokenType,
};
use logger::location::Section;

impl super::Parser {
    pub(super) fn parse_let(&mut self) -> StatementResult {
        let start = self
            .tokens
            .get(self.position)
            .ok_or_else(|| raw_err!(NoTokensLeft))?
            .clone();

        consume!(self, Let);

        let name = match advance!(self) {
            Some(token) => {
                if let TokenType::Identifier(name) = &token.token_type {
                    (token.clone(), name.clone())
                } else {
                    return err!(NoIdentifierAfterLet, self.location_from_token(token));
                }
            }
            _ => {
                return err!(
                    NoIdentifierAfterLet,
                    self.location_from_token(
                        self.tokens
                            .get(self.position.saturating_sub(1))
                            .ok_or_else(|| raw_err!(NoTokensLeft))?,
                    )
                );
            }
        };

        consume!(self, Equals);
        let value = self.parse_expr()?;
        let end_section = value.section.clone();

        Ok(vec![Statement::new(
            StatementType::Let {
                name: name.1,
                value,
            },
            Section::merge_start_end(&start.section, &end_section),
        )])
    }
}
