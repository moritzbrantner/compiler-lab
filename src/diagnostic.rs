use crate::Span;

/// A deterministic compiler diagnostic. Messages are static; source excerpts are rendered on demand.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Diagnostic {
    code: &'static str,
    message: &'static str,
    span: Span,
}

impl Diagnostic {
    pub(crate) const fn new(code: &'static str, message: &'static str, span: Span) -> Self {
        Self {
            code,
            message,
            span,
        }
    }

    #[must_use]
    pub const fn code(self) -> &'static str {
        self.code
    }

    #[must_use]
    pub const fn message(self) -> &'static str {
        self.message
    }

    #[must_use]
    pub const fn span(self) -> Span {
        self.span
    }
}
