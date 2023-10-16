use crate::parser::{
    Statement,
    Expression,
    CallExpression,
    LiteralExpression,
    Literal,
    BinaryExpression,
    BinaryExpressionKind,
    VariableExpression,
};

pub struct Compiler {
    bytecode: Vec<Opcode>,
    functions: std::collections::HashMap<String, usize>,
    locals: Vec<String>,
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler { 
            bytecode: Vec::new(),
            functions: std::collections::HashMap::new(),
            locals: Vec::new(),
        }
    }

    pub fn compile(&mut self, ast: Vec<Statement>) -> Vec<Opcode> {
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
pub enum Opcode {
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