use crate::runtime::Value;
use std::collections::HashMap;

macro_rules! def_stdlib {
    ($($module:ident),* $(,)?) => {
        $(
            mod $module;
        )*

        pub fn stdlib() -> HashMap<String, Value> {
            let mut map = HashMap::new();
            $(
                $module::$module(&mut map);
            )*
            map
        }
    };
}

def_stdlib!(conditionals, misc);
