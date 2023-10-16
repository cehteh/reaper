use std::collections::VecDeque;

use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq)]
enum TokenKind {
    Print,
    Fn,
    If,
    Else,
    Identifier,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Number,
    Plus,
    Minus,
    Star,
    Slash,
    Comma,
    Semicolon,
    Less,
    Return,
}

#[derive(Debug, Clone)]
struct Token {
    kind: TokenKind,
    value: String,
}

struct Tokenizer<'a> {
    src: &'a str,
    start: usize,
}

impl Iterator for Tokenizer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let re_keyword = r"?P<keyword>print|fn|if|else|return";
        let re_identifier = r"?P<identifier>[a-zA-Z_][a-zA-Z0-9_]*";
        let re_individual = r"?P<individual>[-+*/(){};,<]";
        let re_number = r"?P<number>[-+]?\d+(\.\d+)?";

        let r = Regex::new(
            format!(
                "({})|({})|({})|({})",
                re_keyword, re_identifier, re_individual, re_number,
            )
            .as_str(),
        )
        .unwrap();

        let token = match r.captures_at(self.src, self.start) {
            Some(captures) => {
                if let Some(m) = captures.name("keyword") {
                    self.start = m.end();
                    match m.as_str() {
                        "print" => self.make_token(TokenKind::Print, String::from("print")),
                        "fn" => self.make_token(TokenKind::Fn, String::from("fn")),
                        "if" => self.make_token(TokenKind::If, "if".to_string()),
                        "else" => self.make_token(TokenKind::Else, "else".to_string()),
                        "return" => self.make_token(TokenKind::Return, "return".to_string()),
                        _ => unreachable!(),
                    }
                } else if let Some(m) = captures.name("identifier") {
                    self.start = m.end();
                    self.make_token(TokenKind::Identifier, m.as_str().to_string())
                } else if let Some(m) = captures.name("individual") {
                    self.start = m.end();
                    match m.as_str() {
                        "(" => self.make_token(TokenKind::LeftParen, "(".to_string()),
                        ")" => self.make_token(TokenKind::RightParen, ")".to_string()),
                        "{" => self.make_token(TokenKind::LeftBrace, "{".to_string()),
                        "}" => self.make_token(TokenKind::RightBrace, "}".to_string()),
                        "+" => self.make_token(TokenKind::Plus, "+".to_string()),
                        "-" => self.make_token(TokenKind::Minus, "-".to_string()),
                        "*" => self.make_token(TokenKind::Star, "*".to_string()),
                        "/" => self.make_token(TokenKind::Slash, "/".to_string()),
                        ";" => self.make_token(TokenKind::Semicolon, ";".to_string()),
                        "," => self.make_token(TokenKind::Comma, ",".to_string()),
                        "<" => self.make_token(TokenKind::Less, ",".to_string()),
                        _ => unreachable!(),
                    }
                } else if let Some(m) = captures.name("number") {
                    self.start = m.end();
                    self.make_token(TokenKind::Number, m.as_str().to_string())
                } else {
                    return None;
                }
            }
            None => return None,
        };

        Some(token)
    }
}

impl<'a> Tokenizer<'a> {
    fn new(src: &'a str) -> Tokenizer<'a> {
        Tokenizer { src, start: 0 }
    }

    fn make_token(&self, kind: TokenKind, value: String) -> Token {
        Token { kind, value }
    }
}

#[derive(Debug)]
struct BlockStatement {
    body: Vec<Statement>,
}


#[derive(Debug)]
struct IfStatement {
    condition: Expression,
    if_branch: Box<Statement>,
    else_branch: Box<Statement>,
}

#[derive(Debug)]
struct ExpressionStatement {
    expression: Expression,
}

#[derive(Debug)]
struct PrintStatement {
    expression: Expression,
}

#[derive(Debug)]
struct FnStatement {
    name: String,
    arguments: Vec<String>,
    body: Vec<Statement>,
}

#[derive(Debug)]
struct ReturnStatement {
    expression: Expression,
}

#[derive(Debug)]
enum Statement {
    Dummy,
    Print(PrintStatement),
    Fn(FnStatement),
    Expression(ExpressionStatement),
    Return(ReturnStatement),
    If(IfStatement),
    Block(BlockStatement),
}

struct Parser {
    current: Option<Token>,
    previous: Option<Token>,
    tokens: VecDeque<Token>,
}

impl Parser {
    fn new() -> Parser {
        Parser {
            current: None,
            previous: None,
            tokens: VecDeque::new(),
        }
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

    fn parse(&mut self, tokens: VecDeque<Token>) -> Vec<Statement> {
        self.tokens = tokens;
        self.advance();
        let mut statements = vec![];
        while !self.current.is_none() {
            statements.push(self.parse_statement());
        }
        statements
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
        Statement::If(IfStatement { condition, if_branch: Box::new(if_branch), else_branch: Box::new(else_branch) })
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
            expr = Expression::Call(CallExpression { variable: name, arguments });
        }
        expr
    }

