mod tokenizer {
    use crate::tokenizer::{tokenize, Token};

    #[test]
    fn boolean() {
        let input = "let valid = true";
        let expected = vec![Token::Let, Token::Identifier("valid".to_string()), Token::Equals, Token::Bool(true)];
        assert_eq!(tokenize(input).unwrap(), expected);

        let input = "let invalid = false";
        let expected = vec![
            Token::Let,
            Token::Identifier("invalid".to_string()),
            Token::Equals,
            Token::Bool(false),
        ];
        assert_eq!(tokenize(input).unwrap(), expected);
    }

    #[test]
    fn number() {
        let input = "42";
        let expected = vec![Token::Number(42)];
        assert_eq!(tokenize(input).unwrap(), expected);
    }

    #[test]
    fn float() {
        const PI: f64 = std::f64::consts::PI;

        let input = PI.to_string();
        let expected = vec![Token::Float(PI)];
        assert_eq!(tokenize(input).unwrap(), expected);
    }

    #[test]
    fn string() {
        let input = r#""Hello, world!""#;
        let expected = vec![Token::String("Hello, world!".to_string())];
        assert_eq!(tokenize(input).unwrap(), expected);
    }

    #[test]
    fn escaped_string() {
        let input = r#""Hello, \n\tworld!""#;
        let expected = vec![Token::String("Hello, \n\tworld!".to_string())];
        assert_eq!(tokenize(input).unwrap(), expected);
    }

    #[test]
    fn parenthesis() {
        let input = "(2 + 3) * 4";
        let expected = vec![
            Token::LParen,
            Token::Number(2),
            Token::Plus,
            Token::Number(3),
            Token::RParen,
            Token::Multiply,
            Token::Number(4),
        ];
        assert_eq!(tokenize(input).unwrap(), expected);
    }

    #[test]
    fn let_assignment() {
        let input = "let x = 42";
        let expected = vec![Token::Let, Token::Identifier("x".to_string()), Token::Equals, Token::Number(42)];
        assert_eq!(tokenize(input).unwrap(), expected);
    }

    #[test]
    fn unclosed_string() {
        let input = r#""Unclosed string"#;
        let result = tokenize(input);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Unclosed string literal");
    }

    #[test]
    fn identifier_with_underscore() {
        let input = "let _temp_var = 5";
        let expected = vec![
            Token::Let,
            Token::Identifier("_temp_var".to_string()),
            Token::Equals,
            Token::Number(5),
        ];
        assert_eq!(tokenize(input).unwrap(), expected);
    }
}
