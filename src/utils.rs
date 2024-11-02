pub fn handle_string_escapes(original: impl Into<String>) -> String {
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
