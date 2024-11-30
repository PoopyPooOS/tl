use logger::{warn, Colorize, Location};

fn main() {
    warn!("Using new line in println! call", location: Location::new_with_section(Some("examples/highlighted/bad_code.rs".into()), 1..=1, 21..=22), hint: format!("Remove {} in {} call", "\\n".bold(), "println!".bold()));
}
