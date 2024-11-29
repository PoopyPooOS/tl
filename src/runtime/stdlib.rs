use super::types::Value;
use logger::warn;

impl super::Scope {
    pub(super) fn init_stdlib(&mut self) {
        self.functions.insert(
            "print".to_string(),
            Value::NativeFunction {
                parameters: vec!["value".to_string()],
                body: |args| {
                    let value = &args[0];
                    if let Value::String(value) = value
                        && value.ends_with('\n')
                    {
                        warn!("using `print()` with a string that ends in a newline", hint: "Use `println()` instead");
                    }

                    print!("{value}");
                    None
                },
            },
        );

        self.functions.insert(
            "println".to_string(),
            Value::NativeFunction {
                parameters: vec!["value".to_string()],
                body: |args| {
                    let value = &args[0];
                    println!("{value}");
                    None
                },
            },
        );
    }
}
