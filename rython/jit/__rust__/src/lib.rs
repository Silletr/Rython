use pyo3::prelude::*;

// Import functions
mod math;
mod print_functions;

// Math operations
use math::add_func::add;
use math::divide_func::divide;
use math::fib_func::fibonacci;
use math::minus_func::minus;
use math::multiply_func::multiply;

// Prints functions
use print_functions::float_print::print_float;
use print_functions::integer_print::print_integer;
use print_functions::string_print::print_string;

#[pyfunction]
fn hello_rust(name: &str) -> PyResult<String> {
    Ok(format!("Hello from Rust, {}!", name))
}

/// Rython JIT module
#[pymodule]
fn rython_jit(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(hello_rust, m)?)?;
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
    Ok(())
}
