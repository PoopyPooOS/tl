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

        let mut function = self.eval_expr(base)?;
        let name = base.as_ident().unwrap_or("<unknown name>".into());

        let mut variables: HashMap<String, Value> = HashMap::new();

        for arg_expr in args {
            function = match function.kind {
                ValueKind::Function {
                    ref def_scope,
                    arg: ref param,
                    expr: ref body,
                } => {
                    let arg_value = self.eval_expr(arg_expr)?;

                    variables.insert(param.to_owned(), arg_value.clone());

                    variables.extend(def_scope.0.local_variables.borrow().clone());
                    let scope = Scope::new(
                        variables.clone(),
                        (*def_scope.0.source).clone(),
                        body.clone(),
                    );
                    scope.define(name.clone(), function.clone());
                    scope.eval()?
                }
                ValueKind::Builtin(Builtin(builtin)) => {
                    let ctx = NativeFnCtx {
                        call_site: self.clone(),
                        expr: expr.clone(),
                    };

                    return builtin(ctx);
                }
                _ => {
                    return Err(Error::new(
                        ErrorKind::NotCallable,
                        (*self.0.source).clone(),
                        expr.span,
                    ));
                }
            };
        }

        Ok(function)
    }
}
