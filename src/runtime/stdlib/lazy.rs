use crate::runtime::{Value, ValueKind, types::builtin};
use std::collections::HashMap;

pub(super) fn lazy(map: &mut HashMap<String, Value>) {
    map.insert(
        "lazy".to_owned(),
        builtin(|ctx| {
            let expr = ctx.get_arg(0, 1)?;
            let span = ctx.expr.span;

            Ok(Value::new(
                ValueKind::Thunk {
                    def_scope: Box::new(ctx.call_site.clone()),
                    expr,
                },
                span,
            ))
        }),
    );
}
