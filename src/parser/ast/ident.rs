use crate::parser::{
    ast::{ExprResult, types::Expr},
    lexer::types::TokenKind,
};
use tl_macro::{change_pos, check, consume};

impl super::Parser {
    pub(super) fn parse_ident(&mut self) -> ExprResult {
        let token = consume!("identifier", TokenKind::Identifier(_))?.clone();

        let expr = match &token.kind {
            TokenKind::Identifier(_) if check!(0, TokenKind::Colon) => {
                // The current identifier would be the function's argument,
                // so we have to go back so that `parse_fn_decl` can parse the identifier as the function argument.
                change_pos!(-1);
                self.parse_fn_decl()?
            }
            TokenKind::Identifier(name) => Expr::ident(name.clone(), token.span),
            _ => unreachable!(),
        };

        Ok(expr)
    }
}
