use crate::{
    BinaryOperator, BlockId, BlockNode, Diagnostic, ExprId, ExprKind, ExprNode, ExpressionTree,
    FunctionId, FunctionNode, Lexed, Parameter, ProgramItem, ProgramTree, Span, StmtId, StmtKind,
    StmtNode, Token, TokenKind, UnaryOperator,
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

/// The syntax result for a full source unit containing statements and functions.
///
/// Parsing continues across deterministic recovery boundaries to collect diagnostics, but a tree is
/// published only when the complete source unit is syntactically valid.
#[derive(Debug, PartialEq, Eq)]
pub struct ParsedProgram {
    tree: Option<ProgramTree>,
    diagnostics: Vec<Diagnostic>,
}

impl ParsedProgram {
    #[must_use]
    pub fn tree(&self) -> Option<&ProgramTree> {
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
    Parser::new(lexed.tokens()).run_expression()
}

/// Parses the complete lexical token stream as a source unit of functions and statements.
///
/// Function declarations are accepted only at the top level. Statement recovery synchronizes at a
/// semicolon, a closing brace, or an unambiguous next-statement starter. Malformed function
/// signatures synchronize at their body or the next top-level function.
pub fn parse_program(lexed: &Lexed<'_>) -> ParsedProgram {
    Parser::new(lexed.tokens()).run_program()
}

struct Parser<'tokens> {
    tokens: &'tokens [Token],
    current: usize,
    expressions: Vec<ExprNode>,
    statements: Vec<StmtNode>,
    blocks: Vec<BlockNode>,
    block_statements: Vec<StmtId>,
    parameters: Vec<Parameter>,
    functions: Vec<FunctionNode>,
    items: Vec<ProgramItem>,
    diagnostics: Vec<Diagnostic>,
}

impl<'tokens> Parser<'tokens> {
    fn new(tokens: &'tokens [Token]) -> Self {
        Self {
            tokens,
            current: 0,
            expressions: Vec::new(),
            statements: Vec::new(),
            blocks: Vec::new(),
            block_statements: Vec::new(),
            parameters: Vec::new(),
            functions: Vec::new(),
            items: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    fn run_expression(mut self) -> ParsedExpression {
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
            tree: root.map(|root| ExpressionTree::new(self.expressions, root)),
            diagnostics: self.diagnostics,
        }
    }

    fn run_program(mut self) -> ParsedProgram {
        while self.current().kind() != TokenKind::Eof {
            if self.current().kind() == TokenKind::RightBrace {
                let token = self.advance();
                self.diagnostics.push(Diagnostic::new(
                    "PARSE114",
                    "unexpected `}`",
                    token.span(),
                ));
                continue;
            }

            if self.current().kind() == TokenKind::Fn {
                if let Some(function) = self.parse_function() {
                    self.items.push(ProgramItem::Function(function));
                } else {
                    self.recover_function();
                }
                continue;
            }

            if let Some(statement) = self.parse_statement() {
                self.items.push(ProgramItem::Statement(statement));
            } else {
                self.recover_statement();
            }
        }

        if !self.diagnostics.is_empty() {
            return ParsedProgram {
                tree: None,
                diagnostics: self.diagnostics,
            };
        }

        ParsedProgram {
            tree: Some(ProgramTree::new(
                self.expressions,
                self.statements,
                self.blocks,
                self.block_statements,
                self.parameters,
                self.functions,
                self.items,
            )),
            diagnostics: self.diagnostics,
        }
    }

    fn parse_function(&mut self) -> Option<FunctionId> {
        let function_token = self.advance();
        let name = self
            .expect(
                TokenKind::Identifier,
                "PARSE104",
                "expected function name",
            )?
            .span();
        self.expect(TokenKind::LeftParen, "PARSE105", "expected `(`")?;

        let parameter_start = self.parameters.len();
        if self.current().kind() != TokenKind::RightParen {
            loop {
                let parameter_name = self
                    .expect(
                        TokenKind::Identifier,
                        "PARSE106",
                        "expected parameter name",
                    )?
                    .span();
                self.expect(TokenKind::Colon, "PARSE107", "expected `:`")?;
                let type_name = self
                    .expect(
                        TokenKind::Identifier,
                        "PARSE108",
                        "expected parameter type",
                    )?
                    .span();
                self.parameters
                    .push(Parameter::new(parameter_name, type_name));

                if self.current().kind() != TokenKind::Comma {
                    break;
                }
                self.advance();
                if self.current().kind() == TokenKind::RightParen {
                    break;
                }
            }
        }

        self.expect(TokenKind::RightParen, "PARSE109", "expected `)`")?;
        let return_type = if self.current().kind() == TokenKind::Arrow {
            self.advance();
            Some(
                self.expect(
                    TokenKind::Identifier,
                    "PARSE110",
                    "expected return type",
                )?
                .span(),
            )
        } else {
            None
        };

        if self.current().kind() != TokenKind::LeftBrace {
            self.diagnostics.push(Diagnostic::new(
                "PARSE111",
                "expected `{`",
                self.current().span(),
            ));
            return None;
        }

        let body = self.parse_block()?;
        let span = function_token.span().through(self.block_node(body).span());
        let parameter_count = self.parameters.len() - parameter_start;
        Some(self.push_function(FunctionNode::new(
            name,
            parameter_start,
            parameter_count,
            return_type,
            body,
            span,
        )))
    }

    fn parse_statement(&mut self) -> Option<StmtId> {
        match self.current().kind() {
            TokenKind::Let => self.parse_let_statement(),
            TokenKind::Return => self.parse_return_statement(),
            TokenKind::LeftBrace => {
                let block = self.parse_block()?;
                let span = self.block_node(block).span();
                Some(self.push_statement(StmtKind::Block { block }, span))
            }
            TokenKind::Fn => {
                let token = self.advance();
                self.diagnostics.push(Diagnostic::new(
                    "PARSE113",
                    "function declarations are only allowed at top level",
                    token.span(),
                ));
                None
            }
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_let_statement(&mut self) -> Option<StmtId> {
        let let_token = self.advance();
        let name = self
            .expect(
                TokenKind::Identifier,
                "PARSE101",
                "expected binding name",
            )?
            .span();
        self.expect(TokenKind::Equal, "PARSE102", "expected `=`")?;
        let initializer = self.parse_precedence(0)?;
        let semicolon = self.expect(TokenKind::Semicolon, "PARSE103", "expected `;`")?;
        let span = let_token.span().through(semicolon.span());
        Some(self.push_statement(StmtKind::Let { name, initializer }, span))
    }

    fn parse_return_statement(&mut self) -> Option<StmtId> {
        let return_token = self.advance();
        let value = if self.current().kind() == TokenKind::Semicolon {
            None
        } else {
            Some(self.parse_precedence(0)?)
        };
        let semicolon = self.expect(TokenKind::Semicolon, "PARSE103", "expected `;`")?;
        let span = return_token.span().through(semicolon.span());
        Some(self.push_statement(StmtKind::Return { value }, span))
    }

    fn parse_expression_statement(&mut self) -> Option<StmtId> {
        let expression = self.parse_precedence(0)?;
        let semicolon = self.expect(TokenKind::Semicolon, "PARSE103", "expected `;`")?;
        let span = self
            .expression_node(expression)
            .span()
            .through(semicolon.span());
        Some(self.push_statement(StmtKind::Expression { expression }, span))
    }

    fn parse_block(&mut self) -> Option<BlockId> {
        let left_brace = self.expect(TokenKind::LeftBrace, "PARSE111", "expected `{`")?;
        let mut statements = Vec::new();

        while !matches!(self.current().kind(), TokenKind::RightBrace | TokenKind::Eof) {
            if let Some(statement) = self.parse_statement() {
                statements.push(statement);
            } else {
                self.recover_statement();
            }
        }

        if self.current().kind() != TokenKind::RightBrace {
            self.diagnostics.push(Diagnostic::new(
                "PARSE112",
                "expected `}`",
                self.current().span(),
            ));
            return None;
        }

        let right_brace = self.advance();
        let statement_start = self.block_statements.len();
        let statement_count = statements.len();
        self.block_statements.extend(statements);
        let span = left_brace.span().through(right_brace.span());
        Some(self.push_block(BlockNode::new(
            statement_start,
            statement_count,
            span,
        )))
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
            let span = self
                .expression_node(left)
                .span()
                .through(self.expression_node(right).span());
            left = self.push_expression(
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
                Some(self.push_expression(ExprKind::Integer, token.span()))
            }
            TokenKind::Identifier => {
                self.advance();
                Some(self.push_expression(ExprKind::Name, token.span()))
            }
            TokenKind::Plus | TokenKind::Minus => {
                self.advance();
                let operator = match token.kind() {
                    TokenKind::Plus => UnaryOperator::Positive,
                    TokenKind::Minus => UnaryOperator::Negate,
                    _ => unreachable!(),
                };
                let operand = self.parse_precedence(PREFIX_BINDING_POWER)?;
                let span = token
                    .span()
                    .through(self.expression_node(operand).span());
                Some(self.push_expression(ExprKind::Unary { operator, operand }, span))
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
                Some(self.push_expression(ExprKind::Group { expression }, span))
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

    fn recover_statement(&mut self) {
        while !matches!(self.current().kind(), TokenKind::Eof | TokenKind::RightBrace) {
            match self.current().kind() {
                TokenKind::Semicolon => {
                    self.advance();
                    break;
                }
                TokenKind::Let | TokenKind::Return | TokenKind::LeftBrace | TokenKind::Fn => break,
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn recover_function(&mut self) {
        while self.current().kind() != TokenKind::Eof {
            match self.current().kind() {
                TokenKind::Fn => break,
                TokenKind::Semicolon => {
                    self.advance();
                    break;
                }
                TokenKind::LeftBrace => {
                    self.skip_balanced_block();
                    break;
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn skip_balanced_block(&mut self) {
        let mut depth = 0_u32;
        while self.current().kind() != TokenKind::Eof {
            match self.current().kind() {
                TokenKind::LeftBrace => {
                    depth += 1;
                    self.advance();
                }
                TokenKind::RightBrace => {
                    self.advance();
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn expect(
        &mut self,
        kind: TokenKind,
        code: &'static str,
        message: &'static str,
    ) -> Option<Token> {
        if self.current().kind() == kind {
            return Some(self.advance());
        }

        self.diagnostics
            .push(Diagnostic::new(code, message, self.current().span()));
        None
    }

    fn push_expression(&mut self, kind: ExprKind, span: Span) -> ExprId {
        let id = ExprId::from_index(self.expressions.len());
        self.expressions.push(ExprNode::new(kind, span));
        id
    }

    fn push_statement(&mut self, kind: StmtKind, span: Span) -> StmtId {
        let id = StmtId::from_index(self.statements.len());
        self.statements.push(StmtNode::new(kind, span));
        id
    }

    fn push_block(&mut self, block: BlockNode) -> BlockId {
        let id = BlockId::from_index(self.blocks.len());
        self.blocks.push(block);
        id
    }

    fn push_function(&mut self, function: FunctionNode) -> FunctionId {
        let id = FunctionId::from_index(self.functions.len());
        self.functions.push(function);
        id
    }

    fn expression_node(&self, id: ExprId) -> ExprNode {
        self.expressions[id.index()]
    }

    fn block_node(&self, id: BlockId) -> BlockNode {
        self.blocks[id.index()]
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
