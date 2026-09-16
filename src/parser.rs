use crate::{
    BinaryOperator, Diagnostic, ExprId, ExprKind, ExprNode, ExpressionTree, Lexed, Span, Token,
    TokenKind, UnaryOperator,
};

const PREFIX_BINDING_POWER: u8 = 5;

/// The syntax result for one full expression source unit.
///
/// Invalid syntax never publishes a partial tree. Diagnostics reference the original source spans.
#[derive(Debug, PartialEq, Eq)]
pub struct ParsedExpression {
    tree: Option<ExpressionTree>,
    diagnostics: Vec<Diagnostic>,
}

impl ParsedExpression {
    #[must_use]
    pub fn tree(&self) -> Option<&ExpressionTree> {
        self.tree.as_ref()
    }

    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}

/// Parses the complete lexical token stream as one expression.
///
/// Lexical diagnostics remain owned by [`Lexed`]; this function reports syntax diagnostics only.
pub fn parse_expression(lexed: &Lexed<'_>) -> ParsedExpression {
    Parser::new(lexed.tokens()).run()
}

struct Parser<'tokens> {
    tokens: &'tokens [Token],
    current: usize,
    nodes: Vec<ExprNode>,
    diagnostics: Vec<Diagnostic>,
}

impl<'tokens> Parser<'tokens> {
    fn new(tokens: &'tokens [Token]) -> Self {
        Self {
            tokens,
            current: 0,
            nodes: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    fn run(mut self) -> ParsedExpression {
        let root = self.parse_precedence(0);

        if root.is_some() && self.current().kind() != TokenKind::Eof {
            let token = self.current();
            self.diagnostics.push(Diagnostic::new(
                "PARSE003",
                "unexpected token after expression",
                token.span(),
            ));
        }

        if !self.diagnostics.is_empty() {
            return ParsedExpression {
                tree: None,
                diagnostics: self.diagnostics,
            };
        }

        ParsedExpression {
            tree: root.map(|root| ExpressionTree::new(self.nodes, root)),
            diagnostics: self.diagnostics,
        }
    }

    fn parse_precedence(&mut self, minimum_binding_power: u8) -> Option<ExprId> {
        let mut left = self.parse_prefix()?;

        while let Some((left_binding_power, right_binding_power, operator)) =
            infix_binding_power(self.current().kind())
        {
            if left_binding_power < minimum_binding_power {
                break;
            }

            self.advance();
            let right = self.parse_precedence(right_binding_power)?;
            let span = self.node(left).span().through(self.node(right).span());
            left = self.push_node(
                ExprKind::Binary {
                    left,
                    operator,
                    right,
                },
                span,
            );
        }

        Some(left)
    }

    fn parse_prefix(&mut self) -> Option<ExprId> {
        let token = self.current();

        match token.kind() {
            TokenKind::Integer => {
                self.advance();
                Some(self.push_node(ExprKind::Integer, token.span()))
            }
            TokenKind::Identifier => {
                self.advance();
                Some(self.push_node(ExprKind::Name, token.span()))
            }
            TokenKind::Plus | TokenKind::Minus => {
                self.advance();
                let operator = match token.kind() {
                    TokenKind::Plus => UnaryOperator::Positive,
                    TokenKind::Minus => UnaryOperator::Negate,
                    _ => unreachable!(),
                };
                let operand = self.parse_precedence(PREFIX_BINDING_POWER)?;
                let span = token.span().through(self.node(operand).span());
                Some(self.push_node(ExprKind::Unary { operator, operand }, span))
            }
            TokenKind::LeftParen => {
                self.advance();
                let expression = self.parse_precedence(0)?;
                if self.current().kind() != TokenKind::RightParen {
                    self.diagnostics.push(Diagnostic::new(
                        "PARSE002",
                        "expected `)`",
                        self.current().span(),
                    ));
                    return None;
                }

                let right_paren = self.advance();
                let span = token.span().through(right_paren.span());
                Some(self.push_node(ExprKind::Group { expression }, span))
            }
            _ => {
                self.diagnostics.push(Diagnostic::new(
                    "PARSE001",
                    "expected expression",
                    token.span(),
                ));
                None
            }
        }
    }

    fn push_node(&mut self, kind: ExprKind, span: Span) -> ExprId {
        let id = ExprId::from_index(self.nodes.len());
        self.nodes.push(ExprNode::new(kind, span));
        id
    }

    fn node(&self, id: ExprId) -> ExprNode {
        self.nodes[id.index()]
    }

    fn current(&self) -> Token {
        self.tokens[self.current]
    }

    fn advance(&mut self) -> Token {
        let token = self.current();
        if token.kind() != TokenKind::Eof {
            self.current += 1;
        }
        token
    }
}

fn infix_binding_power(kind: TokenKind) -> Option<(u8, u8, BinaryOperator)> {
    match kind {
        TokenKind::Plus => Some((1, 2, BinaryOperator::Add)),
        TokenKind::Minus => Some((1, 2, BinaryOperator::Subtract)),
        TokenKind::Star => Some((3, 4, BinaryOperator::Multiply)),
        TokenKind::Slash => Some((3, 4, BinaryOperator::Divide)),
        _ => None,
    }
}
