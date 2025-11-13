use crate::{
    parser::parse,
    runtime::{Scope, types::Value},
};
use miette::{NamedSource, Report, SourceSpan};
use std::collections::HashMap;

/// Evaluate a source script and return the result as a deserialized value.
/// # Errors
/// This function will return an error if either an evaluation error occurs or a deserialization error occurs.
#[cfg(feature = "serde")]
pub fn eval<T>(source: NamedSource<String>, scope_setup: impl Fn(&mut Scope)) -> Result<T, Report>
where
    T: for<'de> serde::Deserialize<'de>,
{
    use serde::Deserialize;

    let ast = parse(&source)?;
    let mut scope = Scope::new(HashMap::new(), source.clone(), ast);

    scope_setup(&mut scope);

    match scope.eval() {
        Ok(value) => Ok(Deserialize::deserialize(value).map_err(|err| {
            Report::msg(format!("Could not deserialize value: {err}")).with_source_code(source)
        })?),
        Err(err) => Err(err.into()),
    }
}

/// Evaluate a source script.
/// # Errors
/// This function will return an error if either an evaluation error occurs.
#[cfg(feature = "serde")]
pub fn eval_untyped(
    source: miette::NamedSource<String>,
    scope_setup: impl Fn(&mut Scope),
) -> Result<Value, Report> {
    let ast = parse(&source)?;
    let mut scope = Scope::new(HashMap::new(), source, ast);

    scope_setup(&mut scope);

    Ok(scope.eval()?)
}

/// Evaluate a source script.
/// # Errors
/// This function will return an error if either an evaluation error occurs.
#[cfg(not(feature = "serde"))]
pub fn eval(
    source: NamedSource<String>,
    scope_setup: impl Fn(&mut Scope),
) -> Result<Value, Report> {
    let ast = parse(&source)?;
    let mut scope = Scope::new(HashMap::new(), source, ast);

    scope_setup(&mut scope);

    Ok(runtime.eval()?)
}

#[allow(dead_code)]
pub(crate) fn span(start: usize, len: usize) -> SourceSpan {
    SourceSpan::new(start.into(), len)
}

pub(crate) fn merge_spans(start: SourceSpan, end: SourceSpan) -> SourceSpan {
    let offset = start.offset().into();
    let len = (end.offset().saturating_add(end.len())).saturating_sub(start.offset());
    SourceSpan::new(offset, len)
}
