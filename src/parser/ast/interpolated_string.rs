use super::{
    ExprResult,
    types::{Expr, ExprKind, Literal},
};
use crate::parser::{
    ast::types::{Error, ErrorKind},
    lexer::types::{Token, TokenKind},
};

impl super::Parser {
    pub(super) fn parse_interpolated_string(&mut self, v: &[Token]) -> ExprResult {
        let mut result = Vec::new();
        let start = self.tokens.get(self.pos).ok_or(Error::new(
            ErrorKind::ExpectedToken {
                expected: "interpolated string".into(),
                found: None,
            },
            self.source.clone(),
            self.closest_span(),
        ))?;

        for token in v {
            match &token.kind {
                TokenKind::String(v) => {
                    result.push(Expr::new(
                        ExprKind::Literal(Literal::String(v.clone())),
                        token.span,
                    ));
                }
                TokenKind::InterpolatedString(v) => {
                    let ast = Self::new(v.clone(), self.source.clone()).parse()?;
                    result.push(ast.clone());
                }
                _ => {
                    let ast = Self::new(vec![token.clone()], self.source.clone()).parse()?;
                    result.push(ast.clone());
                }
            }
        }

        // Consume the interpolated string
        self.pos = self.pos.saturating_add(1);

        Ok(Expr::new(
            ExprKind::Literal(Literal::InterpolatedString(result)),
            start.span,
        ))
    }
}
