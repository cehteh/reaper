use crate::tokenizer::{Token, TokenKind};
use std::collections::VecDeque;

#[derive(Debug)]
pub enum Expression {
    Literal(LiteralExpression),
    Variable(VariableExpression),
    Binary(BinaryExpression),
    Call(CallExpression),
}

#[derive(Debug)]
pub struct LiteralExpression {
    pub value: Literal,
}

#[derive(Debug)]
pub enum Literal {
    Num(f64),
    Bool(bool),
    Null,
}

#[derive(Debug)]
pub struct VariableExpression {
    pub value: String,
}

#[derive(Debug)]
pub struct BinaryExpression {
    pub kind: BinaryExpressionKind,
    pub lhs: Box<Expression>,
    pub rhs: Box<Expression>,
}

#[derive(Debug)]
pub enum BinaryExpressionKind {
    Add,
    Sub,
    Mul,
    Div,
    Less,
}

#[derive(Debug)]
pub struct CallExpression {
    pub variable: String,
    pub arguments: Vec<Expression>,
}

#[derive(Debug)]
pub enum Statement {
    Dummy,
    Print(PrintStatement),
    Fn(FnStatement),
    Expression(ExpressionStatement),
    Return(ReturnStatement),
    If(IfStatement),
    Block(BlockStatement),
}

#[derive(Debug)]
pub struct PrintStatement {
    pub expression: Expression,
}

#[derive(Debug)]
pub struct FnStatement {
    pub name: String,
    pub arguments: Vec<String>,
    pub body: Vec<Statement>,
}

#[derive(Debug)]
pub struct ExpressionStatement {
    pub expression: Expression,
}

#[derive(Debug)]
pub struct ReturnStatement {
    pub expression: Expression,
}

#[derive(Debug)]
pub struct IfStatement {
    pub condition: Expression,
    pub if_branch: Box<Statement>,
    pub else_branch: Box<Statement>,
}

#[derive(Debug)]
pub struct BlockStatement {
    pub body: Vec<Statement>,
}

