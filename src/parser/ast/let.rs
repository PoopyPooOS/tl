use crate::{
    merge_spans,
    parser::{
        ast::{
            ExprResult,
            types::{Error, ErrorKind, Expr, ExprKind},
        },
        lexer::types::TokenKind,
    },
};
use tl_macro::{consume, peek_or_err};

impl super::Parser {
    pub(super) fn parse_let(&mut self) -> ExprResult {
        let start = consume!("'let'", TokenKind::Let)?.clone();

        let mut bindings = Vec::new();

        loop {
            let token = peek_or_err!(0)?.clone();

            if token.kind == TokenKind::In {
                break;
            }

            let key = self.parse()?;
            consume!("'='", TokenKind::Equals)?;
            let value = self.parse()?;

            bindings.push((key, value));
        }

        consume!("'in'", TokenKind::In)?;

        let body = self.parse()?;
        let end_span = body.span;

        Ok(Expr::new(
            ExprKind::LetIn {
                bindings,
                expr: Box::new(body),
            },
            merge_spans(start.span, end_span),
        ))
    }
}
