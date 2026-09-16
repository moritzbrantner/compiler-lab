use core::mem::size_of;

use compiler_lab::{SourceFile, Token, TokenKind, lex};

#[test]
fn lexes_keywords_identifiers_integers_and_punctuation() {
    let source = SourceFile::new("let total = 12 + value;\nreturn total;")
        .expect("fixture must fit in one source unit");
    let lexed = lex(&source);

    let kinds: Vec<_> = lexed.tokens().iter().map(|token| token.kind()).collect();
    assert_eq!(
        kinds,
        [
            TokenKind::Let,
            TokenKind::Identifier,
            TokenKind::Equal,
            TokenKind::Integer,
            TokenKind::Plus,
            TokenKind::Identifier,
            TokenKind::Semicolon,
            TokenKind::Return,
            TokenKind::Identifier,
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
    assert!(lexed.diagnostics().is_empty());
    assert_eq!(lexed.token_text(lexed.tokens()[1]), "total");
    assert_eq!(lexed.token_text(lexed.tokens()[3]), "12");
}

#[test]
fn skips_line_comments_and_recognizes_function_surface() {
    let source = SourceFile::new(
        "fn add(left: value, right: value) -> value { // body\nreturn left + right; }",
    )
    .expect("fixture must fit in one source unit");
    let lexed = lex(&source);

    let kinds: Vec<_> = lexed.tokens().iter().map(|token| token.kind()).collect();
    assert_eq!(kinds[0], TokenKind::Fn);
    assert!(kinds.contains(&TokenKind::Arrow));
    assert!(kinds.contains(&TokenKind::Return));
    assert!(
        !lexed
            .tokens()
            .iter()
            .any(|token| lexed.token_text(*token) == "body")
    );
    assert!(lexed.diagnostics().is_empty());
}

#[test]
fn unexpected_unicode_uses_a_whole_character_span() {
    let source = SourceFile::new("let π = 1;").expect("fixture must fit in one source unit");
    let lexed = lex(&source);

    assert_eq!(lexed.diagnostics().len(), 1);
    let diagnostic = lexed.diagnostics()[0];
    assert_eq!(diagnostic.code(), "LEX001");
    assert_eq!(diagnostic.message(), "unexpected character");
    assert_eq!(lexed.span_text(diagnostic.span()), "π");

    let kinds: Vec<_> = lexed.tokens().iter().map(|token| token.kind()).collect();
    assert_eq!(
        kinds,
        [
            TokenKind::Let,
            TokenKind::Equal,
            TokenKind::Integer,
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexical_output_is_replayable() {
    let source =
        SourceFile::new("let answer = 40 + 2;").expect("fixture must fit in one source unit");
    assert_eq!(lex(&source), lex(&source));
}

#[test]
fn token_representation_stays_compact() {
    assert!(size_of::<Token>() <= 12);
}
