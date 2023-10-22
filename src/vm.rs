use crate::compiler::Opcode;

#[derive(Debug, PartialEq, Clone, Copy)]
enum Object {
    Number(f64),
    Bool(bool),
    Null,
    BytecodePtr(isize),
}

impl std::ops::Add for Object {
    type Output = Object;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Object::Number(a), Object::Number(b)) => Object::Number(a + b),
            _ => unimplemented!(),
        }
    }
}

impl std::ops::Sub for Object {
    type Output = Object;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Object::Number(a), Object::Number(b)) => Object::Number(a - b),
            _ => unimplemented!(),
        }
    }
}

impl std::ops::Mul for Object {
    type Output = Object;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Object::Number(a), Object::Number(b)) => Object::Number(a * b),
            _ => unimplemented!(),
        }
    }
}

impl std::ops::Div for Object {
    type Output = Object;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Object::Number(a), Object::Number(b)) => Object::Number(a / b),
            _ => unimplemented!(),
        }
    }
}

impl std::ops::Not for Object {
    type Output = Object;

    fn not(self) -> Self::Output {
        match self {
            Object::Bool(b) => Object::Bool(!b),
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
        Object::Bool(value)
    }
}

fn adjust_idx(frame_ptrs: &[usize], idx: usize) -> usize {
    let fp = *frame_ptrs.last().unwrap();
    fp - idx
}

fn adjust_idx_reverse(frame_ptrs: &[usize], idx: usize) -> usize {
    let (fp, idx) = match frame_ptrs.last() {
        Some(&ptr) => (ptr, idx),
        None => (0, idx-1),
    };
    fp + idx
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
    frame_ptrs: Vec<usize>,
    ip: isize,
}

impl VM {
    pub fn new() -> VM {
        VM {
            stack: vec![],
            frame_ptrs: vec![],
            ip: 0,
        }
    }

    pub fn run(&mut self, bytecode: &[Opcode]) {
        loop {
            println!("current instruction: {:?}", bytecode[self.ip as usize]);

            match bytecode[self.ip as usize] {
                Opcode::Const(n) => {
                    self.stack.push(Object::Number(n));
                }
                Opcode::Print => {
                    let obj = self.stack.pop();
                    if let Some(o) = obj {
                        println!("dbg: {:?}", o);
                    }
                }
                Opcode::Add => binop!(self, +),
                Opcode::Sub => binop!(self, -),
                Opcode::Mul => binop!(self, *),
                Opcode::Div => binop!(self, /),
                Opcode::Less => binop!(self, <),
                Opcode::False => {
                    self.stack.push(Object::Bool(false));
                }
                Opcode::Not => {
                    let obj = self.stack.pop().unwrap();
                    self.stack.push(!obj);
                }
                Opcode::Null => {
                    self.stack.push(Object::Null);
                }
                Opcode::Jmp(addr) => {
                    self.ip = addr;
                }
                Opcode::Jz(addr) => {
                    let item = self.stack.pop().unwrap();
                    match item {
                        Object::Bool(b) => match b {
                            true => {}
                            false => {
                                self.ip = addr;
                            }
                        },
                        _ => unimplemented!(),
                    }
                }
                Opcode::Ip => {
                    self.frame_ptrs.push(self.stack.len());
                    self.stack.push(Object::BytecodePtr(self.ip + 1));
                }
                Opcode::Ret(before, after) => {
                    let retvalue = self.stack.pop().unwrap();
                    self.stack.truncate(self.stack.len() - after);
                    let retaddr = self.stack.pop().unwrap();
                    self.stack.truncate(self.stack.len() - before);
                    self.frame_ptrs.pop();
                    self.stack.push(retvalue);
                    if let Object::BytecodePtr(ptr) = retaddr {
                        self.ip = ptr;
                    }
                }
                Opcode::Deepget(idx) => {
                    let adjusted_idx = adjust_idx(&self.frame_ptrs, idx);
                    let item = self.stack[adjusted_idx];
                    self.stack.push(item);
                }
                Opcode::DeepgetReverse(idx) => {
                    let adjusted_idx = adjust_idx_reverse(&self.frame_ptrs, idx);
                    let item = self.stack[adjusted_idx];
                    self.stack.push(item);
                }
                Opcode::Deepset(idx) => {
                    let adjusted_idx = adjust_idx(&self.frame_ptrs, idx);
                    self.stack[adjusted_idx] = self.stack.pop().unwrap();
                }
                Opcode::DeepsetReverse(idx) => {
                    let adjusted_idx = adjust_idx_reverse(&self.frame_ptrs, idx);
                    self.stack[adjusted_idx] = self.stack.pop().unwrap();
                }
                Opcode::Pop => {
                    self.stack.pop();
                }
            }

            println!("stack: {:?}", self.stack);

            self.ip += 1;

            if self.ip as usize == bytecode.len() {
                break;
            }
        }
    }
}
