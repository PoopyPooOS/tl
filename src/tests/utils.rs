use crate::utils;

#[test]
fn string_escapes() {
    let input = r#"\"\'\n\r\t\0hello!"#;
    let expected = "\"\'\n\r\t\0hello!";
    assert_eq!(utils::handle_string_escapes(input), expected);
}
