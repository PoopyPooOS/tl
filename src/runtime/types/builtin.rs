use crate::{
    parser::ast::types::{
        FnArg,
        expr::{Expr, ExprKind},
    },
    runtime::{Error, ErrorKind, Scope, Value, ValueKind, types::value::ValueResult},
};
use indexmap::IndexMap;
use miette::SourceSpan;
use std::{
    fmt::{Debug, Formatter},
    path::PathBuf,
    rc::Rc,
};

#[derive(Debug, Clone)]
pub struct ExtractedValue<T> {
    pub data: T,
    pub span: SourceSpan,
}

#[derive(Clone)]
pub struct Builtin(pub NativeFn);

pub type NativeFn = Rc<dyn Fn(NativeFnCtx) -> ValueResult>;

pub struct NativeFnCtx {
    pub call_site: Scope,
    pub expr: Expr,
}

impl NativeFnCtx {
    pub fn new_scope(&self) -> Scope {
        self.call_site.create_scope(self.expr.clone())
    }

    pub fn get_arg(&self, index: usize, expected_len: usize) -> Result<Expr, Error> {
        let ExprKind::Call { args, .. } = &self.expr.kind else {
            unreachable!()
        };

        let arg = args.get(index).ok_or(Error::new(
            ErrorKind::ArgsMismatch {
                len: expected_len,
                args: self.call_args_span(),
            },
            (*self.call_site.0.source).clone(),
            self.expr.span,
        ))?;

        Ok(arg.clone())
    }

    pub fn get_arg_evaluated(&self, index: usize, expected_len: usize) -> ValueResult {
        let ExprKind::Call { args, .. } = &self.expr.kind else {
            unreachable!()
        };

        let arg = args.get(index).ok_or(Error::new(
            ErrorKind::ArgsMismatch {
                len: expected_len,
                args: self.call_args_span(),
            },
            (*self.call_site.0.source).clone(),
            self.expr.span,
        ))?;

        self.eval_expr(arg.clone())
    }

    pub fn eval_expr(&self, expr: Expr) -> ValueResult {
        let scope = self.call_site.create_scope(self.expr.clone());
        scope.eval_expr(&expr)
    }

    pub fn expr_args(&self) -> Vec<Expr> {
        let ExprKind::Call { ref args, .. } = self.expr.kind else {
            unreachable!()
        };

        args.clone()
    }

    pub fn expr_args_evaluated(&self) -> Vec<ValueResult> {
        let scope = self.call_site.create_scope(self.expr.clone());
        let args = self.expr_args();

        args.iter().map(|arg| scope.eval_expr(arg)).collect()
    }

    pub fn call_args_span(&self) -> SourceSpan {
        let ExprKind::Call { ref args, .. } = self.expr.kind else {
            unreachable!()
        };

        let mut args_spans = args.iter().map(|arg| arg.span);

        if let Some(start) = args_spans.next()
            && let Some(end) = args_spans.next_back()
        {
            SourceSpan::new(start.offset().into(), end.len())
        } else {
            self.expr.span
        }
    }

    pub fn ensure_is_null(&self, value: Value) -> ValueResult {
        match value.kind {
            ValueKind::Null => Ok(value),
            _ => Err(Error::new(
                ErrorKind::MismatchedTypes {
                    expected: "null".to_owned(),
                    got: value.type_of().into(),
                },
                (*self.call_site.0.source).clone(),
                self.expr.span,
            )),
        }
    }

    pub fn ensure_is_boolean(&self, value: Value) -> Result<ExtractedValue<bool>, Error> {
        match value.kind {
            ValueKind::Boolean(v) => Ok(ExtractedValue {
                data: v,
                span: value.span,
            }),
            _ => Err(Error::new(
                ErrorKind::MismatchedTypes {
                    expected: "boolean".to_owned(),
                    got: value.type_of().into(),
                },
                (*self.call_site.0.source).clone(),
                self.expr.span,
            )),
        }
    }

    pub fn ensure_is_int(&self, value: Value) -> Result<ExtractedValue<isize>, Error> {
        match value.kind {
            ValueKind::Int(v) => Ok(ExtractedValue {
                data: v,
                span: value.span,
            }),
            _ => Err(Error::new(
                ErrorKind::MismatchedTypes {
                    expected: "int".to_owned(),
                    got: value.type_of().into(),
                },
                (*self.call_site.0.source).clone(),
                self.expr.span,
            )),
        }
    }

