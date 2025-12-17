use crate::parser::{
    ast::{
        ExprResult,
        types::{Error, Expr},
    },
    lexer::types::TokenKind,
};
use miette::SourceSpan;
use tl_macro::consume;

impl super::Parser {
    pub(super) fn parse_ident(&mut self) -> ExprResult {
        let ident = self.parse_ident_plain()?;
        Ok(Expr::ident(ident.0, ident.1))
    }

    pub(super) fn parse_ident_plain(&mut self) -> Result<(String, SourceSpan), Error> {
        let token = consume!("identifier", TokenKind::Identifier(_))?.clone();

        match &token.kind {
            TokenKind::Identifier(name) => Ok((name.clone(), token.span)),
            _ => unreachable!(),
        }
    }
}