pub struct Parser {
    current: Option<Token>,
    previous: Option<Token>,
    tokens: VecDeque<Token>,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            current: None,
            previous: None,
            tokens: VecDeque::new(),
        }
    }

    pub fn parse(&mut self, tokens: VecDeque<Token>) -> Vec<Statement> {
        self.tokens = tokens;
        self.advance();
        let mut statements = vec![];
        while self.current.is_some() {
            statements.push(self.parse_statement());
        }
        statements
    }

    fn is_next(&mut self, tokens: &[TokenKind]) -> bool {
        for token in tokens {
            if self.check(*token) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, kind: TokenKind) -> bool {
        self.current.clone().unwrap().kind == kind
    }

    fn advance(&mut self) {
        self.previous = self.current.clone();
        self.current = self.tokens.pop_front();
    }

    fn parse_statement(&mut self) -> Statement {
        if self.is_next(&[TokenKind::Print]) {
            self.parse_print_statement()
        } else if self.is_next(&[TokenKind::Fn]) {
            self.parse_fn_statement()
        } else if self.is_next(&[TokenKind::If]) {
            self.parse_if_statement()
        } else if self.is_next(&[TokenKind::LeftBrace]) {
            self.parse_block_statement()
        } else if self.is_next(&[TokenKind::Return]) {
            self.parse_return_statement()
        } else {
            self.parse_expression_statement()
        }
    }

    fn parse_return_statement(&mut self) -> Statement {
        let expression = self.parse_expression();
        self.consume(TokenKind::Semicolon);
        Statement::Return(ReturnStatement { expression })
    }

    fn parse_block_statement(&mut self) -> Statement {
        let mut body = vec![];
        while !self.is_next(&[TokenKind::RightBrace]) {
            body.push(self.parse_statement());
        }
        Statement::Block(BlockStatement { body })
    }

    fn parse_if_statement(&mut self) -> Statement {
        self.consume(TokenKind::LeftParen);
        let condition = self.parse_expression();
        self.consume(TokenKind::RightParen);
        let if_branch = self.parse_statement();
        let else_branch: Statement;
        if self.is_next(&[TokenKind::Else]) {
            else_branch = self.parse_statement();
        } else {
            else_branch = Statement::Dummy;
        }
        Statement::If(IfStatement {
            condition,
            if_branch: Box::new(if_branch),
            else_branch: Box::new(else_branch),
        })
    }

    fn parse_expression_statement(&mut self) -> Statement {
        let expr = self.parse_expression();
        self.consume(TokenKind::Semicolon);
        Statement::Expression(ExpressionStatement { expression: expr })
    }

    fn parse_print_statement(&mut self) -> Statement {
        self.consume(TokenKind::Print);
        let expression = self.parse_expression();
        self.consume(TokenKind::Semicolon);
        Statement::Print(PrintStatement { expression })
    }

    fn parse_fn_statement(&mut self) -> Statement {
        self.consume(TokenKind::Fn);
        let name = self.consume(TokenKind::Identifier).unwrap();
        self.consume(TokenKind::LeftParen);
        let mut arguments = vec![];
        while !self.is_next(&[TokenKind::RightParen]) {
            let arg = self.consume(TokenKind::Identifier).unwrap();
            self.consume(TokenKind::Comma);
            arguments.push(arg.value);
        }
        self.consume(TokenKind::LeftBrace);
        let mut body = vec![];
        while !self.is_next(&[TokenKind::RightBrace]) {
            body.push(self.parse_statement());
        }
        Statement::Fn(FnStatement {
            name: name.value,
            arguments,
            body,
        })
    }

    fn consume(&mut self, kind: TokenKind) -> Option<Token> {
        if self.check(kind) {
            let token = self.current.clone();
            self.advance();
            return token;
        }
        None
    }

    fn parse_expression(&mut self) -> Expression {
        self.relational()
    }

    fn relational(&mut self) -> Expression {
        let mut result = self.term();
        while self.is_next(&[TokenKind::Less]) {
            let kind = match self.previous.clone() {
                Some(token) => match token.kind {
                    TokenKind::Less => BinaryExpressionKind::Less,
                    _ => unreachable!(),
                },
                None => unreachable!(),
            };
            result = Expression::Binary(BinaryExpression {
                kind,
                lhs: Box::new(result),
                rhs: Box::new(self.term()),
            });
        }
        result
    }

    fn term(&mut self) -> Expression {
        let mut result = self.factor();
        while self.is_next(&[TokenKind::Plus, TokenKind::Minus]) {
            let kind = match self.previous.clone() {
                Some(token) => match token.kind {
                    TokenKind::Plus => BinaryExpressionKind::Add,
                    TokenKind::Minus => BinaryExpressionKind::Sub,
                    _ => unreachable!(),
                },
                None => unreachable!(),
            };
            result = Expression::Binary(BinaryExpression {
                kind,
                lhs: Box::new(result),
                rhs: Box::new(self.factor()),
            });
        }
        result
    }

    fn factor(&mut self) -> Expression {
        let mut result = self.call();
        while self.is_next(&[TokenKind::Star, TokenKind::Slash]) {
            let kind = match self.previous.clone() {
                Some(token) => match token.kind {
                    TokenKind::Star => BinaryExpressionKind::Mul,
                    TokenKind::Slash => BinaryExpressionKind::Div,
                    _ => unreachable!(),
                },
                None => unreachable!(),
            };
            result = Expression::Binary(BinaryExpression {
                kind,
                lhs: Box::new(result),
                rhs: Box::new(self.call()),
            });
        }
        result
    }

    fn call(&mut self) -> Expression {
        let mut expr = self.primary();
        if self.is_next(&[TokenKind::LeftParen]) {
            let mut arguments = vec![];
            if !self.check(TokenKind::RightParen) {
                loop {
                    arguments.push(self.parse_expression());
                    if !self.is_next(&[TokenKind::Comma]) {
                        break;
                    }
                }
            }
            self.consume(TokenKind::RightParen);
            let name = match expr {
                Expression::Variable(v) => v.value,
                _ => unimplemented!(),
            };
            expr = Expression::Call(CallExpression {
                variable: name,
                arguments,
            });
        }
        expr
    }

    fn primary(&mut self) -> Expression {
        if self.is_next(&[TokenKind::Number]) {
            let n = self.previous.clone().unwrap().value.parse().unwrap();
            Expression::Literal(LiteralExpression {
                value: Literal::Num(n),
            })
        } else if self.is_next(&[TokenKind::Identifier]) {
            let var = self.previous.clone().unwrap().value;
            Expression::Variable(VariableExpression { value: var })
        } else {
            // println!("current is: {:?}", self.current.clone());
            unimplemented!();
        }
    }
}
