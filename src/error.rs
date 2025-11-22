use crate::Source;
use miette::{Diagnostic, LabeledSpan, Severity, SourceSpan};
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub struct Error<E: Diagnostic> {
    pub kind: E,

    pub source: Source,
    pub span: SourceSpan,
}

impl<E: Diagnostic> Diagnostic for Error<E> {
    fn severity(&self) -> Option<miette::Severity> {
        Some(Severity::Error)
    }

    fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        self.kind.diagnostic_source()
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        Some(&self.source)
    }

    fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        self.kind.code()
    }

    fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        self.kind.help()
    }

    fn url<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        self.kind.url()
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        let extra_label = LabeledSpan::new_primary_with_span(None, self.span);

        match self.kind.labels() {
            Some(labels) => Some(Box::new(labels.chain(std::iter::once(extra_label)))),
            None => Some(Box::new(std::iter::once(extra_label))),
        }
    }

    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn Diagnostic> + 'a>> {
        self.kind.related()
    }
}

impl<E: Diagnostic> Display for Error<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.kind, f)
    }
}

impl<E: Diagnostic> std::error::Error for Error<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.kind.source()
    }
}

impl<T: Diagnostic> Error<T> {
    pub fn new(kind: T, source: Source, span: SourceSpan) -> Self {
        Self { kind, source, span }
    }
}
