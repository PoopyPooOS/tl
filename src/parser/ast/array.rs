use super::{
    ExprResult,
    types::{Expr, ExprKind, Literal},
};
use crate::{
    merge_spans,
    parser::{
        ast::{
            consume,
            types::{Error, ErrorKind},
        },
        lexer::types::TokenKind,
    },
};

impl super::Parser {
    pub(super) fn parse_array(&mut self) -> ExprResult {
        let start = self
            .tokens
            .get(self.pos)
            .ok_or(Error::new(
                ErrorKind::ExpectedToken {
                    expected: "'{'".into(),
                    found: None,
                },
                self.source.clone(),
                self.closest_span(),
            ))?
            .clone();

        consume!(self, LBracket);

        let mut array = Vec::new();
        while let Some(next_token) = self.tokens.get(self.pos).cloned() {
            if next_token.kind == TokenKind::RBracket {
                consume!(self, RBracket);
                break;
            }

            let expr = self.parse()?;
            array.push(expr);
        }

        let end = self
            .tokens
            .get(self.pos.saturating_sub(1))
            .unwrap_or(&start);

        Ok(Expr::new(
            ExprKind::Literal(Literal::Array(array)),
            merge_spans(start.span, end.span),
        ))
    }
}
