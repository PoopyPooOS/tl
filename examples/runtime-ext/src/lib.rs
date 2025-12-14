use tl::{extension, indexmap::IndexMap};

extension! {
    name: "greet",
    init: |_| {
        let mut map = IndexMap::new();

        map.insert(
            "greet".to_owned(),
            builtin(|ctx| {
                let name = ctx.ensure_is_string(ctx.get_arg_evaluated(0, 1)?)?;

                Ok(Value::new(
                    ValueKind::String(format!("Hello, {}!", name.data)),
                    name.span,
                ))
            }),
        );

        Ok(Value::new_builtin(ValueKind::Object(map)))
    }
}
