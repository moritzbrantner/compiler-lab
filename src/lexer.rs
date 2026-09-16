use core::fmt;

use crate::{Diagnostic, Span, Token, TokenKind};

/// The result of one deterministic lexical pass over a borrowed source buffer.
#[derive(Debug, PartialEq, Eq)]
pub struct Lexed<'src> {
    source: &'src str,
    tokens: Vec<Token>,
    diagnostics: Vec<Diagnostic>,
}

impl<'src> Lexed<'src> {
    #[must_use]
    pub const fn source(&self) -> &'src str {
        self.source
    }

    #[must_use]
    pub fn tokens(&self) -> &[Token] {
        &self.tokens
    }

    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Resolves a token's text from the authoritative source without storing a copied lexeme.
    #[must_use]
    pub fn token_text(&self, token: Token) -> &'src str {
        &self.source[token.span().range()]
    }

    /// Resolves any compiler-produced span against the authoritative source.
    #[must_use]
    pub fn span_text(&self, span: Span) -> &'src str {
        &self.source[span.range()]
    }
}

/// The lexer uses compact 32-bit source positions, so one source unit is bounded to 4 GiB.
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

/// Lexes one source unit in a single forward pass.
///
/// Identifiers are deliberately ASCII-only in this first slice. A non-ASCII character is emitted
/// as one whole-character diagnostic span so every produced span remains a valid UTF-8 boundary.
pub fn lex(source: &str) -> Result<Lexed<'_>, SourceTooLarge> {
    if source.len() > u32::MAX as usize {
        return Err(SourceTooLarge {
            bytes: source.len(),
        });
    }

    Ok(Lexer::new(source).run())
}

struct Lexer<'src> {
    source: &'src str,
    current: usize,
    tokens: Vec<Token>,
    diagnostics: Vec<Diagnostic>,
}

impl<'src> Lexer<'src> {
    fn new(source: &'src str) -> Self {
        Self {
            source,
            current: 0,
            tokens: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    fn run(mut self) -> Lexed<'src> {
        while self.current < self.source.len() {
            match self.byte() {
                b' ' | b'\t' | b'\r' | b'\n' => self.current += 1,
                b'/' if self.peek(1) == Some(b'/') => self.skip_line_comment(),
                b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.lex_identifier(),
                b'0'..=b'9' => self.lex_integer(),
                b'(' => self.push_single(TokenKind::LeftParen),
                b')' => self.push_single(TokenKind::RightParen),
                b'{' => self.push_single(TokenKind::LeftBrace),
                b'}' => self.push_single(TokenKind::RightBrace),
                b',' => self.push_single(TokenKind::Comma),
                b':' => self.push_single(TokenKind::Colon),
                b';' => self.push_single(TokenKind::Semicolon),
                b'+' => self.push_single(TokenKind::Plus),
                b'-' if self.peek(1) == Some(b'>') => self.push_arrow(),
                b'-' => self.push_single(TokenKind::Minus),
                b'*' => self.push_single(TokenKind::Star),
                b'/' => self.push_single(TokenKind::Slash),
                b'=' => self.push_single(TokenKind::Equal),
                _ => self.push_unexpected_character(),
            }
        }

        let eof = Span::from_usize(self.current, self.current);
        self.tokens.push(Token::new(TokenKind::Eof, eof));

        Lexed {
            source: self.source,
            tokens: self.tokens,
            diagnostics: self.diagnostics,
        }
    }

    fn byte(&self) -> u8 {
        self.source.as_bytes()[self.current]
    }

    fn peek(&self, offset: usize) -> Option<u8> {
        self.source
            .as_bytes()
            .get(self.current + offset)
            .copied()
    }

    fn push_single(&mut self, kind: TokenKind) {
        let start = self.current;
        self.current += 1;
        self.push_token(kind, start, self.current);
    }

    fn push_arrow(&mut self) {
        let start = self.current;
        self.current += 2;
        self.push_token(TokenKind::Arrow, start, self.current);
    }

    fn lex_identifier(&mut self) {
        let start = self.current;
        self.current += 1;

        while self.current < self.source.len()
            && matches!(self.byte(), b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_')
        {
            self.current += 1;
        }

        let span = Span::from_usize(start, self.current);
        let kind = match &self.source[span.range()] {
            "let" => TokenKind::Let,
            "fn" => TokenKind::Fn,
            "return" => TokenKind::Return,
            _ => TokenKind::Identifier,
        };
        self.tokens.push(Token::new(kind, span));
    }

    fn lex_integer(&mut self) {
        let start = self.current;
        self.current += 1;

        while self.current < self.source.len() && self.byte().is_ascii_digit() {
            self.current += 1;
        }

        self.push_token(TokenKind::Integer, start, self.current);
    }

    fn skip_line_comment(&mut self) {
        self.current += 2;
        while self.current < self.source.len() && self.byte() != b'\n' {
            self.current += 1;
        }
    }

    fn push_unexpected_character(&mut self) {
        let start = self.current;
        let width = self.source[self.current..]
            .chars()
            .next()
            .map_or(1, char::len_utf8);
        self.current += width;
        let span = Span::from_usize(start, self.current);
        self.diagnostics.push(Diagnostic::new(
            "LEX001",
            "unexpected character",
            span,
        ));
    }

    fn push_token(&mut self, kind: TokenKind, start: usize, end: usize) {
        self.tokens
            .push(Token::new(kind, Span::from_usize(start, end)));
    }
}