    fn primary(&mut self) -> Expression {
        if self.is_next(&[TokenKind::Number]) {
            let n = self.previous.clone().unwrap().value.parse().unwrap();
            Expression::Literal(LiteralExpression { value: Literal::Num(n) })
        } else if self.is_next(&[TokenKind::Identifier]) {
            let var = self.previous.clone().unwrap().value;
            Expression::Variable(VariableExpression { value: var })
        } else {
            // println!("current is: {:?}", self.current.clone());
            unimplemented!();
        }
    }
}

#[derive(Debug)]
enum Expression {
    Literal(LiteralExpression),
    Variable(VariableExpression),
    Binary(BinaryExpression),
    Call(CallExpression),
}

#[derive(Debug)]
enum BinaryExpressionKind {
    Add,
    Sub,
    Mul,
    Div,
    Less,
}

#[derive(Debug)]
struct CallExpression {
    variable: String,
    arguments: Vec<Expression>,
}

#[derive(Debug)]
struct BinaryExpression {
    kind: BinaryExpressionKind,
    lhs: Box<Expression>,
    rhs: Box<Expression>,
}

#[derive(Debug)]
enum Literal {
    Num(f64),
    Bool(bool),
    Null,
}

#[derive(Debug)]
struct LiteralExpression {
    value: Literal,
}

#[derive(Debug)]
struct VariableExpression {
    value: String,
}

struct Compiler {
    bytecode: Vec<Opcode>,
    functions: std::collections::HashMap<String, usize>,
    locals: Vec<String>,
}

impl Compiler {
    fn new() -> Compiler {
        Compiler { 
            bytecode: Vec::new(),
            functions: std::collections::HashMap::new(),
            locals: Vec::new(),
        }
    }

    fn compile(&mut self, ast: Vec<Statement>) -> Vec<Opcode> {
        for statement in ast {
            statement.codegen(self);
        }
        self.bytecode.clone()
    }

    fn emit_bytes(&mut self, opcodes: &[Opcode]) -> usize {
        for opcode in opcodes {
            self.bytecode.push(*opcode);
        }
        self.bytecode.len() - opcodes.len()
    }

