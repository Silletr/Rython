use pyo3::{prelude::*, exceptions::PyValueError};
use inkwell::context::Context;
use inkwell::OptimizationLevel;
use inkwell::execution_engine::JitFunction;

// Import modules
mod function;
mod math;
mod print_functions;
pub mod compiler;
pub mod parser;
pub mod runtime; // New runtime module with GC

// Math operations
use math::add_func::add;
use math::divide_func::divide;
use math::fib_func::fibonacci;
use math::minus_func::minus;
use math::multiply_func::multiply;

// Print functions
use print_functions::concatenate_print::print_con;
use print_functions::float_print::print_float;
use print_functions::integer_print::print_integer;
use print_functions::string_print::print_string;

// Function defining
use function::function_define::function_define;

#[pyfunction]
fn hello_rust(name: &str) -> PyResult<String> {
    Ok(format!("Hello from Rust, {}!", name))
}

type MainFunc = unsafe extern "C" fn() -> i64;

#[pyfunction]
fn jit_run(code: &str) -> PyResult<i64> {
    let ast = parser::parse_rython_code(code.trim()).map_err(|e| PyValueError::new_err(format!("{}", e)))?;
    
    let context = Context::create();
    let mut compiler = compiler::Compiler::new(&context, "jit_run_mod");
    compiler.compile_program(&ast);
    
    let execution_engine = compiler.module
        .create_jit_execution_engine(OptimizationLevel::None)
        .map_err(|e| PyValueError::new_err(format!("Failed to create JIT: {}", e)))?;
    
    // Explicitly map runtime symbols for the JIT engine
    let init_fn = compiler.module.get_function("rython_mem_init").unwrap();
    execution_engine.add_global_mapping(&init_fn, runtime::rython_mem_init as usize);

    let malloc_fn = compiler.module.get_function("rython_malloc").unwrap();
    execution_engine.add_global_mapping(&malloc_fn, runtime::rython_malloc as usize);
    
    unsafe {
        let ok_main: JitFunction<MainFunc> = execution_engine.get_function("main")
            .map_err(|e| PyValueError::new_err(format!("Main not found: {}", e)))?;
        Ok(ok_main.call())
    }
}

#[pyfunction]
fn jit_test() -> PyResult<i64> {
    let code = "function main() -> int:\n    return 42 + 8 * 2";
    jit_run(code)
}

#[pyfunction]
fn compile_to_native(code: &str) -> PyResult<String> {
    match parser::parse_rython_code(code) {
        Ok(ast) => {
            let context = Context::create();
            let mut compiler = compiler::Compiler::new(&context, "native_mod");
            compiler.compile_program(&ast);
            Ok(compiler.emit_ir_to_string())
        }
        Err(e) => Err(PyValueError::new_err(format!("Parsing error: {}", e))),
    }
}

#[pyfunction]
fn compile_to_object(code: &str, output_path: &str) -> PyResult<()> {
    let ast = parser::parse_rython_code(code).map_err(|e| PyValueError::new_err(format!("{}", e)))?;
    let context = Context::create();
    let mut compiler = compiler::Compiler::new(&context, "obj_mod");
    compiler.compile_program(&ast);
    compiler.emit_to_file(output_path).map_err(|e| PyValueError::new_err(e))?;
    Ok(())
}

#[pyfunction]
fn get_llvm_ir(code: &str) -> PyResult<String> {
    let ast = parser::parse_rython_code(code).map_err(|e| PyValueError::new_err(format!("{}", e)))?;
    let context = Context::create();
    let mut compiler = compiler::Compiler::new(&context, "ir_mod");
    compiler.compile_program(&ast);
    Ok(compiler.emit_ir_to_string())
}

/// Rython JIT module
#[pymodule]
fn rython_jit(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(hello_rust, m)?)?;
    m.add_function(wrap_pyfunction!(jit_test, m)?)?;
    m.add_function(wrap_pyfunction!(jit_run, m)?)?;
    m.add_function(wrap_pyfunction!(compile_to_native, m)?)?;
    m.add_function(wrap_pyfunction!(compile_to_object, m)?)?;
    m.add_function(wrap_pyfunction!(get_llvm_ir, m)?)?;
    
    // Math operations
    m.add_function(wrap_pyfunction!(add, m)?)?;
    m.add_function(wrap_pyfunction!(minus, m)?)?;
    m.add_function(wrap_pyfunction!(fibonacci, m)?)?;
    m.add_function(wrap_pyfunction!(multiply, m)?)?;
    m.add_function(wrap_pyfunction!(divide, m)?)?;

    // Prints functions
    m.add_function(wrap_pyfunction!(print_float, m)?)?;
    m.add_function(wrap_pyfunction!(print_integer, m)?)?;
    m.add_function(wrap_pyfunction!(print_string, m)?)?;
    m.add_function(wrap_pyfunction!(print_con, m)?)?;
    m.add_function(wrap_pyfunction!(function_define, m)?)?;
    Ok(())
}
