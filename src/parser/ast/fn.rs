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
    pub(super) fn parse_fn_decl(&mut self) -> ExprResult {
        let start_span = peek_or_err!(0)?.span;

        // Arg
        let arg = consume!(
            "identifier (as function argument)",
            TokenKind::Identifier(_)
        )?
        .kind
        .clone();
        let TokenKind::Identifier(arg) = arg else {
            unreachable!("checked by `consume!` macro")
        };

        // Body
        consume!("':'", TokenKind::Colon)?;

        let expr = self.parse()?;

        let span = merge_spans(start_span, expr.span);

        Ok(Expr::new(
            ExprKind::FnDecl {
                arg,
                expr: Box::new(expr),
            },
            span,
        ))
    }
}
