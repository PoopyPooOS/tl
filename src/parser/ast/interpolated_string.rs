use super::{
    types::{Error, ErrorType, Expr, ExprType, Literal, StatementType},
    ExprResult,
};
use crate::parser::tokenizer::types::{Token, TokenType};

impl super::Parser {
    pub(super) fn parse_interpolated_string(&mut self, v: &[Token]) -> ExprResult {
        let mut result: Vec<Expr> = Vec::new();
        let start = self
            .tokens
            .get(self.position)
            .ok_or_else(|| Box::new(Error::new(ErrorType::ExpectedToken(TokenType::InterpolatedString(vec![])), None)))?;

        for token in v {
            match &token.token_type {
                TokenType::String(v) => {
                    result.push(Expr::new(
                        ExprType::Literal(Literal::String(v.clone())),
                        token.line,
                        token.cols.clone(),
                    ));
                }
                TokenType::InterpolatedString(v) => {
                    let ast = Self::new(v.clone(), self.source.clone()).parse()?;
                    if let Some(first) = ast.first() {
                        let StatementType::Expr(first) = &first.statement_type else {
                            unreachable!("Interpolated strings can only contain expressions");
                        };

                        result.push(first.clone());
                    }
                }
                _ => {
                    // FIXME: Cloning `self.source` is very inefficient.
                    let ast = Self::new(vec![token.clone()], self.source.clone()).parse()?;
                    if let Some(first) = ast.first() {
                        let StatementType::Expr(first) = &first.statement_type else {
                            unreachable!("Interpolated strings can only contain expressions");
                        };

                        result.push(first.clone());
                    }
                }
            }
        }

        Ok(Expr::new(
            ExprType::Literal(Literal::InterpolatedString(result)),
            start.line,
            start.cols.clone(),
        ))
    }
}
