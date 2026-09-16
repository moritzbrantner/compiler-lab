//! Copy-conscious compiler experiments.
//!
//! The source buffer remains authoritative. Front-end products refer back to it with compact spans
//! instead of copying lexemes into every token or diagnostic.

mod diagnostic;
mod lexer;
mod source;
mod span;
mod token;

pub use diagnostic::Diagnostic;
pub use lexer::{Lexed, lex};
pub use source::{DiagnosticView, SourceFile, SourcePosition, SourceTooLarge};
pub use span::Span;
pub use token::{Token, TokenKind};
