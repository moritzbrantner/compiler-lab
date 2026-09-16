use core::mem::size_of;

use compiler_lab::{
    BinaryOperator, ExprKind, ExprNode, ParsedExpression, SourceFile, UnaryOperator, lex,
    parse_expression,
};

fn parse(source_text: &'static str) -> (SourceFile<'static>, ParsedExpression) {
    let source = SourceFile::new(source_text).expect("fixture must fit in one source unit");
    let lexed = lex(&source);
    assert!(lexed.diagnostics().is_empty());
    let parsed = parse_expression(&lexed);
    (source, parsed)
}

#[test]
fn multiplication_binds_more_tightly_than_addition() {
    let (source, parsed) = parse("1 + 2 * 3");
    assert!(parsed.diagnostics().is_empty());
    let tree = parsed.tree().expect("valid expression must publish a tree");

    let ExprKind::Binary {
        left,
        operator: BinaryOperator::Add,
        right,
    } = tree.node(tree.root()).kind()
    else {
        panic!("root must be addition");
    };

    assert_eq!(source.span_text(tree.node(left).span()), "1");
    let ExprKind::Binary {
        left: multiply_left,
        operator: BinaryOperator::Multiply,
        right: multiply_right,
    } = tree.node(right).kind()
    else {
        panic!("right operand must be multiplication");
    };
    assert_eq!(source.span_text(tree.node(multiply_left).span()), "2");
    assert_eq!(source.span_text(tree.node(multiply_right).span()), "3");
}

#[test]
fn binary_operators_are_left_associative() {
    let (_, parsed) = parse("8 - 3 - 1");
    let tree = parsed.tree().expect("valid expression must publish a tree");

    let ExprKind::Binary {
        left,
        operator: BinaryOperator::Subtract,
        ..
    } = tree.node(tree.root()).kind()
    else {
        panic!("root must be subtraction");
    };

    assert!(matches!(
        tree.node(left).kind(),
        ExprKind::Binary {
            operator: BinaryOperator::Subtract,
            ..
        }
    ));
}

#[test]
fn grouping_and_prefix_operators_override_infix_precedence() {
    let (source, parsed) = parse("-(value + 2) * +3");
    let tree = parsed.tree().expect("valid expression must publish a tree");
    let root = tree.node(tree.root());
    assert_eq!(source.span_text(root.span()), "-(value + 2) * +3");

    let ExprKind::Binary {
        left,
        operator: BinaryOperator::Multiply,
        right,
    } = root.kind()
    else {
        panic!("root must be multiplication");
    };

    let ExprKind::Unary {
        operator: UnaryOperator::Negate,
        operand,
    } = tree.node(left).kind()
    else {
        panic!("left operand must be negation");
    };
    assert!(matches!(tree.node(operand).kind(), ExprKind::Group { .. }));

    assert!(matches!(
        tree.node(right).kind(),
        ExprKind::Unary {
            operator: UnaryOperator::Positive,
            ..
        }
    ));
}

#[test]
fn names_and_literals_keep_source_identity_without_copied_values() {
    let (source, parsed) = parse("answer / 42");
    let tree = parsed.tree().expect("valid expression must publish a tree");

    let ExprKind::Binary { left, right, .. } = tree.node(tree.root()).kind() else {
        panic!("root must be binary");
    };
    assert_eq!(tree.node(left).kind(), ExprKind::Name);
    assert_eq!(tree.node(right).kind(), ExprKind::Integer);
    assert_eq!(source.span_text(tree.node(left).span()), "answer");
    assert_eq!(source.span_text(tree.node(right).span()), "42");
}

#[test]
fn invalid_syntax_never_publishes_a_partial_tree() {
    let source = SourceFile::new("1 +").expect("fixture must fit in one source unit");
    let lexed = lex(&source);
    let parsed = parse_expression(&lexed);

    assert!(parsed.tree().is_none());
    assert_eq!(parsed.diagnostics().len(), 1);
    let diagnostic = parsed.diagnostics()[0];
    assert_eq!(diagnostic.code(), "PARSE001");
    assert_eq!(diagnostic.message(), "expected expression");
    assert_eq!(
        source.diagnostic_view(diagnostic).to_string(),
        "PARSE001: expected expression\n --> 1:4\n  |\n1 | 1 +\n  |    ^"
    );
}

#[test]
fn reports_missing_group_terminator_at_the_current_span() {
    let (source, parsed) = parse("(1 + 2");

    assert!(parsed.tree().is_none());
    let diagnostic = parsed.diagnostics()[0];
    assert_eq!(diagnostic.code(), "PARSE002");
    assert_eq!(diagnostic.message(), "expected `)`");
    assert!(diagnostic.span().is_empty());
    assert_eq!(source.position(diagnostic.span()).column(), 7);
}

#[test]
fn rejects_tokens_after_a_complete_expression() {
    let (source, parsed) = parse("1 2");

    assert!(parsed.tree().is_none());
    let diagnostic = parsed.diagnostics()[0];
    assert_eq!(diagnostic.code(), "PARSE003");
    assert_eq!(diagnostic.message(), "unexpected token after expression");
    assert_eq!(source.span_text(diagnostic.span()), "2");
}

#[test]
fn parser_output_is_replayable() {
    let source =
        SourceFile::new("-1 + value * (2 + 3)").expect("fixture must fit in one source unit");
    let lexed = lex(&source);

    assert_eq!(parse_expression(&lexed), parse_expression(&lexed));
}

#[test]
fn expression_nodes_stay_compact() {
    assert!(size_of::<ExprNode>() <= 24);
}
