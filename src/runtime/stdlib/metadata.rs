use crate::runtime::{Error, ErrorKind, Value, ValueKind, types::builtin};
use std::collections::HashMap;

pub(super) fn metadata(map: &mut HashMap<String, Value>) {
    map.insert(
        "get_metadata".to_owned(),
        builtin(|ctx| {
            let value = ctx.get_arg_evaluated(0, 1)?;

            // Build object with metadata fields
            let mut obj = indexmap::IndexMap::new();
            if let Some(doc) = value.metadata.doc {
                obj.insert(
                    "doc".to_owned(),
                    Value::new(ValueKind::String(doc.clone()), ctx.expr.span),
                );
            }

            Ok(Value::new(ValueKind::Object(obj), ctx.expr.span))
        }),
    );

    map.insert(
        "set_metadata".to_owned(),
        builtin(|ctx| {
            let metadata_obj = ctx.get_arg_evaluated(0, 2)?;
            let value = ctx.get_arg_evaluated(1, 2)?;

            let ValueKind::Object(obj) = &metadata_obj.kind else {
                return Err(Error::new(
                    ErrorKind::MismatchedTypes {
                        expected: "object".to_owned(),
                        got: metadata_obj.type_of().to_owned(),
                    },
                    (*ctx.call_site.0.source).clone(),
                    metadata_obj.span,
                ));
            };

            let mut result = value.clone();

            if let Some(doc_value) = obj.get("doc") {
                if let ValueKind::String(doc_str) = &doc_value.kind {
                    result.metadata.doc = Some(doc_str.to_owned());
                } else {
                    return Err(Error::new(
                        ErrorKind::MismatchedTypes {
                            expected: "string".to_owned(),
                            got: doc_value.type_of().to_owned(),
                        },
                        (*ctx.call_site.0.source).clone(),
                        doc_value.span,
                    ));
                }
            }

            Ok(result)
        }),
    );

    map.insert(
        "doc".to_owned(),
        builtin(|ctx| {
            let doc = ctx.ensure_is_string(ctx.get_arg_evaluated(0, 2)?)?.data;
            let value = ctx.get_arg_evaluated(1, 2)?;

            let mut result = value.clone();
            result.metadata.doc = Some(doc);
            Ok(result)
        }),
    );
}
