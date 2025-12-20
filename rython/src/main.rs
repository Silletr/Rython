use rython_jit::compiler::Compiler;
use rython_jit::parser;
use inkwell::context::Context;
use std::env;
use std::fs;
use std::path::Path;
use anyhow::{Result, Context as AnyhowContext};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("Rython Standalone Compiler (rythonc)");
        println!("Usage: rythonc <input.ry> [-o <output.o>] [--emit-llvm]");
        return Ok(());
    }

    let input_file = &args[1];
    let mut output_file = "output.o".to_string();
    let mut emit_llvm = false;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "-o" => {
                if i + 1 < args.len() {
                    output_file = args[i + 1].clone();
                    i += 2;
                } else {
                    anyhow::bail!("Missing output path after -o");
                }
            }
            "--emit-llvm" => {
                emit_llvm = true;
                i += 1;
            }
            _ => i += 1,
        }
    }

    let code = fs::read_to_string(input_file)
        .with_context(|| format!("Failed to read input file: {}", input_file))?;

    let ast = parser::parse_rython_code(&code)
        .map_err(|e| anyhow::anyhow!("Parsing error: {}", e))?;

    let context = Context::create();
    let mut compiler = Compiler::new(&context, "rython_mod");
    compiler.compile_program(&ast);

    if emit_llvm {
        let ir = compiler.emit_ir_to_string();
        let ir_path = format!("{}.ll", Path::new(&output_file).file_stem().unwrap().to_str().unwrap());
        fs::write(&ir_path, ir)?;
        println!("LLVM IR emitted to: {}", ir_path);
    }

    compiler.emit_to_file(&output_file)
        .map_err(|e| anyhow::anyhow!("Compilation error: {}", e))?;

    println!("Successfully compiled {} to {}", input_file, output_file);
    
    Ok(())
}
