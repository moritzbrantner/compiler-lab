use crate::Span;

/// Stable index into one parsed expression tree or program expression arena.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExprId(u32);

impl ExprId {
    pub(crate) fn from_index(index: usize) -> Self {
        debug_assert!(index <= u32::MAX as usize);
        Self(index as u32)
    }

    pub(crate) fn index(self) -> usize {
        self.0 as usize
    }
}

/// Stable index into the statement arena of one parsed program.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StmtId(u32);

impl StmtId {
    pub(crate) fn from_index(index: usize) -> Self {
        debug_assert!(index <= u32::MAX as usize);
        Self(index as u32)
    }

    pub(crate) fn index(self) -> usize {
        self.0 as usize
    }
}

/// Stable index into the block arena of one parsed program.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(u32);

impl BlockId {
    pub(crate) fn from_index(index: usize) -> Self {
        debug_assert!(index <= u32::MAX as usize);
        Self(index as u32)
    }

    pub(crate) fn index(self) -> usize {
        self.0 as usize
    }
}

/// Stable index into the function arena of one parsed program.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionId(u32);

impl FunctionId {
    pub(crate) fn from_index(index: usize) -> Self {
        debug_assert!(index <= u32::MAX as usize);
        Self(index as u32)
    }

    pub(crate) fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOperator {
    Positive,
    Negate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

/// Syntax-only expression data. Names and literal text remain represented by the node span.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExprKind {
    Integer,
    Name,
    Group {
        expression: ExprId,
    },
    Unary {
        operator: UnaryOperator,
        operand: ExprId,
    },
    Binary {
        left: ExprId,
        operator: BinaryOperator,
        right: ExprId,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExprNode {
    kind: ExprKind,
    span: Span,
}

impl ExprNode {
    pub(crate) const fn new(kind: ExprKind, span: Span) -> Self {
        Self { kind, span }
    }

    #[must_use]
    pub const fn kind(self) -> ExprKind {
        self.kind
    }

    #[must_use]
    pub const fn span(self) -> Span {
        self.span
    }
}

/// A compact syntax tree for one successfully parsed expression.
#[derive(Debug, PartialEq, Eq)]
pub struct ExpressionTree {
    nodes: Vec<ExprNode>,
    root: ExprId,
}

impl ExpressionTree {
    pub(crate) fn new(nodes: Vec<ExprNode>, root: ExprId) -> Self {
        Self { nodes, root }
    }

    #[must_use]
    pub const fn root(&self) -> ExprId {
        self.root
    }

    #[must_use]
    pub fn node(&self, id: ExprId) -> ExprNode {
        self.nodes[id.index()]
    }

    #[must_use]
    pub fn nodes(&self) -> &[ExprNode] {
        &self.nodes
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StmtKind {
    Let { name: Span, initializer: ExprId },
    Return { value: Option<ExprId> },
    Expression { expression: ExprId },
    Block { block: BlockId },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StmtNode {
    kind: StmtKind,
    span: Span,
}

impl StmtNode {
    pub(crate) const fn new(kind: StmtKind, span: Span) -> Self {
        Self { kind, span }
    }

    #[must_use]
    pub const fn kind(self) -> StmtKind {
        self.kind
    }

    #[must_use]
    pub const fn span(self) -> Span {
        self.span
    }
}

/// One source-order block references a contiguous range in the program's statement-order table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockNode {
    statement_start: u32,
    statement_count: u32,
    span: Span,
}

impl BlockNode {
    pub(crate) fn new(statement_start: usize, statement_count: usize, span: Span) -> Self {
        debug_assert!(statement_start <= u32::MAX as usize);
        debug_assert!(statement_count <= u32::MAX as usize);
        Self {
            statement_start: statement_start as u32,
            statement_count: statement_count as u32,
            span,
        }
    }

    #[must_use]
    pub const fn span(self) -> Span {
        self.span
    }

    pub(crate) fn statement_range(self) -> core::ops::Range<usize> {
        let start = self.statement_start as usize;
        start..start + self.statement_count as usize
    }
}

/// A function parameter keeps only source spans for its name and optional type name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Parameter {
    name: Span,
    type_name: Span,
}

impl Parameter {
    pub(crate) const fn new(name: Span, type_name: Span) -> Self {
        Self { name, type_name }
    }

    #[must_use]
    pub const fn name(self) -> Span {
        self.name
    }

    #[must_use]
    pub const fn type_name(self) -> Span {
        self.type_name
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionNode {
    name: Span,
    parameter_start: u32,
    parameter_count: u32,
    return_type: Option<Span>,
    body: BlockId,
    span: Span,
}

impl FunctionNode {
    pub(crate) fn new(
        name: Span,
        parameter_start: usize,
        parameter_count: usize,
        return_type: Option<Span>,
        body: BlockId,
        span: Span,
    ) -> Self {
        debug_assert!(parameter_start <= u32::MAX as usize);
        debug_assert!(parameter_count <= u32::MAX as usize);
        Self {
            name,
            parameter_start: parameter_start as u32,
            parameter_count: parameter_count as u32,
            return_type,
            body,
            span,
        }
    }

    #[must_use]
    pub const fn name(self) -> Span {
        self.name
    }

    #[must_use]
    pub const fn return_type(self) -> Option<Span> {
        self.return_type
    }

    #[must_use]
    pub const fn body(self) -> BlockId {
        self.body
    }

    #[must_use]
    pub const fn span(self) -> Span {
        self.span
    }

    pub(crate) fn parameter_range(self) -> core::ops::Range<usize> {
        let start = self.parameter_start as usize;
        start..start + self.parameter_count as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProgramItem {
    Function(FunctionId),
    Statement(StmtId),
}

/// Compact syntax storage for one successfully parsed source unit.
///
/// Every syntax object is stored once. Relationships use IDs and contiguous side tables, while all
/// user-written names and types remain spans into the authoritative source buffer.
#[derive(Debug, PartialEq, Eq)]
pub struct ProgramTree {
    expressions: Vec<ExprNode>,
    statements: Vec<StmtNode>,
    blocks: Vec<BlockNode>,
    block_statements: Vec<StmtId>,
    parameters: Vec<Parameter>,
    functions: Vec<FunctionNode>,
    items: Vec<ProgramItem>,
}

impl ProgramTree {
    pub(crate) fn new(
        expressions: Vec<ExprNode>,
        statements: Vec<StmtNode>,
        blocks: Vec<BlockNode>,
        block_statements: Vec<StmtId>,
        parameters: Vec<Parameter>,
        functions: Vec<FunctionNode>,
        items: Vec<ProgramItem>,
    ) -> Self {
        Self {
            expressions,
            statements,
            blocks,
            block_statements,
            parameters,
            functions,
            items,
        }
    }

    #[must_use]
    pub fn items(&self) -> &[ProgramItem] {
        &self.items
    }

    #[must_use]
    pub fn expression(&self, id: ExprId) -> ExprNode {
        self.expressions[id.index()]
    }

    #[must_use]
    pub fn expressions(&self) -> &[ExprNode] {
        &self.expressions
    }

    #[must_use]
    pub fn statement(&self, id: StmtId) -> StmtNode {
        self.statements[id.index()]
    }

    #[must_use]
    pub fn statements(&self) -> &[StmtNode] {
        &self.statements
    }

    #[must_use]
    pub fn block(&self, id: BlockId) -> BlockNode {
        self.blocks[id.index()]
    }

    #[must_use]
    pub fn block_statements(&self, id: BlockId) -> &[StmtId] {
        &self.block_statements[self.block(id).statement_range()]
    }

    #[must_use]
    pub fn function(&self, id: FunctionId) -> FunctionNode {
        self.functions[id.index()]
    }

    #[must_use]
    pub fn functions(&self) -> &[FunctionNode] {
        &self.functions
    }

    #[must_use]
    pub fn parameters(&self, id: FunctionId) -> &[Parameter] {
        &self.parameters[self.function(id).parameter_range()]
    }
}
