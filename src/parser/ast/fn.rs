use crate::{
    merge_spans,
    parser::{
        ast::{
            ExprResult,
            types::{Expr, ExprKind, FnArg, Type},
        },
        lexer::types::TokenKind,
    },
};
use tl_macro::{change_pos, check, consume};

impl super::Parser {
    pub(super) fn parse_fn_decl(&mut self) -> ExprResult {
        let start_span = consume!("'|'", TokenKind::Pipe)?.span;

        let mut args = Vec::new();

        loop {
            let name = consume!(
                "identifier (as function argument)",
                TokenKind::Identifier(_)
            )?
            .kind
            .clone();
            let TokenKind::Identifier(name) = name else {
                unreachable!("checked by `consume!` macro")
            };

            let ty = if check!(0, TokenKind::Colon) {
                change_pos!(1);
                self.parse_type()?
            } else {
                Type::default()
            };

            args.push(FnArg { name, ty });

            if check!(0, TokenKind::Pipe) {
                change_pos!(1);
                break;
            }

            consume!("','", TokenKind::Comma)?;
        }

        // Return type
        let ret_ty = if check!(0, TokenKind::Colon) {
            change_pos!(1);
            self.parse_type()?
        } else {
            Type::default()
        };

        // Body
        let expr = self.parse()?;

        let span = merge_spans(start_span, expr.span);

        Ok(Expr::new(
            ExprKind::Function {
                args,
                ret_ty: Box::new(ret_ty),
                expr: Box::new(expr),
            },
            span,
        ))
    }
}
