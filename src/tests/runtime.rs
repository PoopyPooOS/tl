use crate::eval;

macro_rules! eval {
    ($type:ty, $input:expr) => {
        eval::<$type>($input).unwrap().unwrap()
    };
}

#[test]
fn binary_operators() {
    let input = "1 + 3 + 4 + 2 * 3 / 2";
    let expected = 11;
    assert_eq!(eval!(i64, input), expected);
}

#[test]
fn functions() {
    let input = r#"
        fn greet(name) {
            "Hello, ${name}!"
        }

        greet("John Doe")
    "#;
    let expected = "Hello, John Doe!";
    assert_eq!(eval!(String, input), expected);
}

#[cfg(feature = "serde")]
#[test]
fn serde() {
    #[derive(PartialEq, Debug, serde::Deserialize)]
    struct Person {
        name: String,
        age: u8,
    }

    let input = r#"
        [
            { name = "John Doe" age = 43 }
            { name = "Jane Doe" age = 41 }
        ]
    "#;
    let expected = vec![
        Person {
            name: "John Doe".to_string(),
            age: 43,
        },
        Person {
            name: "Jane Doe".to_string(),
            age: 41,
        },
    ];
    assert_eq!(eval!(Vec<Person>, input), expected);
}
