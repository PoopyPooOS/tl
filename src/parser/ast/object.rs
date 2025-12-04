use crate::{
    merge_spans,
    parser::{
        ast::{
            ExprResult,
            types::{Error, ErrorKind, Expr, ExprKind, Literal},
        },
        lexer::types::TokenKind,
    },
};
use tl_macro::{consume, peek_or_err};

impl super::Parser {
    pub(super) fn parse_object(&mut self) -> ExprResult {
        let start = consume!("'{'", TokenKind::LBrace)?.clone();

        let mut fields = Vec::new();

        loop {
            let token = peek_or_err!(0)?.clone();

            if token.kind == TokenKind::RBrace {
                break;
            }

            let key = self.parse()?;

            consume!("'='", TokenKind::Equals)?;
            let value = self.parse()?;

            fields.push((key, value));
        }

        let end = consume!("'}'", TokenKind::RBrace)?;

        Ok(Expr::new(
            ExprKind::Literal(Literal::Object(fields)),
            merge_spans(start.span, end.span),
        ))
    }
}
