use miette::SourceSpan;

use super::{
    ValueResult,
    types::{Error, ErrorKind, Value},
};
use crate::{
    parser::ast::types::{Expr, ExprKind, Literal},
    runtime::{Scope, ValueKind},
};
use std::collections::BTreeMap;

impl super::Scope {
    pub(super) fn eval_expr(&mut self, expr: &Expr) -> ValueResult {
        match &expr.kind {
            ExprKind::Literal(literal) => self.eval_literal(literal, expr.span),
            ExprKind::Not(body) => Ok(Value::new(
                ValueKind::Boolean(!self.eval_expr(body)?.is_truthy()),
                expr.span,
            )),
            ExprKind::Identifier(ident) => Ok(self
                .fetch_var(ident)
                .ok_or(Error::new(
                    ErrorKind::VariableNotInScope {
                        variable: expr.span,
                    },
                    self.source.clone(),
                    expr.span,
                ))?
                .clone()),
            ExprKind::ArrayIndex { base, index } => {
                let base = self.eval_expr(base)?;
                let item = base.try_index(*index);

                match item {
                    Ok(item) => Ok(item.clone()),
                    Err(len) => Err(Error::new(
                        ErrorKind::IndexOutOfBounds {
                            length: len,
                            // TODO: Add span for the index itself, not the full expr
                            index: expr.span,
                        },
                        self.source.clone(),
                        expr.span,
                    )),
                }
            }
            ExprKind::ObjectAccess { base, field } => {
                let base = self.eval_expr(base)?;
                Ok(base.access(field))
            }
            ExprKind::BinaryOp {
                left,
                operator,
                right,
            } => Ok(self.eval_binary_op(left, operator, right)?),
            ExprKind::FnDecl { args, expr: body } => Ok(Value::new(
                ValueKind::Function {
                    args: args.clone(),
                    expr: *body.clone(),
                },
                expr.span,
            )),
            ExprKind::Call { .. } => self.eval_call(expr),
            ExprKind::LetIn {
                bindings,
                expr: body,
            } => {
                let mut child_scope =
                    Scope::new(self.variables.clone(), self.source.clone(), *body.clone());

                for (name, expr) in bindings {
                    let value = child_scope.eval_expr(expr)?;
                    child_scope.define(name, value);
                }

                child_scope.eval_expr(body)
            }
        }
    }

    pub(super) fn eval_literal(&mut self, literal: &Literal, span: SourceSpan) -> ValueResult {
        match literal {
            Literal::Null => Ok(Value::new(ValueKind::Null, span)),
            Literal::Int(v) => Ok(Value::new(ValueKind::Int(*v), span)),
            Literal::Float(v) => Ok(Value::new(ValueKind::Float(*v), span)),
            Literal::Bool(v) => Ok(Value::new(ValueKind::Boolean(*v), span)),
            Literal::String(v) => Ok(Value::new(ValueKind::String(v.clone()), span)),
            Literal::InterpolatedString(v) => {
                let mut value = String::new();

                for expr in v {
                    let expr = self.eval_expr(expr)?;
                    value.push_str(&expr.to_string());
                }

                Ok(Value::new(ValueKind::String(value), span))
            }
            Literal::Path(path) => Ok(Value::new(ValueKind::Path(path.clone()), span)),
            Literal::InterpolatedPath(v) => {
                let mut value = String::new();

                for expr in v {
                    let expr = self.eval_expr(expr)?;
                    value.push_str(&expr.to_string());
                }

                Ok(Value::new(ValueKind::Path(value.into()), span))
            }
            Literal::Array(v) => {
                let mut values = Vec::new();

                for expr in v {
                    values.push(self.eval_expr(expr)?);
                }

                Ok(Value::new(ValueKind::Array(values), span))
            }
            Literal::Object(v) => {
                let mut values = BTreeMap::new();

                for (k, expr) in v {
                    values.insert(k.clone(), self.eval_expr(expr)?);
                }

                Ok(Value::new(ValueKind::Object(values), span))
            }
        }
    }
}
