#![allow(clippy::unwrap_used, reason = "Panics automatically invalidate tests")]

use crate::{
    Source,
    parser::parse,
    runtime::{
        Scope, ValueKind,
        types::{Error as RuntimeError, ErrorKind as RuntimeErrorKind, Value},
    },
    span,
};
use indexmap::IndexMap;
use pretty_assertions::assert_eq;
use std::collections::HashMap;

fn run(text: impl AsRef<str>) -> miette::Result<Value> {
    let source = Source::text_with_name("test", text);
    let ast = parse(&source)?;

    Ok(Scope::new(HashMap::new(), source, ast).eval()?)
}

/// Evaluate something expecting a runtime error.
fn run_err(text: impl AsRef<str>) -> RuntimeError {
    let source = Source::text_with_name("test", text);
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
    let input = r#"with {
    name = "John Doe"
}
"Hello, my name is ${name}!"
"#;
    let expected = Value::new(
        ValueKind::String("Hello, my name is John Doe!".into()),
        span(31, 28),
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
    let input = r"with {
    numbers = [ 1 2 3 ]
}
numbers[1]";
    let expected = Value::new(ValueKind::Int(2), span(25, 1));
    assert_eq!(run(input).unwrap(), expected);

    // Out of bounds index
    let input = r"with {
    numbers = [ 1 2 3 ]
}
numbers[3]";
    let expected = RuntimeError::new(
        RuntimeErrorKind::IndexOutOfBounds {
            length: 3,
            // TODO: The span start here should be `41` after making the index an expr
            index: span(33, 10),
        },
        Source::text_with_name("test", input),
        span(33, 10),
    );
    assert_eq!(run_err(input), expected);
}

#[test]
fn object() {
    let input = "{ name = \"John Doe\" age = 42 }";
    let expected = Value::new(
        ValueKind::Object({
            let mut map = IndexMap::new();
            map.insert(
                "name".into(),
                Value::new(ValueKind::String("John Doe".into()), span(9, 10)),
            );
            map.insert("age".into(), Value::new(ValueKind::Int(42), span(26, 2)));
            map
        }),
        span(0, 30),
    );
    assert_eq!(run(input).unwrap(), expected);
}

#[test]
fn field_access() {
    let input = r#"with {
    package = {
        dependencies = [ "other_package" ]
    }
}
package.dependencies"#;
    let expected = Value::new(
        ValueKind::Array(vec![Value::new(
            ValueKind::String("other_package".into()),
            span(48, 15),
        )]),
        span(46, 19),
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
fn parenthesized() {
    let input = "(1 + 3) * 2";
    let expected = Value::new(ValueKind::Int(8), span(0, 11));
    assert_eq!(run(input).unwrap(), expected);
}

#[test]
fn function() {
    let input = r#"with {
  greet = name: "Hello, ${name}!"
}
greet("John Doe")
"#;
    let expected = Value::new(ValueKind::String("Hello, John Doe!".into()), span(23, 17));
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
    let input = "with { name = \"John Doe\" } name";
    let expected = Value::new(ValueKind::String("John Doe".into()), span(14, 10));
    assert_eq!(run(input).unwrap(), expected);
}

#[test]
#[ignore = "Weird stack overflow bug that only happens in tests"]
fn recursion() {
    let input = r"with {
  pow = base: exponent: if(
    exponent == 0,
    1,
    base * pow(base, exponent - 1)
  )
}
pow(2, 10)";
    let expected = Value::new(ValueKind::Int(1024), span(0, 0));
    assert_eq!(run(input).unwrap(), expected);
}
