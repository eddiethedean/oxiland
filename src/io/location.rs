use std::fmt;
use std::ops::Range;

use oxigraph::io::TextPosition;

/// A text position inside an RDF source.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceLocation {
    /// Zero-based line number.
    pub line: u64,
    /// Zero-based column number.
    pub column: u64,
    /// Zero-based byte offset.
    pub offset: u64,
}

impl SourceLocation {
    pub(crate) fn from_text_position(position: TextPosition) -> Self {
        Self {
            line: position.line,
            column: position.column,
            offset: position.offset,
        }
    }

    pub(crate) fn from_range(range: Range<TextPosition>) -> Self {
        Self::from_text_position(range.start)
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            " at line {}, column {}, offset {}",
            self.line.saturating_add(1),
            self.column.saturating_add(1),
            self.offset
        )
    }
}
