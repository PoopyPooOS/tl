use super::{
    types::{Error, ErrorType, Statement, StatementType},
    StatementResult,
};
use crate::parser::tokenizer::types::TokenType;

impl super::Parser {
    pub(super) fn parse_let(&mut self) -> StatementResult {
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
}
