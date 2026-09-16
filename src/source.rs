use core::fmt;
use std::sync::OnceLock;

use crate::{Diagnostic, Span};

/// One authoritative source unit with a lazily memoized line index.
#[derive(Debug)]
pub struct SourceFile<'src> {
    text: &'src str,
    line_starts: OnceLock<Vec<u32>>,
}

impl<'src> SourceFile<'src> {
    pub fn new(text: &'src str) -> Result<Self, SourceTooLarge> {
        if text.len() > u32::MAX as usize {
            return Err(SourceTooLarge { bytes: text.len() });
        }

        Ok(Self {
            text,
            line_starts: OnceLock::new(),
        })
    }

    #[must_use]
    pub const fn text(&self) -> &'src str {
        self.text
    }

    /// Resolves compiler-produced source spans without storing copied lexemes in derived data.
    #[must_use]
    pub fn span_text(&self, span: Span) -> &'src str {
        &self.text[span.range()]
    }

    #[must_use]
    pub fn line_count(&self) -> usize {
        self.line_starts().len()
    }

    /// Resolves the start of a compiler-produced span to a one-based line and character column.
    #[must_use]
    pub fn position(&self, span: Span) -> SourcePosition {
        self.position_at(span.start() as usize)
    }

    /// Creates a borrowed diagnostic view. Source snippets are resolved only while rendering.
    #[must_use]
    pub fn diagnostic_view(&self, diagnostic: Diagnostic) -> DiagnosticView<'_, 'src> {
        DiagnosticView {
            source: self,
            diagnostic,
        }
    }

    fn line_starts(&self) -> &[u32] {
        self.line_starts.get_or_init(|| {
            let mut starts = vec![0];
            starts.extend(
                self.text
                    .bytes()
                    .enumerate()
                    .filter_map(|(index, byte)| (byte == b'\n').then_some((index + 1) as u32)),
            );
            starts
        })
    }

    fn position_at(&self, offset: usize) -> SourcePosition {
        debug_assert!(offset <= self.text.len());
        debug_assert!(self.text.is_char_boundary(offset));

        let starts = self.line_starts();
        let line_index = starts.partition_point(|start| (*start as usize) <= offset) - 1;
        let line_start = starts[line_index] as usize;
        let column = self.text[line_start..offset].chars().count() + 1;

        SourcePosition {
            line: line_index + 1,
            column,
        }
    }

    fn line_bounds(&self, line_index: usize) -> (usize, usize) {
        let starts = self.line_starts();
        let start = starts[line_index] as usize;
        let mut end = starts
            .get(line_index + 1)
            .map_or(self.text.len(), |next| *next as usize);

        if end > start && self.text.as_bytes()[end - 1] == b'\n' {
            end -= 1;
        }
        if end > start && self.text.as_bytes()[end - 1] == b'\r' {
            end -= 1;
        }

        (start, end)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourcePosition {
    line: usize,
    column: usize,
}

impl SourcePosition {
    #[must_use]
    pub const fn line(self) -> usize {
        self.line
    }

    #[must_use]
    pub const fn column(self) -> usize {
        self.column
    }
}

/// A borrowed, on-demand rendering of one diagnostic against its source file.
#[derive(Debug)]
pub struct DiagnosticView<'file, 'src> {
    source: &'file SourceFile<'src>,
    diagnostic: Diagnostic,
}

impl<'file, 'src> DiagnosticView<'file, 'src> {
    #[must_use]
    pub fn position(&self) -> SourcePosition {
        self.source.position(self.diagnostic.span())
    }

    #[must_use]
    pub fn line_text(&self) -> &'src str {
        let line_index = self.position().line() - 1;
        let (start, end) = self.source.line_bounds(line_index);
        &self.source.text[start..end]
    }

    fn marker_width(&self) -> usize {
        let span = self.diagnostic.span();
        let start = span.start() as usize;
        let line_index = self.position().line() - 1;
        let (_, line_end) = self.source.line_bounds(line_index);
        let end = (span.end() as usize).min(line_end);

        if end <= start {
            1
        } else {
            self.source.text[start..end].chars().count().max(1)
        }
    }
}

impl fmt::Display for DiagnosticView<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let position = self.position();
        let gutter_width = position.line().ilog10() as usize + 1;

        writeln!(
            formatter,
            "{}: {}",
            self.diagnostic.code(),
            self.diagnostic.message()
        )?;
        writeln!(formatter, " --> {}:{}", position.line(), position.column())?;
        writeln!(formatter, "{:>gutter_width$} |", "")?;
        writeln!(
            formatter,
            "{:>gutter_width$} | {}",
            position.line(),
            self.line_text()
        )?;
        write!(formatter, "{:>gutter_width$} | ", "")?;
        write!(formatter, "{:>width$}", "", width = position.column() - 1)?;
        for _ in 0..self.marker_width() {
            write!(formatter, "^")?;
        }
        Ok(())
    }
}

/// The compiler uses compact 32-bit source spans, so one source unit is bounded to 4 GiB.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceTooLarge {
    bytes: usize,
}

impl SourceTooLarge {
    #[must_use]
    pub const fn bytes(self) -> usize {
        self.bytes
    }
}

impl fmt::Display for SourceTooLarge {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "source is {} bytes; compiler-lab source units are limited to {} bytes",
            self.bytes,
            u32::MAX
        )
    }
}

impl std::error::Error for SourceTooLarge {}
