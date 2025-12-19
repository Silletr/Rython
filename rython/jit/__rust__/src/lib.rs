use pyo3::{prelude::*, exceptions::PyValueError};

// Import modules
mod function;
mod math;
mod print_functions;
pub mod compiler;
pub mod parser;

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

#[pyfunction]
fn jit_test() -> PyResult<u64> {
    let code = "function main() -> int:\n    return 42 + 8 * 2";
    let ast = parser::parse_rython_code(code).map_err(|e| PyValueError::new_err(format!("{}", e)))?;
    
    let mut interpreter = compiler::Interpreter::new();
    match interpreter.run(&ast) {
        Some(val) => Ok(val.to_u64()),
        None => Ok(0),
    }
}

#[pyfunction]
fn compile_to_native(code: &str) -> PyResult<String> {
    println!("Rust received code:\n{}", code);
    match parser::parse_rython_code(code) {
        Ok(ast) => {
            let mut interpreter = compiler::Interpreter::new();
            match interpreter.run(&ast) {
                Some(val) => Ok(format!("Execution result: {:?}", val)),
                None => Ok("Code executed successfully (no return value)".to_string()),
            }
        }
        Err(e) => Err(PyValueError::new_err(format!("Parsing error: {}", e))),
    }
}

/// Rython JIT module
#[pymodule]
fn rython_jit(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(hello_rust, m)?)?;
    m.add_function(wrap_pyfunction!(jit_test, m)?)?;
    m.add_function(wrap_pyfunction!(compile_to_native, m)?)?;
    
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
