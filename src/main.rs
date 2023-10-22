mod compiler;
mod parser;
mod tokenizer;
mod util;
mod vm;

use crate::compiler::Compiler;
use crate::parser::Parser;
use crate::tokenizer::Tokenizer;
use crate::util::read_file;
use crate::vm::VM;
use std::env;

fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = env::args().collect();
    match args.iter().nth(1) {
        Some(path) => {
            let src = read_file(path)?;
            let tokenizer = Tokenizer::new(&src);
            let mut parser = Parser::new();
            let mut compiler = Compiler::new();
            let mut vm = VM::new();
            let ast = parser.parse(tokenizer.into_iter().collect());
            let bytecode = compiler.compile(ast);
            vm.run(&bytecode);
        }
        None => eprintln!("You must pass in a path."),
    }

    Ok(())
}
