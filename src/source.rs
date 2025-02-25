use std::{
    fmt::Display,
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Default)]
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

    pub fn from_text(text: impl Into<String>) -> Self {
        Self::from(text.into())
    }

    /// Reads from the given file path.
    ///
    /// # Errors
    /// This function will return an error if it fails to read from the given file path.
    pub fn from_path(path: impl AsRef<Path>) -> io::Result<Self> {
        let text = fs::read_to_string(&path)?;

        Ok(Self {
            path: Some(path.as_ref().to_path_buf()),
            text,
        })
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

impl Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.text)
    }
}
