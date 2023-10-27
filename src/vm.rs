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
            _ => unimplemented!(),
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

pub struct VM {
    stack: Vec<Object>,
    frame_ptrs: Vec<InternalObject>,
    ip: usize,
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

const STACK_MIN: usize = 1024;

impl VM {
    pub fn new() -> VM {
        VM {
            stack: Vec::with_capacity(STACK_MIN),
            frame_ptrs: Vec::with_capacity(STACK_MIN),
            ip: 0,
        }
    }

    pub fn run(&mut self, bytecode: &[Opcode]) {
        while self.ip != bytecode.len() {
            if cfg!(debug_assertions) {
                println!("current instruction: {:?}", bytecode[self.ip]);
            }

            match &bytecode[self.ip] {
                Opcode::Const(n) => {
                    self.stack.push((*n).into());
                }
                Opcode::Str(s) => {
                    self.stack.push(s.clone().into());
                }
                Opcode::Print => {
                    let obj = self.stack.pop();
                    if let Some(o) = obj {
                        if cfg!(debug_assertions) {
                            print!("dbg: ");
                        }
                        println!("{:?}", o);
                    }
                }
                Opcode::Add => binop!(self, +),
                Opcode::Sub => binop!(self, -),
                Opcode::Mul => binop!(self, *),
                Opcode::Div => binop!(self, /),
                Opcode::Less => binop!(self, <),
                Opcode::Eq => binop!(self, ==),
                Opcode::False => {
                    self.stack.push(false.into());
                }
                Opcode::Not => {
                    let obj = self.stack.pop().unwrap();
                    self.stack.push(!obj);
                }
                Opcode::Null => {
                    self.stack.push(Object::Null);
                }
                Opcode::Jmp(addr) => {
                    self.ip = *addr;
                }
                Opcode::Jz(addr) => {
                    let item = self.stack.pop().unwrap();
                    if let Object::Bool(_b @ false) = item {
                        self.ip = *addr;
                    }
                }
                Opcode::Invoke(n) => {
                    self.frame_ptrs.push(InternalObject::BytecodePtr(
                        self.ip + 1,
                        self.stack.len() - n,
                    ));
                }
                Opcode::Ret => {
                    let retaddr = self.frame_ptrs.pop().unwrap();
                    let InternalObject::BytecodePtr(ptr, _) = retaddr;
                    self.ip = ptr;
                }
                Opcode::Deepget(idx) => {
                    let item = self.stack[adjust_idx!(self, idx)].clone();
                    self.stack.push(item);
                }
                Opcode::Deepset(idx) => {
                    self.stack.swap_remove(adjust_idx!(self, idx));
                }
                Opcode::Pop => {
                    self.stack.pop();
                }
            }

            if cfg!(debug_assertions) {
                println!("stack: {:?}", self.stack);
            }

            self.ip += 1;
        }
    }
}
