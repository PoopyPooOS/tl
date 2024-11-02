#[cfg(test)]
use crate::tokenizer::{tokenize, Token};

#[test]
fn test_tokenize_simple_assignment() {
    let input = "let x = 42";
    let expected = vec![Token::Let, Token::Identifier("x".to_string()), Token::Equals, Token::Number(42)];

    assert_eq!(tokenize(input).unwrap(), expected);
}
