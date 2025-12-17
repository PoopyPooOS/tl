use crate::{
    parser::ast::types::{Expr, ExprKind, Literal},
    runtime::{Error, ErrorKind, Scope, Value, ValueKind, types::ValueResult},
};
use indexmap::IndexMap;
use miette::SourceSpan;

fn create_nested_object(path: &[String], value: Value) -> Value {
    if let Some((first, rest)) = path.split_first() {
        let mut map = IndexMap::new();
        map.insert(first.clone(), create_nested_object(rest, value));
        Value::new_builtin(ValueKind::Object(map))
    } else {
        value
    }
}

impl super::Scope {
    pub(super) fn eval_expr(&self, expr: &Expr) -> ValueResult {
        match &expr.kind {
            ExprKind::Literal(literal) => self.eval_literal(literal, expr.span),
            ExprKind::Not(body) => Ok(Value::new(
                ValueKind::Boolean(!self.eval_expr(body)?.is_truthy()),
                expr.span,
            )),
            ExprKind::Parenthesized(inner_expr) => {
                let value = self.eval_expr(inner_expr)?;
                Ok(Value::new(value.kind, expr.span))
            }
            ExprKind::Identifier(ident) => self.force(self.fetch_var(ident).ok_or(Error::new(
                ErrorKind::VariableNotInScope {
                    variable: expr.span,
                },
                (*self.0.source).clone(),
                expr.span,
            ))?),
            ExprKind::ArrayIndex {
                base,
                index: index_expr,
            } => {
                let base = self.eval_expr(base)?;
                let index = match self.eval_expr(index_expr)?.kind {
                    ValueKind::Int(index) => index as usize,
                    invalid => {
                        return Err(Error::new(
                            ErrorKind::InvalidIndex(invalid.type_of().to_owned()),
                            (*self.0.source).clone(),
                            index_expr.span,
                        ));
                    }
                };
                let item = base.try_index(index);

                match item {
                    Ok(item) => self.force(item.clone()),
                    Err(len) => Err(Error::new(
                        ErrorKind::IndexOutOfBounds {
                            length: len,
                            index: index_expr.span,
                        },
                        (*self.0.source).clone(),
                        expr.span,
                    )),
                }
            }
            ExprKind::MemberAccess { base, field } => {
                let base = self.eval_expr(base)?;
                self.force(base.access(field))
            }
            ExprKind::BinaryOp {
                left,
                operator,
                right,
            } => Ok(self.eval_binary_op(left, operator, right)?),
            ExprKind::Function {
                args,
                ret_ty,
                expr: body,
            } => Ok(Value::new(
                ValueKind::Function {
                    def_scope: Box::new(Scope(self.0.clone())),
                    args: args.clone(),
                    ret_ty: *ret_ty.clone(),
                    expr: *body.clone(),
                },
                expr.span,
            )),
            ExprKind::Call { .. } => self.eval_call(expr),
            ExprKind::With { object, expr: body } => {
                let child_scope = self.create_scope(*body.clone());

                let object = child_scope.eval_expr(object)?;

                let ValueKind::Object(object) = object.kind else {
                    return Err(Error::new(
                        ErrorKind::NonObjectInWithExpr,
                        (*self.0.source).clone(),
                        object.span,
                    ));
                };

                for (key, value) in object {
                    child_scope.define(key, value);
                }

                child_scope.eval()
            }
        }
    }

    pub(super) fn eval_literal(&self, literal: &Literal, span: SourceSpan) -> ValueResult {
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
                let scope = self.create_scope(Expr::default());
                let mut values = IndexMap::new();

                for (key, expr) in v {
                    let value = scope.eval_expr(expr)?;

                    let path = scope.extract_path(key)?;
                    if let Some(first) = path.first() {
                        if path.len() == 1 {
                            scope.define(first.clone(), value.clone());
                            values.insert(first.clone(), value);
                        } else if let Some(rest) = path.get(1..) {
                            let obj = create_nested_object(rest, value);
                            scope.define(first.clone(), obj.clone());
                            values.insert(first.clone(), obj);
                        }
                    }
                }

                Ok(Value::new(ValueKind::Object(values), span))
            }
        }
    }

    pub(super) fn extract_path(&self, expr: &Expr) -> Result<Vec<String>, Error> {
        match &expr.kind {
            ExprKind::Identifier(ident) | ExprKind::Literal(Literal::String(ident)) => {
                Ok(vec![ident.clone()])
            }
            ExprKind::Literal(Literal::InterpolatedString(v)) => {
                let mut value = String::new();
                for expr in v {
                    let expr_val = self.eval_expr(expr)?;
                    value.push_str(&expr_val.to_string());
                }
                Ok(vec![value])
            }
            ExprKind::MemberAccess { base, field } => {
                let mut path = self.extract_path(base)?;
                path.push(field.clone());
                Ok(path)
            }
            _ => Err(Error::new(
                ErrorKind::MismatchedTypes {
                    expected: "identifier or member access".to_owned(),
                    got: format!("{:?}", expr.kind),
                },
                (*self.0.source).clone(),
                expr.span,
            )),
        }
    }
}
