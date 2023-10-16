mod tokenizer;
mod parser;
mod compiler;
mod vm;

use crate::tokenizer::Tokenizer;
use crate::parser::Parser;
use crate::compiler::Compiler;
use crate::vm::VM;

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
    let mut vm = VM::new();
    let ast = parser.parse(tokenizer.into_iter().collect());
    let bytecode = compiler.compile(ast);
    vm.run(&bytecode);
}
