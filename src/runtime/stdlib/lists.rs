use crate::runtime::{Value, ValueKind, types::builtin};
use std::collections::HashMap;

pub(super) fn lists(map: &mut HashMap<String, Value>) {
    map.insert(
        "map".to_owned(),
        builtin(|ctx| {
            let args_len = 2;
            let list = ctx.ensure_is_array(ctx.get_arg_evaluated(0, args_len)?)?;
            let callback = ctx.ensure_is_function(ctx.get_arg_evaluated(1, args_len)?)?;
            let (_, arg, expr) = callback.data;

            let mut result = Vec::new();

            for item in &list.data {
                let scope = ctx.call_site.create_scope(expr.clone());
                scope.define(arg.clone(), item.clone());
                result.push(scope.eval()?);
            }

            Ok(Value::new(ValueKind::Array(result), ctx.expr.span))
        }),
    );

    map.insert(
        "any".to_owned(),
        builtin(|ctx| {
            let args_len = 2;
            let list = ctx.ensure_is_array(ctx.get_arg_evaluated(0, args_len)?)?;
            let callback = ctx.ensure_is_function(ctx.get_arg_evaluated(1, args_len)?)?;
            let (_, arg, expr) = callback.data;

            for item in &list.data {
                let scope = ctx.call_site.create_scope(expr.clone());
                scope.define(arg.clone(), item.clone());
                let res = scope.eval()?;
                if res.is_truthy() {
                    return Ok(Value::new(ValueKind::Boolean(true), ctx.expr.span));
                }
            }

            Ok(Value::new(ValueKind::Boolean(false), ctx.expr.span))
        }),
    );

    map.insert(
        "all".to_owned(),
        builtin(|ctx| {
            let args_len = 2;
            let list = ctx.ensure_is_array(ctx.get_arg_evaluated(0, args_len)?)?;
            let callback = ctx.ensure_is_function(ctx.get_arg_evaluated(1, args_len)?)?;
            let (_, arg, expr) = callback.data;

            for item in &list.data {
                let scope = ctx.call_site.create_scope(expr.clone());
                scope.define(arg.clone(), item.clone());
                let res = scope.eval()?;
                if !res.is_truthy() {
                    return Ok(Value::new(ValueKind::Boolean(false), ctx.expr.span));
                }
            }

            Ok(Value::new(ValueKind::Boolean(true), ctx.expr.span))
        }),
    );

    map.insert(
        "filter".to_owned(),
        builtin(|ctx| {
            let args_len = 2;
            let list = ctx.ensure_is_array(ctx.get_arg_evaluated(0, args_len)?)?;
            let callback = ctx.ensure_is_function(ctx.get_arg_evaluated(1, args_len)?)?;
            let (_, arg, expr) = callback.data;

            let mut new_list = Vec::with_capacity(list.data.len());

            for item in &list.data {
                let scope = ctx.call_site.create_scope(expr.clone());
                scope.define(arg.clone(), item.clone());

                let res = scope.eval()?;
                if res.is_truthy() {
                    new_list.push(item.clone());
                }
            }

            Ok(Value::new(ValueKind::Array(new_list), ctx.expr.span))
        }),
    );

    // TODO: fold

    map.insert(
        "reverse".to_owned(),
        builtin(|ctx| {
            let mut list = ctx.ensure_is_array(ctx.get_arg_evaluated(0, 1)?)?;

            list.data.reverse();

            Ok(Value::new(ValueKind::Array(list.data), ctx.expr.span))
        }),
    );

    map.insert(
        "sort".to_owned(),
        builtin(|ctx| {
            let mut list = ctx.ensure_is_array(ctx.get_arg_evaluated(0, 1)?)?;

            list.data.sort();

            Ok(Value::new(ValueKind::Array(list.data), ctx.expr.span))
        }),
    );

    map.insert(
        "find".to_owned(),
        builtin(|ctx| {
            let args_len = 2;
            let list = ctx.ensure_is_array(ctx.get_arg_evaluated(0, args_len)?)?;
            let element = ctx.get_arg_evaluated(1, args_len)?;

            Ok(list
                .data
                .into_iter()
                .find(|x| *x == element)
                .unwrap_or(Value::new(ValueKind::Null, element.span)))
        }),
    );

    map.insert(
        "first".to_owned(),
        builtin(|ctx| {
            let list = ctx.ensure_is_array(ctx.get_arg_evaluated(0, 1)?)?;
            let first = list
                .data
                .first()
                .unwrap_or(&Value::new(ValueKind::Null, ctx.expr.span))
                .clone();

            Ok(first)
        }),
    );
    map.insert(
        "last".to_owned(),
        builtin(|ctx| {
            let list = ctx.ensure_is_array(ctx.get_arg_evaluated(0, 1)?)?;
            let first = list
                .data
                .last()
                .unwrap_or(&Value::new(ValueKind::Null, ctx.expr.span))
                .clone();

            Ok(first)
        }),
    );
}
