use logger::{warn, Colorize, Location};

fn main() {
    warn!("Using new line in println! call", location: Location::new_with_section("examples/highlighted/bad_code.rs", 2..=2, 21..=22), hint: format!("Remove {} in {} call", "\\n".bold(), "println!".bold()));
}