    fn resolve_local(&self, name: String) -> Option<usize> {
        for (idx, local) in self.locals.iter().enumerate() {
            if *local == name {
                return Some(idx);
            }
        }
        None
    }
}

#[derive(Debug, Clone, Copy)]
enum Opcode {
    Print,
    Const(f64),
    Add,
    Sub,
    Mul,
    Div,
    Null,
    Not,
    False,
    Jmp(i64),
    DirectJmp(usize),
    Jz(i64),
    Ip(usize),
    Ret,
    Less,
    Deepget(usize),
    Deepset(usize),
    IncFpcount,
    Pop,
}

trait Codegen {
    fn codegen(&self, compiler: &mut Compiler) {}
}

impl Codegen for Statement {
    fn codegen(&self, compiler: &mut Compiler) {
        match self {
            Statement::Print(print_statement) => {
                print_statement.expression.codegen(compiler);
                compiler.emit_bytes(&[Opcode::Print]);
            }
            Statement::Fn(fn_statement) => {
                let jmp_idx = compiler.emit_bytes(&[Opcode::Jmp(0xFFFF)]);
                let start = compiler.bytecode.len();
                compiler.functions.insert(fn_statement.name.clone(), jmp_idx);
                // println!("Compiler.functions: {:?}", compiler.functions);
                for argument in &fn_statement.arguments {
                    compiler.locals.push(argument.clone());
                }
                for statement in &fn_statement.body {
                    statement.codegen(compiler);
                }
                compiler.emit_bytes(&[Opcode::Null, Opcode::Ret]);
                let fn_size = compiler.bytecode.len() - start;
                compiler.bytecode[jmp_idx] = Opcode::Jmp(fn_size as i64);
            }
            Statement::Expression(expr_statement) => {
                match &expr_statement.expression {
                    Expression::Call(call_expr) => call_expr.codegen(compiler),
                    _ => {}
                }
            }
            Statement::Return(return_statement) => {
                return_statement.expression.codegen(compiler);
                let mut deepset_no = compiler.locals.len();
                while deepset_no > 0 {
                    compiler.emit_bytes(&[Opcode::Deepset(deepset_no)]);
                    deepset_no -= 1;
                }
                compiler.emit_bytes(&[Opcode::Ret]);
            }
            Statement::If(if_statement) => {
                if_statement.condition.codegen(compiler);
                let jz_idx = compiler.emit_bytes(&[Opcode::Jz(0xFFFF)]);
                let start = compiler.bytecode.len();
                if_statement.if_branch.codegen(compiler);
                let if_branch_size = compiler.bytecode.len() - start;
                compiler.bytecode[jz_idx] = Opcode::Jz(if_branch_size as i64);
                let else_idx = compiler.emit_bytes(&[Opcode::Jmp(0xFFFF)]);
                let start_else = compiler.bytecode.len();
                if_statement.else_branch.codegen(compiler);
                let else_branch_size = compiler.bytecode.len() - start_else;
                compiler.bytecode[else_idx] = Opcode::Jmp(else_branch_size as i64);
            }
            Statement::Block(block_statement) => {
                for statement in &block_statement.body {
                    statement.codegen(compiler);
                }
            }
            _ => {}
        }
    }
}

impl Codegen for Expression {
    fn codegen(&self, compiler: &mut Compiler) {
        match self {
            Expression::Binary(binexp) => binexp.codegen(compiler),
            Expression::Literal(literal) => literal.codegen(compiler),
            Expression::Variable(variable) => variable.codegen(compiler),
            Expression::Call(call) => call.codegen(compiler),
        }
    }
}

impl Codegen for CallExpression {
    fn codegen(&self, compiler: &mut Compiler) {
        let ip_idx = compiler.emit_bytes(&[Opcode::Ip(0xFFFF)]);
        let start = compiler.bytecode.len();
        for argument in &self.arguments {
            argument.codegen(compiler);
        }
        compiler.emit_bytes(&[Opcode::IncFpcount]);
        let jmp_addr = compiler.functions.get(&self.variable).unwrap();
        compiler.emit_bytes(&[Opcode::DirectJmp(*jmp_addr)]);
        compiler.bytecode[ip_idx] = Opcode::Ip(compiler.bytecode.len() - start);
    }
}

impl Codegen for BinaryExpression {
    fn codegen(&self, compiler: &mut Compiler) {
        self.lhs.codegen(compiler);
        self.rhs.codegen(compiler);
        match self.kind {
            BinaryExpressionKind::Add => { compiler.emit_bytes(&[Opcode::Add]); }
            BinaryExpressionKind::Sub => { compiler.emit_bytes(&[Opcode::Sub]); }
            BinaryExpressionKind::Mul => { compiler.emit_bytes(&[Opcode::Mul]); }
            BinaryExpressionKind::Div => { compiler.emit_bytes(&[Opcode::Div]); }
            BinaryExpressionKind::Less => { compiler.emit_bytes(&[Opcode::Less]); }
        }
    }
}

impl Codegen for LiteralExpression {
    fn codegen(&self, compiler: &mut Compiler) {
        match self.value {
            Literal::Num(n) => {
                compiler.emit_bytes(&[Opcode::Const(n)]);
            }
            Literal::Bool(b) => match b {
                true => {
                    compiler.emit_bytes(&[Opcode::False, Opcode::Not]);
                }
                false => {
                    compiler.emit_bytes(&[Opcode::False]);
                }
            }
            Literal::Null => {
                compiler.emit_bytes(&[Opcode::Null]);
            }
        }
    }
}

impl Codegen for VariableExpression {
    fn codegen(&self, compiler: &mut Compiler) {
        let idx = compiler.resolve_local(self.value.clone()).unwrap();
        compiler.emit_bytes(&[Opcode::Deepget(idx+1)]);
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Object {
    Number(f64),
    Bool(bool),
    Null,
    Ptr(i64),
}

impl std::ops::Add for Object {
    type Output = Object;
    
    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Object::Number(a), Object::Number(b)) => Object::Number(a + b),
            _ => unimplemented!()
        }
    }
}

impl std::ops::Sub for Object {
    type Output = Object;
    
    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Object::Number(a), Object::Number(b)) => Object::Number(a - b),
            _ => unimplemented!()
        }
    }
}

