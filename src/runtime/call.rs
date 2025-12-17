use crate::{
    parser::ast::types::{Expr, ExprKind},
    runtime::{
        Builtin, Error, ErrorKind, Scope, Value, ValueKind,
        types::{NativeFnCtx, ValueResult},
    },
};
use std::collections::HashMap;

impl super::Scope {
    pub(super) fn eval_call(&self, expr: &Expr) -> ValueResult {
        let ExprKind::Call { base, args } = &expr.kind else {
            unreachable!()
        };

        let function = self.eval_expr(base)?;
        let name = base.as_ident().unwrap_or("<unknown name>".into());

        match function.kind {
            ValueKind::Function {
                ref def_scope,
                args: ref params,
                ref ret_ty,
                ref expr,
            } => {
                if params.len() != args.len() {
                    return Err(Error::new(
                        ErrorKind::ArgsMismatch {
                            len: params.len(),
                            args: expr.span,
                        },
                        (*self.0.source).clone(),
                        expr.span,
                    ));
                }

                let mut evaluated_args = Vec::new();
                for arg_expr in args {
                    // TODO: Type check this against `params`
                    evaluated_args.push(self.eval_expr(arg_expr)?);
                }

                let mut variables: HashMap<String, Value> = HashMap::new();
                for (param, arg_value) in params.iter().zip(evaluated_args) {
                    variables.insert(param.clone().name, arg_value);
                }
                variables.extend(def_scope.0.local_variables.borrow().clone());

                let scope = Scope::new(variables, (*def_scope.0.source).clone(), expr.clone());
                scope.define(name.clone(), function.clone());
                scope.eval()
            }
            ValueKind::Builtin(Builtin(builtin)) => {
                let ctx = NativeFnCtx {
                    call_site: self.clone(),
                    expr: expr.clone(),
                };

                builtin(ctx)
            }
            _ => Err(Error::new(
                ErrorKind::NotCallable,
                (*self.0.source).clone(),
                expr.span,
            )),
        }
    }
}
