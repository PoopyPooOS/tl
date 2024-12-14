use super::types::Value;
use logger::{error, warn};

impl super::Scope {
    #[allow(clippy::too_many_lines, reason = "This lint is stupid")]
    pub(super) fn init_stdlib(&mut self) {
        macro_rules! add_native_fn {
            ($name:ident, [$($param:ident),*], $body:block) => {
                self.functions.insert(
                    stringify!($name).to_string(),
                    Value::NativeFunction {
                        parameters: vec![$(stringify!($param).to_string()),*],
                        body: |args| {
                            // Automatically bind arguments to variables
                            let mut iter = args.iter();
                            $(
                                let $param = iter.next().expect(concat!("Expected ", stringify!($param)));
                            )*
                            // Insert the function body
                            $body
                        },
                    },
                )
            };
            ($name:ident, [$($param:ident?),*], $body:block) => {
                self.functions.insert(
                    stringify!($name).to_string(),
                    Value::NativeFunction {
                        parameters: vec![$(stringify!($param).to_string()),*],
                        body: |args| {
                            // Automatically bind arguments to variables
                            let mut iter = args.iter();
                            $(
                                let $param = iter.next().unwrap_or(&Value::Null);
                            )*
                            // Insert the function body
                            $body
                        },
                    },
                )
            };
        }

        #[allow(unused_macros, reason = "Will be used... maybe... some day")]
        macro_rules! add_fn {
            ($name:ident, [$($param:ident),*], $body:expr) => {
                self.functions.insert(
                    stringify!($name).to_string(),
                    Value::Function {
                        parameters: vec![$(stringify!($param).to_string()),*],
                        body: $body,
                    },
                )
            };
        }

        // Output
        add_native_fn!(print, [value], {
            if let Value::String(value) = value
                && value.ends_with('\n')
            {
                warn!("using `print()` with a string that ends in a newline", hint: "Use `println()` instead");
            }

            print!("{value}");
            None
        });
        add_native_fn!(println, [value], {
            println!("{value}");
            None
        });
        add_native_fn!(error, [value], {
            error!(format!("{value}"));
            None
        });

        // Objects
        add_native_fn!(objectKeys, [object], {
            if let Value::Object(object) = object {
                Some(Value::Array(object.keys().map(|key| Value::String(key.to_string())).collect()))
            } else {
                None
            }
        });
        add_native_fn!(objectValues, [object], {
            if let Value::Object(object) = object {
                Some(Value::Array(object.values().cloned().collect()))
            } else {
                None
            }
        });
        add_native_fn!(objectMerge, [object1, object2], {
            if let (Value::Object(object1), Value::Object(object2)) = (object1, object2) {
                Some(Value::Object(
                    object1
                        .iter()
                        .chain(object2.iter())
                        .map(|(key, value)| (key.clone(), value.clone()))
                        .collect(),
                ))
            } else {
                None
            }
        });
        add_native_fn!(objectGet, [object, key], {
            if let (Value::Object(object), Value::String(key)) = (object, key) {
                object.get(key).cloned()
            } else {
                None
            }
        });

        // Branching
        add_native_fn!(if, [condition, then_block, else_block], {
            if condition.is_truthy() {
                Some(then_block.clone())
            } else {
                Some(else_block.clone())
            }
        });

        // Other
        add_native_fn!(typeOf, [value?], { Some(Value::String(value.type_of().to_string())) });
        add_native_fn!(exit, [code?], {
            std::process::exit(match code {
                Value::Number(code) => i32::try_from(*code).unwrap_or(0),
                _ => 0,
            })
        });
    }
}
