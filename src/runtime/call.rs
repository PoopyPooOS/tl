use super::{
    ValueResult,
    types::{Builtin, Error, ErrorKind},
};
use crate::{
    merge_spans,
    parser::ast::types::{Expr, ExprKind},
    runtime::{ValueKind, types::NativeFnCtx},
};

impl super::Scope {
    pub(super) fn eval_call(&mut self, expr: &Expr) -> ValueResult {
        let ExprKind::Call { base, args } = &expr.kind else {
            unreachable!()
        };

        let function = self.eval_expr(base)?;
        let name = base.as_ident().unwrap_or("<unknown name>".into());

        match function.kind {
            ValueKind::Function {
                args: ref parameters,
                expr: ref body,
            } => {
                let mut evaluated_args = Vec::with_capacity(args.len());
                for expr in args {
                    evaluated_args.push(self.eval_expr(expr)?);
                }

                if args.len() != parameters.len() {
                    let args = if let Some(first) = args.iter().next()
                        && let Some(last) = args.iter().last()
                    {
                        merge_spans(first.span, last.span)
                    } else {
                        base.span
                    };

                    return Err(Error::new(
                        ErrorKind::ArgsMismatch {
                            len: parameters.len(),
                            args,
                        },
                        self.source.clone(),
                        expr.span,
                    ));
                }

                let scope = self.create_scope(body.clone());

                for (param, arg) in parameters.iter().zip(evaluated_args) {
                    scope.define(param, arg);
                }

                scope.define(&name, function);

                scope.eval()
            }
            ValueKind::Builtin(Builtin(builtin)) => {
                let ctx = NativeFnCtx {
                    expr: expr.clone(),
                    variables: self.variables.clone(),
                    source: self.source.clone(),
                };

                builtin(ctx)
            }
            _ => unreachable!("`function` was filtered before to only match for functions"),
        }
    }
}
