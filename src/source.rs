use miette::{NamedSource, SourceCode};
use std::{
    io,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Source {
    pub(crate) inner: NamedSource<String>,
    pub(crate) path: Option<PathBuf>,
}

impl Source {
    pub fn new(inner: NamedSource<String>, path: Option<PathBuf>) -> Self {
        Self { inner, path }
    }

    pub fn path(path: impl AsRef<Path>) -> io::Result<Self> {
        let text = std::fs::read_to_string(&path)?;

        Ok(Self {
            inner: NamedSource::new(path.as_ref().display().to_string(), text),
            path: Some(path.as_ref().to_path_buf()),
        })
    }

    pub fn text(text: impl AsRef<str>) -> Self {
        Self {
            inner: NamedSource::new("builtin", text.as_ref().to_owned()),
            path: None,
        }
    }

    pub fn text_with_name(name: impl AsRef<str>, text: impl AsRef<str>) -> Self {
        Self {
            inner: NamedSource::new(name, text.as_ref().to_owned()),
            path: None,
        }
    }

    pub fn with_text(mut self, text: impl AsRef<str>) -> Self {
        self.inner = NamedSource::new(self.inner.name(), text.as_ref().to_owned());
        self
    }

    pub fn name(&self) -> &str {
        self.inner.name()
    }

    pub fn inner(&self) -> &String {
        self.inner.inner()
    }
}

impl SourceCode for Source {
    fn read_span<'a>(
        &'a self,
        span: &miette::SourceSpan,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Result<Box<dyn miette::SpanContents<'a> + 'a>, miette::MietteError> {
        self.inner
            .read_span(span, context_lines_before, context_lines_after)
    }
}
