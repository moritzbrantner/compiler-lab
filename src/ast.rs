use crate::Span;

/// Stable index into one parsed expression tree.
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
///
/// Child relationships use stable IDs instead of recursive boxes, so parser construction does not
/// require cloning or moving already-built subtrees.
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
