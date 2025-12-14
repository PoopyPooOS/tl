use crate::runtime::{Value, types::builtin};
use std::collections::HashMap;

pub(super) fn lists(map: &mut HashMap<String, Value>) {
    map.insert(
        "map".to_owned(),
        builtin(|ctx| {
            let args_len = 3;
            let cond = ctx.get_arg(0, args_len)?;
            let then_branch = ctx.get_arg(1, args_len)?;
            let else_branch = ctx.get_arg(2, args_len)?;

            let scope = ctx.new_scope();

            let cond = scope.eval_expr(&cond)?;

            if cond.is_truthy() {
                return scope.eval_expr(&then_branch);
            }

            scope.eval_expr(&else_branch)
        }),
    );
}
