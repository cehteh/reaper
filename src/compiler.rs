use crate::parser::{
    AssignExpression, BinaryExpression, BinaryExpressionKind, BlockStatement, CallExpression,
    Expression, ExpressionStatement, FnStatement, IfStatement, Literal, LiteralExpression,
    PrintStatement, ReturnStatement, Statement, VariableExpression,
};

pub struct Compiler {
    bytecode: Vec<Opcode>,
    functions: std::collections::HashMap<String, usize>,
    locals: Vec<String>,
    depth: usize,
    pops: [usize; 1024],
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler {
            bytecode: Vec::new(),
            functions: std::collections::HashMap::new(),
            locals: Vec::new(),
            depth: 0,
            pops: [0; 1024],
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

    fn emit_stack_cleanup(&mut self) {
        let popcount = self.pops[self.depth];
        for _ in 0..popcount {
            self.bytecode.push(Opcode::Pop);
        }
    }

    fn resolve_local(&self, name: String) -> Option<usize> {
        self.locals.iter().position(|local| *local == name)
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
    Eq,
    Jmp(isize),
    Jz(isize),
    Ret,
    Less,
    Deepget(usize),
    Deepset(usize),
    Pop,
    Invoke(usize, usize),
}

trait Codegen {
    fn codegen(&self, _compiler: &mut Compiler) {}
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
            compiler.locals.push(argument.clone());
        }

        compiler.pops[1] = compiler.locals.len();

        if let Statement::Block(block) = &*self.body {
            block.codegen(compiler);
        }

        compiler.emit_stack_cleanup();

        compiler.emit_bytes(&[Opcode::Null, Opcode::Ret]);

        compiler.bytecode[jmp_idx] = Opcode::Jmp(compiler.bytecode.len() as isize - 1);

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
        let mut deepset_no = compiler.locals.len();
        for _ in 0..compiler.locals.len() {
            compiler.emit_bytes(&[Opcode::Deepset(deepset_no)]);
            deepset_no -= 1;
        }
        compiler.emit_bytes(&[Opcode::Ret]);
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
        compiler.depth += 1;
        for statement in &self.body {
            statement.codegen(compiler);
        }
        compiler.emit_stack_cleanup();
        compiler.depth -= 1;
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

        let local = compiler.resolve_local(variable_name.clone());

        if let Some(idx) = local {
            compiler.emit_bytes(&[Opcode::Deepset(idx + 1)]);
        } else {
            compiler.locals.push(variable_name.clone());
            compiler.pops[compiler.depth] += 1;
        }
    }
}

impl Codegen for CallExpression {
    fn codegen(&self, compiler: &mut Compiler) {
        for argument in &self.arguments {
            argument.codegen(compiler);
        }

        let jmp_addr = compiler.functions.get(&self.variable).unwrap();

        compiler.emit_bytes(&[Opcode::Invoke(self.arguments.len(), *jmp_addr)]);
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
            BinaryExpressionKind::Equality(negation) => {
                compiler.emit_bytes(&[Opcode::Eq]);
                if negation {
                    compiler.emit_bytes(&[Opcode::Not]);
                }
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
        let local = compiler.resolve_local(self.value.clone());

        if let Some(idx) = local {
            compiler.emit_bytes(&[Opcode::Deepget(idx + 1)]);
        } else {
            compiler.locals.push(self.value.clone());
            let idx = compiler.resolve_local(self.value.clone()).unwrap();
            compiler.emit_bytes(&[Opcode::Deepget(idx + 1)]);
        }
    }
}
