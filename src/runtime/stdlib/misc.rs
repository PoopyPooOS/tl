use crate::runtime::{Value, ValueKind, types::builtin};
use std::collections::HashMap;

pub(super) fn misc(map: &mut HashMap<String, Value>) {
    map.insert(
        "dbg".to_owned(),
        builtin(|ctx| {
            let msg = ctx.get_arg_evaluated(0, 2)?;

            // TODO: Switch to `log` crate
            println!("{msg}");

            ctx.get_arg_evaluated(1, 2)
        }),
    );

    map.insert(
        "len".to_owned(),
        builtin(|ctx| {
            let value = ctx.get_arg_evaluated(0, 1)?;

            Ok(match value.kind {
                ValueKind::String(v) => {
                    Value::new(ValueKind::Int(v.chars().count() as isize), ctx.expr.span)
                }
                ValueKind::Array(v) => Value::new(ValueKind::Int(v.len() as isize), ctx.expr.span),
                _ => Value::new(ValueKind::Null, ctx.expr.span),
            })
        }),
    );
}
