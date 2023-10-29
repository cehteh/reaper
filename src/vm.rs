use crate::compiler::Opcode;

#[derive(Debug, PartialEq, Clone)]
pub enum Object {
    Number(f64),
    Bool(bool),
    String(Box<String>),
    Null,
}

#[derive(Debug, Clone, Copy)]
enum InternalObject {
    BytecodePtr(usize, usize),
}

impl std::ops::Add for Object {
    type Output = Object;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Object::Number(a), Object::Number(b)) => (a + b).into(),
            _ => unreachable!(),
        }
    }
}

impl std::ops::Sub for Object {
    type Output = Object;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Object::Number(a), Object::Number(b)) => (a - b).into(),
            _ => unimplemented!(),
        }
    }
}

impl std::ops::Mul for Object {
    type Output = Object;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Object::Number(a), Object::Number(b)) => (a * b).into(),
            _ => unimplemented!(),
        }
    }
}

impl std::ops::Div for Object {
    type Output = Object;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Object::Number(a), Object::Number(b)) => (a / b).into(),
            _ => unimplemented!(),
        }
    }
}

impl std::ops::Not for Object {
    type Output = Object;

    fn not(self) -> Self::Output {
        match self {
            Object::Bool(b) => (!b).into(),
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

impl From<bool> for Object {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<f64> for Object {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl From<String> for Object {
    fn from(value: String) -> Self {
        Self::String(value.into())
    }
}

macro_rules! adjust_idx {
    ($self:tt, $index:expr) => {{
        let (fp, idx) = match $self.frame_ptrs.last() {
            Some(&internal_obj) => {
                let InternalObject::BytecodePtr(_, location) = internal_obj;
                (location, $index)
            }
            None => (0, $index),
        };
        fp + idx
    }};
}

macro_rules! binop {
    ($self:tt, $op:tt) => {
        {
            let b = $self.stack.pop().unwrap();
            let a = $self.stack.pop().unwrap();
            $self.stack.push((a $op b).into());
        }
    };
}

pub struct VM<'a> {
    bytecode: Option<&'a [Opcode]>,
    stack: Vec<Object>,
    frame_ptrs: Vec<InternalObject>,
    ip: usize,
}

impl Default for VM<'_> {
    fn default() -> Self {
        Self::new()
    }
}

macro_rules! runtime_error {
    ($msg:expr) => {{
        eprintln!("{}", $msg);
        std::process::exit(1);
    }};
}

const STACK_MIN: usize = 1024;



impl<'a> VM<'a> {

    pub fn new() -> VM<'a> {
        VM {
            bytecode: None,
            stack: Vec::with_capacity(STACK_MIN),
            frame_ptrs: Vec::with_capacity(STACK_MIN),
            ip: 0,
        }
    }

    pub fn load(&mut self, bytecode: &'a mut Vec<Opcode>) {
        bytecode.push(Opcode::EndOfProgram);
        self.bytecode = Some(bytecode);
    }

    pub fn run(&mut self) {
        let bytecode= self.bytecode.expect("no program loaded");
        assert!(self.ip < bytecode.len(), "ip out of bounds");
        loop {
            match unsafe { bytecode.get_unchecked(self.ip)} {
                Opcode::Const(n) => self.handle_op_const(*n),
                Opcode::Str(ref s) => self.handle_op_str(s),
                Opcode::Strcat => self.handle_op_strcat(),
                Opcode::Print => self.handle_op_print(),
                Opcode::Add => self.handle_op_add(),
                Opcode::Sub => self.handle_op_sub(),
                Opcode::Mul => self.handle_op_mul(),
                Opcode::Div => self.handle_op_div(),
                Opcode::Less => self.handle_op_less(),
                Opcode::Eq => self.handle_op_eq(),
                Opcode::False => self.handle_op_false(),
                Opcode::Not => self.handle_op_not(),
                Opcode::Null => self.handle_op_null(),
                Opcode::Jmp(addr) => self.handle_op_jmp(*addr),
                Opcode::Jz(addr) => self.handle_op_jz(*addr),
                Opcode::Invoke(n) => self.handle_op_invoke(*n),
                Opcode::Ret => self.handle_op_ret(),
                Opcode::Deepget(idx) => self.handle_op_deepget(*idx),
                Opcode::Deepset(idx) => self.handle_op_deepset(*idx),
                Opcode::Pop => self.handle_op_pop(),
                Opcode::EndOfProgram => break,
            }
            self.ip += 1;
        }
    }

    fn handle_op_const(&mut self, n: f64) {
        self.stack.push(n.into());
    }

    fn handle_op_str(&mut self, s: &str) {
        self.stack.push(s.to_owned().into());
    }

    fn handle_op_strcat(&mut self) {
        let b = self.stack.pop().unwrap();
        let a = self.stack.pop().unwrap();

        match (a, b) {
            (Object::String(mut a), Object::String(b)) => {
                a.push_str(&b);
                self.stack.push((*a).into());
            }
            _ => {
                runtime_error!("Can only concatenate two strings.");
            }
        }
    }

    fn handle_op_print(&mut self) {
        let obj = self.stack.pop();
        if let Some(o) = obj {
            if cfg!(debug_assertions) {
                print!("dbg: ");
            }
            println!("{:?}", o);
        }
    }

    fn handle_op_add(&mut self) {
        binop!(self, +);
    }

    fn handle_op_sub(&mut self) {
        binop!(self, -);
    }

    fn handle_op_mul(&mut self) {
        binop!(self, *);
    }

    fn handle_op_div(&mut self) {
        binop!(self, /);
    }

    fn handle_op_eq(&mut self) {
        binop!(self, ==);
    }

    fn handle_op_less(&mut self) {
        binop!(self, <);
    }

    fn handle_op_false(&mut self) {
        self.stack.push(false.into());
    }

    fn handle_op_not(&mut self) {
        let obj = self.stack.pop().unwrap();
        self.stack.push(!obj);
    }

    fn handle_op_null(&mut self) {
        self.stack.push(Object::Null);
    }

    fn handle_op_jmp(&mut self, addr: usize) {
//        assert!(addr+1 < self.bytecode.unwrap().len(), "jmp out of bounds");
        self.ip = addr;
    }

    fn handle_op_jz(&mut self, addr: usize) {
        let item = self.stack.pop().unwrap();
        if let Object::Bool(_b @ false) = item {
//            assert!(addr+1 < self.bytecode.unwrap().len(), "jz out of bounds");
            self.ip = addr;
        }
    }

    fn handle_op_invoke(&mut self, n: usize) {
        self.frame_ptrs.push(InternalObject::BytecodePtr(
            self.ip + 1,
            self.stack.len() - n,
        ));
    }

    fn handle_op_ret(&mut self) {
        let retaddr = self.frame_ptrs.pop().unwrap();
        let InternalObject::BytecodePtr(ptr, _) = retaddr;
//        debug_assert!(ptr+1 < self.bytecode.unwrap().len(), "ret out of bounds");
        self.ip = ptr;
    }

    fn handle_op_deepget(&mut self, idx: usize) {
        let item = self.stack[adjust_idx!(self, idx)].clone();
        self.stack.push(item);
    }

    fn handle_op_deepset(&mut self, idx: usize) {
        self.stack.swap_remove(adjust_idx!(self, idx));
    }

    fn handle_op_pop(&mut self) {
        self.stack.pop();
    }
}
