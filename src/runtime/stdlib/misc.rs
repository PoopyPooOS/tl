use crate::runtime::{Value, types::builtin};
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
}
