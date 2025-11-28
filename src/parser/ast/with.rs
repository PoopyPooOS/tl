use crate::{
    merge_spans,
    parser::{
        ast::{
            ExprResult,
            types::{Expr, ExprKind},
        },
        lexer::types::TokenKind,
    },
};
use tl_macro::consume;

impl super::Parser {
    pub(super) fn parse_with(&mut self) -> ExprResult {
        let with = consume!("with (keyword)", TokenKind::With)?.clone();

        let object = self.parse()?;

        let expr = self.parse()?;

        let span = merge_spans(with.span, expr.span);

        Ok(Expr::new(
            ExprKind::With {
                object: Box::new(object),
                expr: Box::new(expr),
            },
            span,
        ))
    }
}
