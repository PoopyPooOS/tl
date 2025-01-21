#![allow(clippy::unwrap_used, reason = "Panics automatically invalidate tests")]
#![allow(clippy::too_many_lines, reason = "Some tests can get very long")]

use crate::{
    parser::parse,
    runtime::{types::Value, Scope},
    Source,
};
use logger::Log;
use pretty_assertions::assert_eq;
use std::collections::BTreeMap;

fn run(text: impl Into<String>) -> Result<Value, Box<Log>> {
    let source = Source::new(text);
    let ast = parse(&source).map_err(|err| Log::from(*err))?;

    Ok(Scope::new(source, ast)
        .eval()
        .map_err(|err| Log::from(*err))?)
}

fn make_scope(text: impl Into<String>) -> Result<Scope, Box<Log>> {
    let source = Source::new(text);
    let ast = parse(&source).map_err(|err| Log::from(*err))?;

    Ok(Scope::new(source, ast))
}

#[test]
fn boolean() {
    let input = "true";
    let expected = Value::Boolean(true);
    assert_eq!(run(input).unwrap(), expected);

    let input = "false";
    let expected = Value::Boolean(false);
    assert_eq!(run(input).unwrap(), expected);
}

#[test]
fn number() {
    let input = "42";
    let expected = Value::Int(42);
    assert_eq!(run(input).unwrap(), expected);
}

#[allow(clippy::approx_constant)]
#[test]
fn float() {
    let input = "3.14";
    let expected = Value::Float(3.14);
    assert_eq!(run(input).unwrap(), expected);
}

#[test]
fn string() {
    let input = "\"Hello, world!\"";
    let expected = Value::String("Hello, world!".into());
    assert_eq!(run(input).unwrap(), expected);
}

#[test]
fn escaped_string() {
    let input = "\"Hello, \\n\\tworld!\"";
    let expected = Value::String("Hello, \n\tworld!".into());
    assert_eq!(run(input).unwrap(), expected);
}

#[test]
fn interpolated_string() {
    let input = r#"
        let name = "John Doe"
        "Hello, my name is ${name}!"
    "#;
    let expected = Value::String("Hello, my name is John Doe!".into());
    assert_eq!(run(input).unwrap(), expected);
}
#[test]
fn array() {
    let input = "[ 1 2 3 ]";
    let expected = Value::Array(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    assert_eq!(run(input).unwrap(), expected);
}

#[test]
fn object() {
    let input = "{ name = \"John Doe\" age = 42 }";
    let expected = Value::Object(BTreeMap::from([
        ("name".into(), Value::String("John Doe".into())),
        ("age".into(), Value::Int(42)),
    ]));
    assert_eq!(run(input).unwrap(), expected);
}

#[ignore = "field access is not implemented"]
#[test]
fn field_access() {
    let input = r#"
        let package = {
            dependencies = [ "other_package" ]
        }

        package.dependencies
    "#;
    let expected = Value::Array(vec![Value::String("other_package".into())]);
    assert_eq!(run(input).unwrap(), expected);
}

#[test]
fn not() {
    let input = "!true";
    let expected = Value::Boolean(false);
    assert_eq!(run(input).unwrap(), expected);
}

#[test]
fn function() {
    let input = r#"
        let greet = (name) {
            "Hello, ${name}!"
        }

        greet("John Doe")
    "#;
    let expected = Value::String("Hello, John Doe!".into());
    assert_eq!(run(input).unwrap(), expected);
}

#[test]
fn binary_op() {
    let input = "2 + 3 * 4";
    let expected = Value::Int(14);
    assert_eq!(run(input).unwrap(), expected);
}

#[test]
fn variable() {
    let input = "let name = \"John Doe\"";
    let mut scope = make_scope(input).unwrap();
    scope.eval().unwrap();
    assert_eq!(
        scope.fetch_var(&"name"),
        Some(&Value::String("John Doe".into()))
    );
}

#[test]
fn recursion() {
    let input = r"
    let pow = (base exponent) {
        if(
            exponent == 0
            1
            base * pow(base exponent - 1)
        )
    }

    pow(2 10)";
    let expected = Value::Int(1024);
    assert_eq!(run(input).unwrap(), expected);
}
