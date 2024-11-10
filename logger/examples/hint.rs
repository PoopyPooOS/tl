use logger::fatal;

fn main() {
    fatal!(
        "Failed to locate config file",
        hint: "Please make sure config.toml exists"
    );
}
