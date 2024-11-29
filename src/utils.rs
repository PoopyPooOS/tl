use crate::{parser::parse, runtime, source::Source};
use logger::Log;

pub(crate) fn handle_string_escapes(original: impl Into<String>) -> String {
    let mut original: String = original.into();
    let replacements: &[(&str, &str)] = &[
        (r"\\", "\\"),
        ("\\\"", "\""),
        (r"\'", "'"),
        (r"\n", "\n"),
        (r"\r", "\r"),
        (r"\t", "\t"),
        (r"\0", "\0"),
    ];

    for replacement in replacements {
        original = original.replace(replacement.0, replacement.1);
    }

    original
}

/// Evaluate a source script and return the result as a deserialized value.
/// # Errors
/// This function will return an error if either an evaluation error occurs or a deserialization error occurs.
#[cfg(feature = "serde")]
pub fn eval<T>(value: impl Into<Source>) -> Result<Option<T>, Box<Log>>
where
    T: for<'de> serde::Deserialize<'de>,
{
    use logger::make_error;

    let ast = parse(value.into())?;
    let mut runtime = crate::runtime::Scope::new(ast);

    match runtime.eval() {
        Ok(Some(value)) => Ok(Some(
            serde::Deserialize::deserialize(value).map_err(|_| Box::new(make_error!("Could not deserialize value")))?,
        )),
        Ok(None) => Ok(None),
        Err(log) => Err(log),
    }
}

/// Evaluate a source script.
/// # Errors
/// This function will return an error if either an evaluation error occurs.
#[cfg(feature = "serde")]
pub fn eval_untyped(value: impl Into<Source>) -> Result<Option<runtime::types::Value>, Box<Log>> {
    let ast = parse(value.into())?;
    let mut runtime = runtime::Scope::new(ast);

    runtime.eval()
}

/// Evaluate a source script.
/// # Errors
/// This function will return an error if either an evaluation error occurs.
#[cfg(not(feature = "serde"))]
pub fn eval(value: impl Into<Source>) -> Result<Option<Value>, Box<Log>> {
    let ast = parse(value.into())?;
    let mut runtime = runtime::Scope::new(ast);

    runtime.eval()
}
