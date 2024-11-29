use logger::error;
use std::{fs, path::PathBuf, process};

#[derive(Debug, Clone)]
pub struct Source {
    pub path: Option<PathBuf>,
    pub text: String,
}

impl Source {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            path: None,
            text: text.into(),
        }
    }

    pub fn new_with_path(path: PathBuf, text: impl Into<String>) -> Self {
        Self {
            path: Some(path),
            text: text.into(),
        }
    }
}

impl From<PathBuf> for Source {
    fn from(path: PathBuf) -> Self {
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(err) => {
                error!(format!("Failed to read from file '{}':\n{err:#?}", &path.display()));
                process::exit(1);
            }
        }
        .trim()
        .to_string();

        Self { path: Some(path), text }
    }
}

impl From<String> for Source {
    fn from(text: String) -> Self {
        Self { path: None, text }
    }
}

impl From<&str> for Source {
    fn from(text: &str) -> Self {
        Self {
            path: None,
            text: text.to_string(),
        }
    }
}

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.text)
    }
}