impl std::ops::Mul for Object {
    type Output = Object;
    
    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Object::Number(a), Object::Number(b)) => Object::Number(a * b),
            _ => unimplemented!()
        }
    }
}

impl std::ops::Div for Object {
    type Output = Object;
    
    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Object::Number(a), Object::Number(b)) => Object::Number(a / b),
            _ => unimplemented!()
        }
    }
}

impl std::ops::Not for Object {
    type Output = Object;

    fn not(self) -> Self::Output {
        match self {
            Object::Bool(b) => match b {
                true => Object::Bool(false),
                false => Object::Bool(true),
            }
            _ => unimplemented!(),
        }
    }    
}

impl std::cmp::PartialOrd for Object {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Object::Number(a), Object::Number(b)) => a.partial_cmp(b),
            _ => unimplemented!(),
        }
    }
}

fn adjust_idx(fp_stack: &[usize], idx: usize, fp_count: usize) -> isize {
    let fp = fp_stack[fp_count-1];
    // println!("fp_stack is: {:?}", fp_stack);
    // println!("fp_count is: {}", fp_count);
    let adjustment = if fp_count == 0 { -1 } else { 0 };
    fp as isize + idx as isize  + adjustment 
}

fn main() {
    let src = r"
        fn fib(n) { 
            if (n < 2) return n;
            return fib(n-1)+fib(n-2);
        }
        
        print fib(40);";
    let tokenizer = Tokenizer::new(src);
    let mut parser = Parser::new();
    let mut compiler = Compiler::new();
    let ast = parser.parse(tokenizer.into_iter().collect());
    let bytecode = compiler.compile(ast);
    // println!("{:?}", bytecode);
    let mut stack = Vec::new();
    let mut ip: i64 = 0;
    let mut fp_count = 0;
    let mut fp_stack = [0; 1024];
    loop {
        println!("stack before current instruction: {:?}", stack);
        println!("current instruction: {:?}", bytecode[ip as usize]);

        match bytecode[ip as usize] {
            Opcode::Const(n) => {
                stack.push(Object::Number(n));
            }
            Opcode::Print => {
                let obj = stack.pop();
                if let Some(o) = obj {
                    println!("{:?}", o);
                }
            }
            Opcode::Add => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(a + b);
            }
            Opcode::Sub => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(a - b);
            }
            Opcode::Mul => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(a * b);
            }
            Opcode::Div => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(a / b);
            }
            Opcode::Less => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(Object::Bool(a < b));
            }
            Opcode::False => {
                stack.push(Object::Bool(false));
            }
            Opcode::Not => {
                let obj = stack.pop().unwrap();
                stack.push(!obj);
            }
            Opcode::Null => {
                stack.push(Object::Null);
            }
            Opcode::Jmp(offset) => {
                ip += offset;
            }
            Opcode::Jz(offset) => {
                let item = stack.pop().unwrap();
                match item {
                    Object::Bool(b) => match b {
                        true => {}
                        false => {
                            ip += offset;
                        }
                    }
                    _ => unimplemented!(),
                }
            }
            Opcode::Ip(offset) => {
                fp_stack[fp_count] = stack.len();
                stack.push(Object::Ptr(ip+offset as i64));
            }
            Opcode::DirectJmp(jmp_addr) => {
                ip = jmp_addr as i64;
            }
            Opcode::Ret => {
                let retvalue = stack.pop().unwrap();
                let retaddr = stack.pop().unwrap();
                fp_count -= 1;
                stack.push(retvalue);
                match retaddr {
                    Object::Ptr(ptr) => {
                        ip = ptr;
                        // println!("Back to ip: {}", ip);
                    }
                    _ => {}
                }
            }
            Opcode::Deepget(idx) => {
                let adjusted_idx = adjust_idx(&fp_stack, idx, fp_count) as usize;
                // println!("{:?}", adjusted_idx);
                let item = stack.get(adjusted_idx).unwrap();
                stack.push(item.clone());
            }
            Opcode::Deepset(idx) => {
                stack[adjust_idx(&fp_stack, idx, fp_count) as usize] = stack.pop().unwrap();
            }
            Opcode::IncFpcount => {
                fp_count += 1;
            }
            Opcode::Pop => {
                stack.pop();
            }
        }

        ip += 1;


        if ip as usize == bytecode.len() {
            break;
        }

    }
}
