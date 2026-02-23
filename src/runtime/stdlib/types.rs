use crate::runtime::{Value, ValueKind, types::builtin};
use std::collections::HashMap;

pub(super) fn types(map: &mut HashMap<String, Value>) {
    map.insert(
        "any".to_owned(),
        builtin(|ctx| Ok(Value::new(ValueKind::Bool(true), ctx.expr.span))),
    );

    map.insert(
        "nothing".to_owned(),
        builtin(|ctx| {
            let value = ctx.get_arg_evaluated(0, 1)?;
            let is_null = matches!(value.kind, ValueKind::Null);
            Ok(Value::new(ValueKind::Bool(is_null), ctx.expr.span))
        }),
    );

    map.insert(
        "bool".to_owned(),
        builtin(|ctx| {
            let value = ctx.get_arg_evaluated(0, 1)?;
            let is_bool = matches!(value.kind, ValueKind::Bool(_));
            Ok(Value::new(ValueKind::Bool(is_bool), ctx.expr.span))
        }),
    );

    map.insert(
        "int".to_owned(),
        builtin(|ctx| {
            let value = ctx.get_arg_evaluated(0, 1)?;
            let is_int = matches!(value.kind, ValueKind::Int(_));
            Ok(Value::new(ValueKind::Bool(is_int), ctx.expr.span))
        }),
    );

    map.insert(
        "uint".to_owned(),
        builtin(|ctx| {
            let value = ctx.get_arg_evaluated(0, 1)?;
            let is_uint = matches!(value.kind, ValueKind::Int(n) if n >= 0);
            Ok(Value::new(ValueKind::Bool(is_uint), ctx.expr.span))
        }),
    );

    map.insert(
        "float".to_owned(),
        builtin(|ctx| {
            let value = ctx.get_arg_evaluated(0, 1)?;
            let is_float = matches!(value.kind, ValueKind::Float(_));
            Ok(Value::new(ValueKind::Bool(is_float), ctx.expr.span))
        }),
    );

    map.insert(
        "number".to_owned(),
        builtin(|ctx| {
            let value = ctx.get_arg_evaluated(0, 1)?;
            let is_number = matches!(value.kind, ValueKind::Int(_) | ValueKind::Float(_));
            Ok(Value::new(ValueKind::Bool(is_number), ctx.expr.span))
        }),
    );

    map.insert(
        "string".to_owned(),
        builtin(|ctx| {
            let value = ctx.get_arg_evaluated(0, 1)?;
            let is_string = matches!(value.kind, ValueKind::String(_));
            Ok(Value::new(ValueKind::Bool(is_string), ctx.expr.span))
        }),
    );

    map.insert(
        "path".to_owned(),
        builtin(|ctx| {
            let value = ctx.get_arg_evaluated(0, 1)?;
            let is_path = matches!(value.kind, ValueKind::Path(_));
            Ok(Value::new(ValueKind::Bool(is_path), ctx.expr.span))
        }),
    );

    map.insert(
        "array".to_owned(),
        builtin(|ctx| {
            let value = ctx.get_arg_evaluated(0, 1)?;
            let is_array = matches!(value.kind, ValueKind::Array(_));
            Ok(Value::new(ValueKind::Bool(is_array), ctx.expr.span))
        }),
    );

    map.insert(
        "object".to_owned(),
        builtin(|ctx| {
            let value = ctx.get_arg_evaluated(0, 1)?;
            let is_object = matches!(value.kind, ValueKind::Object(_));
            Ok(Value::new(ValueKind::Bool(is_object), ctx.expr.span))
        }),
    );

    map.insert(
        "fn".to_owned(),
        builtin(|ctx| {
            let value = ctx.get_arg_evaluated(0, 1)?;
            let is_function = matches!(value.kind, ValueKind::Function(_) | ValueKind::Builtin(_));
            Ok(Value::new(ValueKind::Bool(is_function), ctx.expr.span))
        }),
    );

    map.insert(
        "list".to_owned(),
        builtin(|ctx| {
            let value = ctx.get_arg_evaluated(0, 2)?;
            let is_array = matches!(value.kind, ValueKind::Array(_));
            Ok(Value::new(ValueKind::Bool(is_array), ctx.expr.span))
        }),
    );
}
