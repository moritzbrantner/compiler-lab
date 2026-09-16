use compiler_lab::{SourceFile, lex};

#[test]
fn positions_are_one_based_and_unicode_aware() {
    let source = SourceFile::new("let value = 1;\nlet π = 2;\n")
        .expect("fixture must fit in one source unit");
    let lexed = lex(&source);
    let diagnostic = lexed.diagnostics()[0];

    let position = source.position(diagnostic.span());
    assert_eq!(position.line(), 2);
    assert_eq!(position.column(), 5);
    assert_eq!(source.line_count(), 3);

    // Repeated location queries reuse the memoized index and remain deterministic.
    assert_eq!(source.position(diagnostic.span()), position);
    assert_eq!(source.line_count(), 3);
}

#[test]
fn diagnostic_rendering_borrows_source_and_materializes_only_on_request() {
    let source = SourceFile::new("let value = 1;\nlet π = 2;")
        .expect("fixture must fit in one source unit");
    let lexed = lex(&source);
    let view = source.diagnostic_view(lexed.diagnostics()[0]);

    assert_eq!(view.line_text(), "let π = 2;");
    assert_eq!(
        view.to_string(),
        "LEX001: unexpected character\n --> 2:5\n  |\n2 | let π = 2;\n  |     ^"
    );
}

#[test]
fn crlf_is_not_exposed_as_line_text() {
    let source =
        SourceFile::new("let a = 1;\r\nπ").expect("fixture must fit in one source unit");
    let lexed = lex(&source);
    let view = source.diagnostic_view(lexed.diagnostics()[0]);

    assert_eq!(view.position().line(), 2);
    assert_eq!(view.position().column(), 1);
    assert_eq!(view.line_text(), "π");
}
