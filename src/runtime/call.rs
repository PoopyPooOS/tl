use std::collections::HashMap;

use super::{ValueResult, types::Builtin};
use crate::{
    parser::ast::types::{Expr, ExprKind},
    runtime::{Error, ErrorKind, Scope, Value, ValueKind, types::NativeFnCtx},
};

impl super::Scope {
    pub(super) fn eval_call(&mut self, expr: &Expr) -> ValueResult {
        let ExprKind::Call { base, args } = &expr.kind else {
            unreachable!()
        };

        let mut function = self.eval_expr(base)?;
        let name = base.as_ident().unwrap_or("<unknown name>".into());

        let mut variables: HashMap<String, Value> = self.variables.clone();

        for arg_expr in args {
            let arg_value = self.eval_expr(arg_expr)?;

            function = match function.kind {
                ValueKind::Function {
                    ref def_scope,
                    arg: ref param,
                    expr: ref body,
                } => {
                    variables.insert(param.to_owned(), arg_value.clone());

                    variables.extend(def_scope.variables.clone());
                    let mut scope =
                        Scope::new(variables.clone(), def_scope.source.clone(), body.clone());
                    scope.define(&name, function.clone());
                    scope.eval()?
                }
                ValueKind::Builtin(Builtin(builtin)) => {
                    let ctx = NativeFnCtx {
                        expr: expr.clone(),
                        variables: self.variables.clone(),
                        source: self.source.clone(),
                    };

                    return builtin(ctx);
                }
                _ => {
                    return Err(Error::new(
                        ErrorKind::NotCallable,
                        self.source.clone(),
                        expr.span,
                    ));
                }
            };
        }

        Ok(function)
    }
}
