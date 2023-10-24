use reaper::compiler::Compiler;
use reaper::parser::Parser;
use reaper::tokenizer::Tokenizer;
use reaper::util::read_file;
use reaper::vm::VM;
use std::env;

fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = env::args().collect();
    match args.get(1) {
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
