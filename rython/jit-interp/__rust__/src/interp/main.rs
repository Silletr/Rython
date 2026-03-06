mod value;
mod bytecode;
mod vm;
mod arena;
mod ir;
mod lower;
mod frontend;

use frontend::{Lexer, Parser, CodeGen};
use lower::LoweringContext;
use vm::VM;
use std::env;
use std::fs;
use std::io::{self, Read, Write};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let source = if args.len() > 1 {
        fs::read_to_string(&args[1])?
    } else {
        print!("Enter your program (end with Ctrl+D on Unix or Ctrl+Z on Windows):\n");
        io::stdout().flush()?;
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    };

    let lexer = Lexer::new(&source);
    let mut parser = Parser::new(lexer);
    let (funcs, main_stmts) = match std::panic::catch_unwind(|| parser.parse_program()) {
    Ok(result) => result,
    Err(_) => return Err(PyValueError::new_err("Parse error")),
};
        Ok(result) => result,
        Err(_) => {
            eprintln!("Parse error");
            return Ok(());
        }
    };

    let mut codegen = CodeGen::new();
    let ir_program = codegen.generate(funcs, main_stmts);

    let mut lower_ctx = LoweringContext::new();
    let (functions, string_pool) = lower_ctx.lower_program(&ir_program);

    let mut vm = VM::new(functions, string_pool, 1024);
    match vm.run() {
        Ok(()) => (),
        Err(e) => eprintln!("Runtime error: {}", e),
    }

    Ok(())
}