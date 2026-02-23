use crate::{
    parser::ast::types::{
        FnArg,
        expr::{Expr, ExprKind},
    },
    runtime::{Error, ErrorKind, Scope, Value, ValueKind, types::ValueResult},
};
use std::{collections::HashMap, fmt::Debug, path::PathBuf, rc::Rc};

pub trait Callable: Debug + Clone {
    fn call(&self, ctx: NativeFnCtx) -> ValueResult;

    fn params(&self) -> Option<&[FnArg]>;

    fn return_type(&self) -> Option<&Expr>;

    fn is_builtin(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct Function {
    pub params: Vec<FnArg>,
    pub return_type: Expr,
    pub body: Expr,
    pub closure_scope: Box<Scope>,
}

impl Callable for Function {
    fn call(&self, ctx: NativeFnCtx) -> ValueResult {
        // Arity check
        let ExprKind::Call { args, .. } = &ctx.expr.kind else {
            unreachable!()
        };

        if self.params.len() != args.len() {
            return Err(Error::new(
                ErrorKind::ArgsMismatch {
                    len: self.params.len(),
                    args: ctx.expr.span,
                },
                (*ctx.call_site.0.source).clone(),
                ctx.expr.span,
            ));
        }

        // Evaluate arguments and type check
        let mut evaluated_args = Vec::new();
        for (arg_expr, param) in args.iter().zip(&self.params) {
            let value = ctx.call_site.eval_expr(arg_expr)?;
            ctx.call_site.type_check(&value, param.ty.clone())?;
            evaluated_args.push(value);
        }

        // Create function scope with parameter bindings
        let mut variables: HashMap<String, Value> = HashMap::new();
        for (param, arg_value) in self.params.iter().zip(evaluated_args) {
            variables.insert(param.name.clone(), arg_value);
        }
        variables.extend(self.closure_scope.0.local_variables.borrow().clone());

        let fn_scope = Scope::new(
            variables,
            (*self.closure_scope.0.source).clone(),
            self.body.clone(),
        );

        // Define recursive reference
        let fn_name = ctx.expr.as_ident().unwrap_or("<lambda>".into());
        let fn_value = Value::new(
            ValueKind::Function(Function {
                params: self.params.clone(),
                return_type: self.return_type.clone(),
                body: self.body.clone(),
                closure_scope: self.closure_scope.clone(),
            }),
            ctx.expr.span,
        );
        fn_scope.define(fn_name, fn_value);

        // Evaluate and type check return value
        let mut value = fn_scope.eval()?;
        ctx.call_site.type_check(&value, self.return_type.clone())?;
        value.span = ctx.expr.span;
        Ok(value)
    }

    fn params(&self) -> Option<&[FnArg]> {
        Some(&self.params)
    }

    fn return_type(&self) -> Option<&Expr> {
        Some(&self.return_type)
    }

    fn is_builtin(&self) -> bool {
        false
    }
}

#[derive(Clone)]
pub struct Builtin(pub Rc<dyn Fn(NativeFnCtx) -> ValueResult>);

impl Debug for Builtin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Builtin")
            .field(&"<native function>")
            .finish()
    }
}

impl Callable for Builtin {
    fn call(&self, ctx: NativeFnCtx) -> ValueResult {
        (self.0)(ctx)
    }

    fn params(&self) -> Option<&[FnArg]> {
        None
    }

    fn return_type(&self) -> Option<&Expr> {
        None
    }

    fn is_builtin(&self) -> bool {
        true
    }
}

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

    pub fn get_arg_evaluated(
        &self,
        index: usize,
        expected_len: usize,
    ) -> Result<crate::runtime::Value, Error> {
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

    pub fn call_args_span(&self) -> miette::SourceSpan {
        let ExprKind::Call { ref args, .. } = self.expr.kind else {
            unreachable!()
        };

        let mut args_spans = args.iter().map(|arg| arg.span);

        if let Some(start) = args_spans.next()
            && let Some(end) = args_spans.next_back()
        {
            miette::SourceSpan::new(start.offset().into(), end.len())
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
            ValueKind::Bool(v) => Ok(ExtractedValue {
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
    ) -> Result<ExtractedValue<indexmap::IndexMap<String, Value>>, Error> {
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

    pub fn ensure_is_function(&self, value: Value) -> Result<ExtractedValue<Function>, Error> {
        match value.kind {
            ValueKind::Function(f) => Ok(ExtractedValue {
                data: f,
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

#[derive(Debug, Clone)]
pub struct ExtractedValue<T> {
    pub data: T,
    pub span: miette::SourceSpan,
}

pub fn builtin(function: impl Fn(NativeFnCtx) -> ValueResult + 'static) -> Value {
    use std::rc::Rc;
    Value::new_builtin(ValueKind::Builtin(Builtin(Rc::new(function))))
}
