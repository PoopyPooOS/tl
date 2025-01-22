use super::{
    types::{Error, ErrorType, Value},
    ValueResult,
};
use crate::parser::ast::types::{Expr, ExprType, Literal};
use std::collections::BTreeMap;

#[allow(clippy::todo, reason = "This is not production code")]
impl super::Scope {
    #[allow(clippy::expect_used)]
    pub(super) fn eval_expr(&mut self, expr: &Expr) -> ValueResult {
        match &expr.expr_type {
            ExprType::Literal(literal) => self.eval_literal(literal),
            ExprType::Not(expr) => Ok(Value::Boolean(!self.eval_expr(expr)?.is_truthy())),
            ExprType::Identifier(ident) => Ok(self
                .fetch_var(ident)
                .ok_or_else(|| {
                    Box::new(Error::new(
                        ErrorType::VariableDoesntExist(ident.clone()),
                        self.location_from_expr(expr),
                    ))
                })?
                .clone()),
            ExprType::ArrayIndex(ident, index) => {
                let var = self
                    .fetch_var(ident)
                    .ok_or_else(|| {
                        Box::new(Error::new(
                            ErrorType::VariableDoesntExist(ident.clone()),
                            self.location_from_expr(expr),
                        ))
                    })?
                    .clone();

                match var {
                    Value::Array(v) => {
                        let item = v.get(*index).unwrap_or(&Value::Null).clone();

                        if item == Value::Null {
                            return Err(Box::new(Error::new(
                                ErrorType::IndexOutOfBounds(*index, v.len()),
                                self.location_from_expr(expr),
                            )));
                        }

                        Ok(item)
                    }
                    _ => unreachable!(),
                }
            }
            // TODO: Construct this expr type in the AST, right now field access is still just binary ops with `Dot` as the operator.
            ExprType::FieldAccess { base, path } => {
                let base = self.eval_expr(base)?;
                let mut value = base.clone();

                let path = path.iter().map(|expr| {
                    (
                        expr,
                        match &expr.expr_type {
                            ExprType::Identifier(v) => v.clone(),
                            _ => unreachable!(),
                        },
                    )
                });

                for (idx, (expr, ident)) in path.enumerate() {
                    value = if idx == 0 {
                        base.access(&ident)
                    } else {
                        value.access(&ident)
                    };

                    if value == Value::Null {
                        return Err(Box::new(Error::new(
                            ErrorType::FieldDoesntExist(ident),
                            self.location_from_expr(expr),
                        )));
                    }
                }

                Ok(value)
            }
            ExprType::BinaryOp {
                left,
                operator,
                right,
            } => Ok(self.eval_binary_op(left, operator, right)?),
            ExprType::FnDecl { args, body } => Ok(Value::Function {
                args: args.clone(),
                body: body.clone(),
            }),
            ExprType::Call { .. } => self.eval_call(expr),
        }
    }

    pub(super) fn eval_literal(&mut self, literal: &Literal) -> ValueResult {
        match literal {
            Literal::Null => Ok(Value::Null),
            Literal::Int(v) => Ok(Value::Int(*v)),
            Literal::Float(v) => Ok(Value::Float(*v)),
            Literal::Boolean(v) => Ok(Value::Boolean(*v)),
            Literal::String(v) => Ok(Value::String(v.clone())),
            Literal::InterpolatedString(v) => {
                let mut value = String::new();

                for expr in v {
                    let expr = self.eval_expr(expr)?;
                    value.push_str(&expr.to_string());
                }

                Ok(Value::String(value))
            }
            Literal::Array(v) => {
                let mut values = Vec::new();

                for expr in v {
                    values.push(self.eval_expr(expr)?);
                }

                Ok(Value::Array(values))
            }
            Literal::Object(v) => {
                let mut values = BTreeMap::new();

                for (k, expr) in v {
                    values.insert(k.clone(), self.eval_expr(expr)?);
                }

                Ok(Value::Object(values))
            }
        }
    }
}
