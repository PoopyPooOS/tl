use crate::parser::{
    ast::{
        ExprResult,
        types::{Expr, ExprKind, Literal},
    },
    lexer::types::{Token, TokenKind},
};
use tl_macro::consume;

impl super::Parser {
    pub(super) fn parse_interpolated_path(&mut self, v: &[Token]) -> ExprResult {
        let start = consume!("interpolated path", TokenKind::InterpolatedPath(_))?;

        let mut result = Vec::new();

        for token in v {
            match &token.kind {
                TokenKind::Path(v) => {
                    let path = if result.is_empty() {
                        self.resolve_path(v.clone())
                    } else {
                        v.clone()
                    };

                    result.push(Expr::new(
                        ExprKind::Literal(Literal::Path(path)),
                        token.span,
                    ));
                }
                TokenKind::InterpolatedPath(v) => {
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
            ExprKind::Literal(Literal::InterpolatedPath(result)),
            start.span,
        ))
    }
}