    pub fn ensure_is_float(&self, value: Value) -> Result<ExtractedValue<f64>, Error> {
        match value.kind {
            ValueKind::Float(v) => Ok(ExtractedValue {
                data: v,
                span: value.span,
            }),
            _ => Err(Error::new(
                ErrorKind::MismatchedTypes {
                    expected: "float".to_owned(),
                    got: value.type_of().into(),
                },
                (*self.call_site.0.source).clone(),
                self.expr.span,
            )),
        }
    }

    pub fn ensure_is_string(&self, value: Value) -> Result<ExtractedValue<String>, Error> {
        match value.kind {
            ValueKind::String(v) => Ok(ExtractedValue {
                data: v,
                span: value.span,
            }),
            _ => Err(Error::new(
                ErrorKind::MismatchedTypes {
                    expected: "string".to_owned(),
                    got: value.type_of().into(),
                },
                (*self.call_site.0.source).clone(),
                self.expr.span,
            )),
        }
    }

    pub fn ensure_is_path(&self, value: Value) -> Result<ExtractedValue<PathBuf>, Error> {
        match value.kind {
            ValueKind::Path(v) => Ok(ExtractedValue {
                data: v,
                span: value.span,
            }),
            _ => Err(Error::new(
                ErrorKind::MismatchedTypes {
                    expected: "path".to_owned(),
                    got: value.type_of().into(),
                },
                (*self.call_site.0.source).clone(),
                self.expr.span,
            )),
        }
    }

    pub fn ensure_is_array(&self, value: Value) -> Result<ExtractedValue<Vec<Value>>, Error> {
        match value.kind {
            ValueKind::Array(v) => Ok(ExtractedValue {
                data: v,
                span: value.span,
            }),
            _ => Err(Error::new(
                ErrorKind::MismatchedTypes {
                    expected: "array".to_owned(),
                    got: value.type_of().into(),
                },
                (*self.call_site.0.source).clone(),
                self.expr.span,
            )),
        }
    }

    pub fn ensure_is_object(
        &self,
        value: Value,
    ) -> Result<ExtractedValue<IndexMap<String, Value>>, Error> {
        match value.kind {
            ValueKind::Object(v) => Ok(ExtractedValue {
                data: v,
                span: value.span,
            }),
            _ => Err(Error::new(
                ErrorKind::MismatchedTypes {
                    expected: "object".to_owned(),
                    got: value.type_of().into(),
                },
                (*self.call_site.0.source).clone(),
                self.expr.span,
            )),
        }
    }

    pub fn ensure_is_function(
        &self,
        value: Value,
    ) -> Result<ExtractedValue<(Box<Scope>, Vec<FnArg>, Expr)>, Error> {
        match value.kind {
            ValueKind::Function {
                def_scope,
                args,
                ret_ty,
                expr,
            } => Ok(ExtractedValue {
                data: (def_scope, args, expr),
                span: value.span,
            }),
            _ => Err(Error::new(
                ErrorKind::MismatchedTypes {
                    expected: "function".to_owned(),
                    got: value.type_of().into(),
                },
                (*self.call_site.0.source).clone(),
                self.expr.span,
            )),
        }
    }

    pub fn ensure_is_builtin(&self, value: Value) -> Result<ExtractedValue<Builtin>, Error> {
        match value.kind {
            ValueKind::Builtin(v) => Ok(ExtractedValue {
                data: v,
                span: value.span,
            }),
            _ => Err(Error::new(
                ErrorKind::MismatchedTypes {
                    expected: "builtin".to_owned(),
                    got: value.type_of().into(),
                },
                (*self.call_site.0.source).clone(),
                self.expr.span,
            )),
        }
    }
}

impl Debug for Builtin {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Builtin")
            .field(&"<native function>")
            .finish()
    }
}

/// Shorthand to construct a builtin
pub fn builtin(function: impl Fn(NativeFnCtx) -> ValueResult + 'static) -> Value {
    Value::new_builtin(ValueKind::Builtin(Builtin(Rc::new(function))))
}
