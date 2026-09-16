use crate::Span;

/// Token categories for the first compiler-lab surface language.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Let,
    Fn,
    Return,
    Identifier,
    Integer,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Colon,
    Semicolon,
    Plus,
    Minus,
    Star,
    Slash,
    Equal,
    Arrow,
    Eof,
}

/// A token references source text by span instead of owning a copied lexeme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Token {
    kind: TokenKind,
    span: Span,
}

impl Token {
    pub(crate) const fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }

    #[must_use]
    pub const fn kind(self) -> TokenKind {
        self.kind
    }

    #[must_use]
    pub const fn span(self) -> Span {
        self.span
    }
}
