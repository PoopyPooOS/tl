use crate::{
    Source,
    parser::parse,
    runtime::{Error, Scope, Value, types::builtin},
};
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
        "import".to_owned(),
        builtin(move |ctx| {
            let (path, path_span) = {
                let path = ctx.ensure_is_path(ctx.get_arg_evaluated(0, 1)?)?;
                (path.data, path.span)
            };

            let source = Source::path(path)
                .map_err(|err| Error::new(err.into(), (*ctx.source).clone(), path_span))?;

            let ast = parse(&source).map_err(|err| {
                let span = err.span;
                let source = err.source.clone();
                Error::new(err.into(), source, span)
            })?;

            Scope::new(HashMap::new(), source, ast).eval()
        }),
    );
}
