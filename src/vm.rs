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
            Object::Bool(b) => match b {
                true => Object::Bool(false),
                false => Object::Bool(true),
            },
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

fn adjust_idx(frame_ptrs: &[usize], idx: usize) -> usize {
    let fp = *frame_ptrs.last().unwrap();
    fp - idx
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
            // println!("stack before current instruction: {:?}", self.stack);
            // println!("current instruction: {:?}", bytecode[self.ip as usize]);

            match bytecode[self.ip as usize] {
                Opcode::Const(n) => {
                    self.stack.push(Object::Number(n));
                }
                Opcode::Print => {
                    let obj = self.stack.pop();
                    if let Some(o) = obj {
                        println!("{:?}", o);
                    }
                }
                Opcode::Add => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(a + b);
                }
                Opcode::Sub => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(a - b);
                }
                Opcode::Mul => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(a * b);
                }
                Opcode::Div => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(a / b);
                }
                Opcode::Less => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(Object::Bool(a < b));
                }
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
                Opcode::DirectJmp(addr) => {
                    self.ip = addr;
                }
                Opcode::Ip => {
                    self.frame_ptrs.push(self.stack.len());
                    self.stack.push(Object::BytecodePtr(self.ip + 1));
                }
                Opcode::Ret(popcount) => {
                    let retvalue = self.stack.pop().unwrap();
                    let retaddr = self.stack.pop().unwrap();
                    self.stack.truncate(self.stack.len() - popcount);
                    self.frame_ptrs.pop();
                    self.stack.push(retvalue);
                    match retaddr {
                        Object::BytecodePtr(ptr) => {
                            self.ip = ptr;
                        }
                        _ => {}
                    }
                }
                Opcode::Deepget(idx) => {
                    let adjusted_idx = adjust_idx(&self.frame_ptrs, idx);
                    let item = self.stack[adjusted_idx];
                    self.stack.push(item.clone());
                }
                Opcode::Deepset(idx) => {
                    let adjusted_idx = adjust_idx(&self.frame_ptrs, idx);
                    self.stack[adjusted_idx] = self.stack.pop().unwrap();
                }
            }

            self.ip += 1;

            if self.ip as usize == bytecode.len() {
                break;
            }
        }
    }
}
