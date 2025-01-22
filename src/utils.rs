use crate::{parser::parse, runtime::types::Value, runtime::Scope, Source};
use logger::Log;

/// Evaluate a source script and return the result as a deserialized value.
/// # Errors
/// This function will return an error if either an evaluation error occurs or a deserialization error occurs.
#[cfg(feature = "serde")]
pub fn eval<T>(source: impl Into<Source>) -> Result<Option<T>, Box<Log>>
where
    T: for<'de> serde::Deserialize<'de>,
{
    use logger::make_fatal;
    use serde::Deserialize;

    let source = source.into();

    let ast = parse(&source).map_err(|err| Log::from(*err))?;
    let mut runtime = Scope::new(source, ast);

    match runtime.eval() {
        Ok(value) if value != Value::Null => {
            Ok(Some(Deserialize::deserialize(value).map_err(|err| {
                Box::new(make_fatal!(format!("Could not deserialize value: {err}")))
            })?))
        }
        Ok(_) => Ok(None),
        Err(err) => Err(Box::new(Log::from(*err))),
    }
}

/// Evaluate a source script.
/// # Errors
/// This function will return an error if either an evaluation error occurs.
#[cfg(feature = "serde")]
pub fn eval_untyped(source: impl Into<Source>) -> Result<Option<Value>, Box<Log>> {
    let source = source.into();

    let ast = parse(&source).map_err(|err| Log::from(*err))?;
    let mut runtime = Scope::new(source, ast);

    runtime
        .eval()
        .map(|value| {
            if value == Value::Null {
                None
            } else {
                Some(value)
            }
        })
        .map_err(|err| Box::new(Log::from(*err)))
}

/// Evaluate a source script.
/// # Errors
/// This function will return an error if either an evaluation error occurs.
#[cfg(not(feature = "serde"))]
pub fn eval(source: impl Into<Source>) -> Result<Option<Value>, Box<Log>> {
    let source = source.into();

    let ast = parse(&source).map_err(|err| Log::from(*err))?;
    let mut runtime = Scope::new(source, ast);

    runtime
        .eval()
        .map(|value| {
            if value == Value::Null {
                None
            } else {
                Some(value)
            }
        })
        .map_err(|err| Box::new(Log::from(*err)))
}
