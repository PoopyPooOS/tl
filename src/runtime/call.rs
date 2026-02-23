use crate::{
    parser::ast::types::{Expr, ExprKind},
    runtime::{
        Error, ErrorKind, ValueKind,
        types::{NativeFnCtx, ValueResult, function::Callable},
    },
};

impl super::Scope {
    pub(super) fn eval_call(&self, expr: &Expr) -> ValueResult {
        let ExprKind::Call { base, args: _ } = &expr.kind else {
            unreachable!()
        };

        let function = self.eval_expr(base)?;

        let ctx = NativeFnCtx {
            call_site: self.clone(),
            expr: expr.clone(),
        };

        match function.kind {
            ValueKind::Function(f) => f.call(ctx),
            ValueKind::Builtin(f) => f.call(ctx),
            _ => Err(Error::new(
                ErrorKind::NotCallable,
                (*self.0.source).clone(),
                expr.span,
            )),
        }
    }
}
