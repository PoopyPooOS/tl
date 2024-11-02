pub fn handle_string_escapes(original: impl Into<String>) -> String {
    let original: String = original.into();
    let mut result = String::new();
    let mut chars = original.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\'
            && let Some(&next_ch) = chars.peek()
        {
            match next_ch {
                '"' => {
                    result.push('"');
                    chars.next();
                }
                '\'' => {
                    result.push('\'');
                    chars.next();
                }
                'n' => {
                    result.push('\n');
                    chars.next();
                }
                'r' => {
                    result.push('\r');
                    chars.next();
                }
                't' => {
                    result.push('\t');
                    chars.next();
                }
                '0' => {
                    result.push('\0');
                    chars.next();
                }
                // Handle hex escape
                'x' => {
                    chars.next();
                    let mut hex = String::new();
                    for _ in 0..2 {
                        if let Some(&ch) = chars.peek() {
                            if ch.is_ascii_hexdigit() {
                                hex.push(ch);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                    }
                    if let Ok(value) = u8::from_str_radix(&hex, 16) {
                        result.push(value as char);
                    }
                }
                // Handle Unicode escape (4 digits)
                'u' => {
                    chars.next();
                    let mut unicode = String::new();
                    for _ in 0..4 {
                        if let Some(&ch) = chars.peek() {
                            if ch.is_ascii_hexdigit() {
                                unicode.push(ch);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                    }
                    if let Ok(value) = u32::from_str_radix(&unicode, 16) {
                        if let Some(ch) = char::from_u32(value) {
                            result.push(ch);
                        }
                    }
                }
                // Handle Unicode escape (8 digits)
                'U' => {
                    chars.next();
                    let mut unicode8 = String::new();
                    for _ in 0..8 {
                        if let Some(&ch) = chars.peek() {
                            if ch.is_ascii_hexdigit() {
                                unicode8.push(ch);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                    }
                    if let Ok(value) = u32::from_str_radix(&unicode8, 16) {
                        if let Some(ch) = char::from_u32(value) {
                            result.push(ch);
                        }
                    }
                }
                _ => {
                    result.push(ch);
                }
            }
        } else {
            result.push(ch);
        }
    }

    result
}
