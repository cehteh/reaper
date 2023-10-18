mod compiler;
mod parser;
mod tokenizer;
mod vm;

use crate::compiler::Compiler;
use crate::parser::Parser;
use crate::tokenizer::Tokenizer;
use crate::vm::VM;

fn main() {
    let src = r"
        fn fib(n) { 
            if (n < 2) return n;
            return fib(n-1) + fib(n-2);
        }
        
        print fib(40);";
    let tokenizer = Tokenizer::new(src);
    let mut parser = Parser::new();
    let mut compiler = Compiler::new();
    let mut vm = VM::new();
    let ast = parser.parse(tokenizer.into_iter().collect());
    let bytecode = compiler.compile(ast);
    vm.run(&bytecode);
}
