use std::fmt::{self, Display};
use std::ops::Range;
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Clone)]
pub struct LayoutDiagnostic {
    summary: String,
    source: String,
    evidence: SourceSpan,
    label: String,
    kind: SnippetKind,
}

impl LayoutDiagnostic {
    pub fn new(
        summary: impl Into<String>,
        source: impl Into<String>,
        evidence: impl Into<SourceSpan>,
        label: impl Into<String>,
        kind: SnippetKind,
    ) -> Self {
        Self {
            summary: summary.into(),
            source: source.into(),
            evidence: evidence.into(),
            label: label.into(),
            kind,
        }
    }

    pub fn quoted(
        summary: impl Into<String>,
        source: impl Into<String>,
        evidence: impl Into<SourceSpan>,
        label: impl Into<String>,
    ) -> Self {
        Self::new(summary, source, evidence, label, SnippetKind::QuotedText)
    }

    pub fn plain(summary: impl Into<String>, source: impl Into<String>, evidence: impl Into<SourceSpan>, label: impl Into<String>) -> Self {
        Self::new(summary, source, evidence, label, SnippetKind::Plain)
    }

    pub fn summary(&self) -> &str {
        &self.summary
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn evidence(&self) -> SourceSpan {
        self.evidence
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn kind(&self) -> SnippetKind {
        self.kind
    }

    pub(crate) fn write_evidence(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_snippet(f, self.source(), self.evidence, self.label(), self.kind())
    }
}

impl Display for LayoutDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.summary)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnippetKind {
    QuotedText,
    Plain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceSpan {
    offset: usize,
    len: usize,
}

impl SourceSpan {
    pub const fn new(offset: usize, len: usize) -> Self {
        Self { offset, len }
    }

    pub const fn offset(self) -> usize {
        self.offset
    }

    pub const fn len(self) -> usize {
        self.len
    }

    fn normalized_range(self, source_len: usize) -> Range<usize> {
        let start = self.offset().min(source_len);
        let end = self.offset().saturating_add(self.len()).max(start).min(source_len);
        start..end
    }
}

impl From<Range<usize>> for SourceSpan {
    fn from(value: Range<usize>) -> Self {
        let end = value.end.max(value.start);
        Self::new(value.start, end - value.start)
    }
}

impl From<(usize, usize)> for SourceSpan {
    fn from((offset, len): (usize, usize)) -> Self {
        Self::new(offset, len)
    }
}

fn write_snippet(f: &mut fmt::Formatter<'_>, source: &str, span: SourceSpan, label: &str, kind: SnippetKind) -> fmt::Result {
    let span = span.normalized_range(source.len());
    let source_line = match kind {
        SnippetKind::QuotedText => format!("\"{source}\""),
        SnippetKind::Plain => source.to_string(),
    };
    let leading_width = match kind {
        SnippetKind::QuotedText => 1,
        SnippetKind::Plain => 0,
    } + UnicodeWidthStr::width(&source[..span.start]);
    let marker_width = UnicodeWidthStr::width(&source[span]).max(1);

    write!(
        f,
        "{source_line}\n{}{}\n{label}",
        " ".repeat(leading_width),
        "^".repeat(marker_width),
    )
}
