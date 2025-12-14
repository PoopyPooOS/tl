use crate::{
    Source,
    parser::parse,
    runtime::{
        Error, Scope, Value, ValueKind,
        extension::{tl_ext_init, tl_ext_name},
        types::builtin,
    },
};
use indexmap::IndexMap;
use libloading::{Library, Symbol};
use std::{
    collections::HashMap,
    ffi::{CStr, OsStr},
};

pub(super) fn import(map: &mut HashMap<String, Value>) {
    map.insert(
        "import".to_owned(),
        builtin(move |ctx| {
            let (path, path_span) = {
                let path = ctx.ensure_is_path(ctx.get_arg_evaluated(0, 1)?)?;
                (path.data, path.span)
            };

            if let Some("so") = path.extension().and_then(OsStr::to_str) {
                let map = |err: libloading::Error| {
                    Error::new(err.into(), (*ctx.call_site.0.source).clone(), ctx.expr.span)
                };

                let ext = unsafe { Library::new(path) }.map_err(map)?;

                let name = unsafe {
                    CStr::from_ptr(ext.get::<tl_ext_name>("tl_ext_name").map_err(map)?())
                        .to_string_lossy()
                        .to_string()
                };

                let init: Symbol<tl_ext_init> = unsafe { ext.get(b"tl_ext_init").map_err(map)? };

                let res = unsafe { init(&mut ctx.call_site.clone()) };
                let map = Into::<Result<_, _>>::into(res)?;

                std::mem::forget(ext);

                Ok(Value::new(
                    ValueKind::Object(IndexMap::from([(name, map)])),
                    ctx.expr.span,
                ))
            } else {
                let source = Source::path(path).map_err(|err| {
                    Error::new(err.into(), (*ctx.call_site.0.source).clone(), path_span)
                })?;

                let ast = parse(&source).map_err(|err| {
                    let span = err.span;
                    let source = err.source.clone();
                    Error::new(err.into(), source, span)
                })?;

                Scope::new(HashMap::new(), source, ast).eval()
            }
        }),
    );
}
