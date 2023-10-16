use crate::compiler::Opcode;

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

fn adjust_idx(frame_ptrs: &[usize], idx: usize, current_fp: usize) -> isize {
    let fp = frame_ptrs[current_fp-1];
    // println!("fp_stack is: {:?}", fp_stack);
    // println!("self.fp_count is: {}", self.fp_count);
    let adjustment = if current_fp == 0 { -1 } else { 0 };
    fp as isize + idx as isize  + adjustment 
}

pub struct VM {
    stack: Vec<Object>,
    frame_ptrs: Vec<usize>,
    current_fp: usize,
    ip: i64,
}

impl VM {
    pub fn new() -> VM {
        VM {
            stack: vec![],
            current_fp: 0,
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
                Opcode::Jmp(offset) => {
                    self.ip += offset;
                }
                Opcode::Jz(offset) => {
                    let item = self.stack.pop().unwrap();
                    match item {
                        Object::Bool(b) => match b {
                            true => {}
                            false => {
                                self.ip += offset;
                            }
                        }
                        _ => unimplemented!(),
                    }
                }
                Opcode::Ip(offset) => {
                    self.frame_ptrs.push(self.stack.len());
                    self.stack.push(Object::Ptr(self.ip+offset as i64));
                }
                Opcode::DirectJmp(jmp_addr) => {
                    self.ip = jmp_addr as i64;
                }
                Opcode::Ret => {
                    let retvalue = self.stack.pop().unwrap();
                    let retaddr = self.stack.pop().unwrap();
                    self.frame_ptrs.pop();
                    self.current_fp -= 1;
                    self.stack.push(retvalue);
                    match retaddr {
                        Object::Ptr(ptr) => {
                            self.ip = ptr;
                        }
                        _ => {}
                    }
                }
                Opcode::Deepget(idx) => {
                    let item = self.stack.get(adjust_idx(&self.frame_ptrs, idx, self.current_fp) as usize).unwrap();
                    self.stack.push(item.clone());
                }
                Opcode::Deepset(idx) => {
                    self.stack[adjust_idx(&self.frame_ptrs, idx, self.current_fp) as usize] = self.stack.pop().unwrap();
                }
                Opcode::IncFpcount => {
                    self.current_fp += 1;
                }
                Opcode::Pop => {
                    self.stack.pop();
                }
            }
    
            self.ip += 1;
        
            if self.ip as usize == bytecode.len() {
                break;
            }
    
        }   
    }
}