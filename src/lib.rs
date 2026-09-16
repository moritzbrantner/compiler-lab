//! Copy-conscious compiler experiments.
//!
//! The source buffer remains authoritative. Front-end products refer back to it with compact spans
//! instead of copying lexemes into every token, diagnostic, or syntax node.

mod ast;
mod diagnostic;
mod lexer;
mod parser;
mod source;
mod span;
mod token;

pub use ast::{
    BinaryOperator, BlockId, BlockNode, ExprId, ExprKind, ExprNode, ExpressionTree, FunctionId,
    FunctionNode, Parameter, ProgramItem, ProgramTree, StmtId, StmtKind, StmtNode, UnaryOperator,
};
pub use diagnostic::Diagnostic;
pub use lexer::{Lexed, lex};
pub use parser::{ParsedExpression, ParsedProgram, parse_expression, parse_program};
pub use source::{DiagnosticView, SourceFile, SourcePosition, SourceTooLarge};
pub use span::Span;
pub use token::{Token, TokenKind};
