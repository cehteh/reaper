use crate::parser::{
    AssignExpression, BinaryExpression, BinaryExpressionKind, BlockStatement, CallExpression,
    Expression, ExpressionStatement, FnStatement, IfStatement, Literal, LiteralExpression,
    PrintStatement, ReturnStatement, Statement, VariableExpression,
};

pub struct Compiler {
    bytecode: Vec<Opcode>,
    functions: std::collections::HashMap<String, usize>,
    arguments: Vec<String>,
    locals: Vec<String>,
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler {
            bytecode: Vec::new(),
            functions: std::collections::HashMap::new(),
            arguments: Vec::new(),
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
        self.locals.iter().position(|local| *local == name)
    }

    fn resolve_arg(&self, name: String) -> Option<usize> {
        self.arguments.iter().position(|arg| *arg == name)
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
    Jmp(isize),
    Jz(isize),
    Ip,
    Ret(usize, usize),
    Less,
    Deepget(usize),
    DeepgetReverse(usize),
    Deepset(usize),
    DeepsetReverse(usize),
    Pop,
}

trait Codegen {
    fn codegen(&self, compiler: &mut Compiler) {}
}

impl Codegen for Statement {
    fn codegen(&self, compiler: &mut Compiler) {
        match self {
            Statement::Print(print_statement) => print_statement.codegen(compiler),
            Statement::Fn(fn_statement) => fn_statement.codegen(compiler),
            Statement::Expression(expr_statement) => expr_statement.codegen(compiler),
            Statement::Return(return_statement) => return_statement.codegen(compiler),
            Statement::If(if_statement) => if_statement.codegen(compiler),
            Statement::Block(block_statement) => block_statement.codegen(compiler),
            _ => {}
        }
    }
}

impl Codegen for PrintStatement {
    fn codegen(&self, compiler: &mut Compiler) {
        self.expression.codegen(compiler);
        compiler.emit_bytes(&[Opcode::Print]);
    }
}

impl Codegen for FnStatement {
    fn codegen(&self, compiler: &mut Compiler) {
        let jmp_idx = compiler.emit_bytes(&[Opcode::Jmp(0xFFFF)]);

        compiler.functions.insert(self.name.clone(), jmp_idx);

        for argument in &self.arguments {
            compiler.arguments.push(argument.clone());
        }

        for statement in &self.body {
            statement.codegen(compiler);
        }

        compiler.emit_bytes(&[Opcode::Null, Opcode::Ret(compiler.arguments.len(), compiler.locals.len())]);

        compiler.bytecode[jmp_idx] = Opcode::Jmp(compiler.bytecode.len() as isize - 1);

        compiler.arguments.clear();
        compiler.locals.clear();
    }
}

impl Codegen for ExpressionStatement {
    fn codegen(&self, compiler: &mut Compiler) {
        match &self.expression {
            Expression::Call(call_expr) => {
                call_expr.codegen(compiler);
                compiler.emit_bytes(&[Opcode::Pop]);
            }
            Expression::Assign(assign_expr) => assign_expr.codegen(compiler),
            _ => unimplemented!(),
        }
    }
}

impl Codegen for ReturnStatement {
    fn codegen(&self, compiler: &mut Compiler) {
        self.expression.codegen(compiler);
        compiler.emit_bytes(&[Opcode::Ret(
            compiler.arguments.len(), compiler.locals.len(),
        )]);
    }
}

impl Codegen for IfStatement {
    fn codegen(&self, compiler: &mut Compiler) {
        self.condition.codegen(compiler);

        let jz_idx = compiler.emit_bytes(&[Opcode::Jz(0xFFFF)]);
        self.if_branch.codegen(compiler);
        compiler.bytecode[jz_idx] = Opcode::Jz(compiler.bytecode.len() as isize - 1);

        let else_idx = compiler.emit_bytes(&[Opcode::Jmp(0xFFFF)]);
        self.else_branch.codegen(compiler);
        compiler.bytecode[else_idx] = Opcode::Jmp(compiler.bytecode.len() as isize - 1);
    }
}

impl Codegen for BlockStatement {
    fn codegen(&self, compiler: &mut Compiler) {
        for statement in &self.body {
            statement.codegen(compiler);
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
            Expression::Assign(assignment) => assignment.codegen(compiler),
        }
    }
}

impl Codegen for AssignExpression {
    fn codegen(&self, compiler: &mut Compiler) {
        let variable_name = match &*self.lhs {
            Expression::Variable(variable) => &variable.value,
            _ => unimplemented!(),
        };
        self.rhs.codegen(compiler);

        let arg = compiler.resolve_arg(variable_name.clone());
        let local = compiler.resolve_local(variable_name.clone());

        match (arg, local) {
            (Some(arg_idx), None) => {
                compiler.emit_bytes(&[Opcode::Deepset(arg_idx + 1)]);
            }
            (None, Some(local_idx)) => {
                compiler.emit_bytes(&[Opcode::DeepsetReverse(local_idx + 1)]);
            }
            (Some(arg), Some(local)) => {}
            (None, None) => {
                compiler.locals.push(variable_name.clone());
            }
        }
    }
}

impl Codegen for CallExpression {
    fn codegen(&self, compiler: &mut Compiler) {
        for argument in self.arguments.iter().rev() {
            argument.codegen(compiler);
        }

        compiler.emit_bytes(&[Opcode::Ip]);

        let jmp_addr = compiler.functions.get(&self.variable).unwrap();
        compiler.emit_bytes(&[Opcode::Jmp(*jmp_addr as isize)]);
    }
}

impl Codegen for BinaryExpression {
    fn codegen(&self, compiler: &mut Compiler) {
        self.lhs.codegen(compiler);
        self.rhs.codegen(compiler);

        match self.kind {
            BinaryExpressionKind::Add => {
                compiler.emit_bytes(&[Opcode::Add]);
            }
            BinaryExpressionKind::Sub => {
                compiler.emit_bytes(&[Opcode::Sub]);
            }
            BinaryExpressionKind::Mul => {
                compiler.emit_bytes(&[Opcode::Mul]);
            }
            BinaryExpressionKind::Div => {
                compiler.emit_bytes(&[Opcode::Div]);
            }
            BinaryExpressionKind::Less => {
                compiler.emit_bytes(&[Opcode::Less]);
            }
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
            },
            Literal::Null => {
                compiler.emit_bytes(&[Opcode::Null]);
            }
        }
    }
}

impl Codegen for VariableExpression {
    fn codegen(&self, compiler: &mut Compiler) {
        let arg = compiler.resolve_arg(self.value.clone());
        let local = compiler.resolve_local(self.value.clone());

        match (arg, local) {
            (Some(arg_idx), None) => {
                compiler.emit_bytes(&[Opcode::Deepget(arg_idx + 1)]);
            }
            (None, Some(local_idx)) => {
                compiler.emit_bytes(&[Opcode::DeepgetReverse(local_idx + 1)]);
            }
            (Some(arg), Some(local)) => {}
            (None, None) => {}
        }
    }
}
