use crate::parser::{
    ast::{
        ExprResult,
        types::{Expr, ExprKind, Literal},
    },
    lexer::types::{Token, TokenKind},
};
use tl_macro::consume;

impl super::Parser {
    pub(super) fn parse_interpolated_string(&mut self, v: &[Token]) -> ExprResult {
        let start = consume!("interpolated string", TokenKind::InterpolatedString(_))?;

        let mut result = Vec::new();

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

        Ok(Expr::new(
            ExprKind::Literal(Literal::InterpolatedString(result)),
            start.span,
        ))
    }
}
