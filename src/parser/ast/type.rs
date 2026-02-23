use crate::parser::{ast::ExprResult, lexer::types::TokenKind};
use tl_macro::peek;

impl super::Parser {
    pub(super) fn parse_type(&mut self) -> ExprResult {
        let mut expr = self.parse_primary()?;
        let mut full_span = expr.span;

        loop {
            match peek!(0).map(|t| &t.kind) {
                Some(TokenKind::Dot) => {
                    expr = self.parse_member_access(expr, full_span)?;
                    full_span = expr.span;
                }
                Some(TokenKind::LParen) => {
                    expr = self.parse_call(expr, full_span)?;
                    full_span = expr.span;
                }
                _ => break,
            }
        }

        Ok(expr)
    }
}
