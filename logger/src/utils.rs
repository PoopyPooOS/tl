use colored::{Color, Colorize};
use std::ops::RangeInclusive;

pub fn highlight(input: impl Into<String>, range: &RangeInclusive<usize>, color: Color) -> String {
    let input: String = input.into();
    let input_len = input.len();

    let start = range.start().min(&input_len).saturating_sub(1);
    let end = range.end().min(&input_len).saturating_sub(1);

    if start > end {
        return input;
    }

    let before = &input[..start];
    let highlighted = &input[start..=end];
    let after = &input[(end + 1)..];

    format!("{}{}{}", before, highlighted.color(color).bold(), after)
}

pub fn bold_highlight(input: impl Into<String>, range: &RangeInclusive<usize>) -> String {
    let input: String = input.into();
    let input_len = input.len();

    let start = range.start().min(&input_len).saturating_sub(1);
    let end = range.end().min(&input_len).saturating_sub(1);

    if start > end {
        return input;
    }

    let before = &input[..start];
    let highlighted = &input[start..=end];
    let after = &input[(end + 1)..];

    format!("{}{}{}", before, highlighted.bold(), after)
}

pub fn range_contains(range: &RangeInclusive<usize>, idx: usize) -> bool {
    range.start() <= &idx && range.end() >= &idx
}

pub fn remove_excess_tabs(input: impl Into<String>) -> String {
    let input: String = input.into();
    let min_whitespace = input
        .lines()
        .filter(|line| !line.trim().is_empty()) // Ignore empty lines
        .map(|line| line.chars().take_while(|&c| c == ' ' || c == '\t').collect::<String>())
        .min_by_key(String::len)
        .unwrap_or_default();

    let min_len = min_whitespace.len();

    // Remove the minimum amount of leading whitespace from each line
    input
        .lines()
        .map(|line| {
            if line.trim().is_empty() {
                String::new() // Keep empty lines as they are
            } else {
                line.chars().skip(min_len).collect::<String>()
            }
        })
        .collect::<Vec<String>>()
        .join("\n")
}
