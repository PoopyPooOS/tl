use crate::{
    merge_spans,
    parser::{
        ast::{
            ExprResult,
            types::{Expr, ExprKind, Literal},
        },
        lexer::types::TokenKind,
    },
};
use tl_macro::{change_pos, consume, peek};

impl super::Parser {
    pub(super) fn parse_array(&mut self) -> ExprResult {
        let start = consume!("'['", TokenKind::LBracket)?.clone();

        let mut items = Vec::new();
        while let Some(next_token) = peek!(0) {
            if next_token.kind == TokenKind::RBracket {
                change_pos!(1);
                break;
            }

            let expr = self.parse()?;
            items.push(expr);
        }

        let end = peek!(-1).unwrap_or(&start);

        Ok(Expr::new(
            ExprKind::Literal(Literal::Array(items)),
            merge_spans(start.span, end.span),
        ))
    }
}
