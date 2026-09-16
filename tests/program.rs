use core::mem::size_of;

use compiler_lab::{
    BinaryOperator, BlockNode, ExprKind, FunctionNode, Parameter, ProgramItem, SourceFile, StmtKind,
    StmtNode, lex, parse_program,
};

#[test]
fn parses_typed_function_with_binding_and_return() {
    let source = SourceFile::new(
        "fn add(left: value, right: value) -> value { let sum = left + right; return sum; }",
    )
    .expect("fixture must fit in one source unit");
    let lexed = lex(&source);
    let parsed = parse_program(&lexed);

    assert!(parsed.diagnostics().is_empty());
    let tree = parsed.tree().expect("valid program must publish a tree");
    let [ProgramItem::Function(function_id)] = tree.items() else {
        panic!("program must contain one function");
    };

    let function = tree.function(*function_id);
    assert_eq!(source.span_text(function.name()), "add");
    assert_eq!(source.span_text(function.return_type().unwrap()), "value");
    assert_eq!(
        source.span_text(function.span()),
        "fn add(left: value, right: value) -> value { let sum = left + right; return sum; }"
    );

    let parameters = tree.parameters(*function_id);
    assert_eq!(parameters.len(), 2);
    assert_eq!(source.span_text(parameters[0].name()), "left");
    assert_eq!(source.span_text(parameters[0].type_name()), "value");
    assert_eq!(source.span_text(parameters[1].name()), "right");

    let statements = tree.block_statements(function.body());
    assert_eq!(statements.len(), 2);

    let StmtKind::Let { name, initializer } = tree.statement(statements[0]).kind() else {
        panic!("first statement must be a binding");
    };
    assert_eq!(source.span_text(name), "sum");
    assert!(matches!(
        tree.expression(initializer).kind(),
        ExprKind::Binary {
            operator: BinaryOperator::Add,
            ..
        }
    ));

    let StmtKind::Return { value: Some(value) } = tree.statement(statements[1]).kind() else {
        panic!("second statement must return a value");
    };
    assert_eq!(tree.expression(value).kind(), ExprKind::Name);
    assert_eq!(source.span_text(tree.expression(value).span()), "sum");
}

#[test]
fn parses_top_level_statements_and_nested_blocks() {
    let source = SourceFile::new("let value = 1; { value + 2; return; }")
        .expect("fixture must fit in one source unit");
    let lexed = lex(&source);
    let parsed = parse_program(&lexed);

    assert!(parsed.diagnostics().is_empty());
    let tree = parsed.tree().expect("valid program must publish a tree");
    assert_eq!(tree.items().len(), 2);

    let ProgramItem::Statement(block_statement) = tree.items()[1] else {
        panic!("second item must be a block statement");
    };
    let StmtKind::Block { block } = tree.statement(block_statement).kind() else {
        panic!("second item must wrap a block");
    };
    let nested = tree.block_statements(block);
    assert_eq!(nested.len(), 2);
    assert!(matches!(
        tree.statement(nested[0]).kind(),
        StmtKind::Expression { .. }
    ));
    assert_eq!(
        tree.statement(nested[1]).kind(),
        StmtKind::Return { value: None }
    );
}

#[test]
fn accepts_trailing_parameter_comma_and_optional_return_type() {
    let source = SourceFile::new("fn run(input: value,) { return input; }")
        .expect("fixture must fit in one source unit");
    let lexed = lex(&source);
    let parsed = parse_program(&lexed);

    let tree = parsed.tree().expect("valid program must publish a tree");
    let ProgramItem::Function(function_id) = tree.items()[0] else {
        panic!("item must be a function");
    };
    let function = tree.function(function_id);
    assert_eq!(tree.parameters(function_id).len(), 1);
    assert_eq!(function.return_type(), None);
}

#[test]
fn statement_recovery_collects_stable_non_cascading_diagnostics() {
    let source = SourceFile::new("let = 1;\nlet two 2;\nreturn 3\nlet ok = 4;")
        .expect("fixture must fit in one source unit");
    let lexed = lex(&source);
    let parsed = parse_program(&lexed);

    assert!(parsed.tree().is_none());
    let codes: Vec<_> = parsed
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.code())
        .collect();
    assert_eq!(codes, ["PARSE101", "PARSE102", "PARSE103"]);
    assert_eq!(
        source.span_text(parsed.diagnostics()[0].span()),
        "="
    );
    assert_eq!(source.span_text(parsed.diagnostics()[1].span()), "2");
    assert_eq!(source.span_text(parsed.diagnostics()[2].span()), "let");
}

#[test]
fn malformed_function_signature_skips_its_body_before_resuming() {
    let source = SourceFile::new(
        "fn broken(left value) { let x = 1; }\nfn good() { return; }",
    )
    .expect("fixture must fit in one source unit");
    let lexed = lex(&source);
    let parsed = parse_program(&lexed);

    assert!(parsed.tree().is_none());
    assert_eq!(parsed.diagnostics().len(), 1);
    assert_eq!(parsed.diagnostics()[0].code(), "PARSE107");
    assert_eq!(parsed.diagnostics()[0].message(), "expected `:`");
    assert_eq!(source.span_text(parsed.diagnostics()[0].span()), "value");
}

#[test]
fn nested_function_declaration_reports_one_boundary_error() {
    let source = SourceFile::new("{ fn inner() { return; } return; }")
        .expect("fixture must fit in one source unit");
    let lexed = lex(&source);
    let parsed = parse_program(&lexed);

    assert!(parsed.tree().is_none());
    assert_eq!(parsed.diagnostics().len(), 1);
    assert_eq!(parsed.diagnostics()[0].code(), "PARSE113");
}

#[test]
fn program_output_is_replayable() {
    let source = SourceFile::new("fn id(value: item) -> item { return value; }")
        .expect("fixture must fit in one source unit");
    let lexed = lex(&source);

    assert_eq!(parse_program(&lexed), parse_program(&lexed));
}

#[test]
fn program_syntax_records_stay_compact() {
    assert!(size_of::<StmtNode>() <= 24);
    assert!(size_of::<BlockNode>() <= 16);
    assert!(size_of::<Parameter>() <= 16);
    assert!(size_of::<FunctionNode>() <= 40);
}
