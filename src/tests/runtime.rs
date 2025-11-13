#![allow(clippy::unwrap_used, reason = "Panics automatically invalidate tests")]

use crate::{
    parser::parse,
    runtime::{
        Scope, ValueKind,
        types::{Error as RuntimeError, ErrorKind as RuntimeErrorKind, Value},
    },
    span,
};
use miette::NamedSource;
use pretty_assertions::assert_eq;
use std::collections::{BTreeMap, HashMap};

fn run(text: impl Into<String>) -> miette::Result<Value> {
    let source = NamedSource::new("test", text.into());
    let ast = parse(&source)?;

    Ok(Scope::new(HashMap::new(), source, ast).eval()?)
}

/// Evaluate something expecting a runtime error.
fn run_err(text: impl Into<String>) -> RuntimeError {
    let source = NamedSource::new("test", text.into());
    let ast = parse(&source).unwrap();

    Scope::new(HashMap::new(), source, ast).eval().unwrap_err()
}

#[test]
fn boolean() {
    let input = "true";
    let expected = Value::new(ValueKind::Boolean(true), span(0, 4));
    assert_eq!(run(input).unwrap(), expected);

    let input = "false";
    let expected = Value::new(ValueKind::Boolean(false), span(0, 5));
    assert_eq!(run(input).unwrap(), expected);
}

#[test]
fn number() {
    let input = "42";
    let expected = Value::new(ValueKind::Int(42), span(0, 2));
    assert_eq!(run(input).unwrap(), expected);
}

#[allow(clippy::approx_constant)]
#[test]
fn float() {
    let input = "3.14";
    let expected = Value::new(ValueKind::Float(3.14), span(0, 4));
    assert_eq!(run(input).unwrap(), expected);
}

#[test]
fn string() {
    let input = "\"Hello, world!\"";
    let expected = Value::new(ValueKind::String("Hello, world!".into()), span(0, 15));
    assert_eq!(run(input).unwrap(), expected);
}

#[test]
fn escaped_string() {
    let input = "\"Hello, \\n\\tworld!\"";
    let expected = Value::new(ValueKind::String("Hello, \n\tworld!".into()), span(0, 19));
    assert_eq!(run(input).unwrap(), expected);
}

#[test]
fn interpolated_string() {
    let input = r#"let
    name = "John Doe"
in
    "Hello, my name is ${name}!"
"#;
    let expected = Value::new(
        ValueKind::String("Hello, my name is John Doe!".into()),
        span(33, 28),
    );
    assert_eq!(run(input).unwrap(), expected);
}

#[test]
fn array() {
    let input = "[ 1 2 3 ]";
    let expected = Value::new(
        ValueKind::Array(vec![
            Value::new(ValueKind::Int(1), span(2, 1)),
            Value::new(ValueKind::Int(2), span(4, 1)),
            Value::new(ValueKind::Int(3), span(6, 1)),
        ]),
        span(0, 9),
    );
    assert_eq!(run(input).unwrap(), expected);
}

#[test]
fn array_indexing() {
    let input = r"let
    numbers = [ 1 2 3 ]
in
    numbers[1]";
    let expected = Value::new(ValueKind::Int(2), span(22, 1));
    assert_eq!(run(input).unwrap(), expected);

    // Out of bounds index
    let input = r"let
    numbers = [ 1 2 3 ]
in
    numbers[3]";
    let expected = RuntimeError::new(
        RuntimeErrorKind::IndexOutOfBounds {
            length: 3,
            index: span(35, 10),
        },
        NamedSource::new("test", input.to_string()),
        span(35, 10),
    );
    assert_eq!(run_err(input), expected);
}

#[test]
fn object() {
    let input = "{ name = \"John Doe\" age = 42 }";
    let expected = Value::new(
        ValueKind::Object(BTreeMap::from([
            (
                "name".into(),
                Value::new(ValueKind::String("John Doe".into()), span(9, 10)),
            ),
            ("age".into(), Value::new(ValueKind::Int(42), span(26, 2))),
        ])),
        span(0, 30),
    );
    assert_eq!(run(input).unwrap(), expected);
}

#[test]
fn field_access() {
    let input = r#"let
    package = {
        dependencies = [ "other_package" ]
    }
in
    package.dependencies"#;
    let expected = Value::new(
        ValueKind::Array(vec![Value::new(
            ValueKind::String("other_package".into()),
            span(45, 15),
        )]),
        span(43, 19),
    );
    assert_eq!(run(input).unwrap(), expected);
}

#[test]
fn not() {
    let input = "!true";
    let expected = Value::new(ValueKind::Boolean(false), span(0, 5));
    assert_eq!(run(input).unwrap(), expected);
}

#[test]
fn function() {
    let input = r#"let
    greet = (name) {
        "Hello, ${name}!"
    }
in
    greet("John Doe")"#;
    let expected = Value::new(ValueKind::String("Hello, John Doe!".into()), span(33, 17));
    assert_eq!(run(input).unwrap(), expected);
}

#[test]
fn binary_op() {
    let input = "2 + 3 * 4";
    let expected = Value::new(ValueKind::Int(14), span(0, 9));
    assert_eq!(run(input).unwrap(), expected);
}

#[test]
fn bindings() {
    let input = "let name = \"John Doe\" in name";
    let expected = Value::new(ValueKind::String("John Doe".into()), span(11, 10));
    assert_eq!(run(input).unwrap(), expected);
}

#[test]
#[ignore = "Weird stack overflow bug that only happens in tests"]
fn recursion() {
    let input = r"let
    pow = (base, exponent) {
        if(
            exponent == 0,
            1,
            base * pow(base, exponent - 1)
        )
    }
in
    pow(2, 10)";
    let expected = Value::new(ValueKind::Int(1024), span(99, 99));
    assert_eq!(run(input).unwrap(), expected);
}
