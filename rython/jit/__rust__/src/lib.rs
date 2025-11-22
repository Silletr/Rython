use pyo3::prelude::*;

// Import functions
mod math;

use math::add_func::add;
use math::fib_func::fib;
use math::multiply_func::multiply;

#[pyfunction]
fn hello_rust(name: &str) -> PyResult<String> {
    Ok(format!("Hello from Rust, {}!", name))
}

/// Rython JIT module
#[pymodule]
fn rython_jit(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(hello_rust, m)?)?;
    m.add_function(wrap_pyfunction!(add, m)?)?;
    m.add_function(wrap_pyfunction!(fib, m)?)?;
    m.add_function(wrap_pyfunction!(multiply, m)?)?;
    Ok(())
}
