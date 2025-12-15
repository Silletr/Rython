use pyo3::prelude::*;

/// Dividing a number, like: a / b
#[pyfunction]
#[pyo3(name = "divide", signature = (a, b))]
/// Divide two numbers (int or float).
/// Automatically converts to float if needed.
///
/// Example:
///     >>> divide(5, 3)
///     1.6
///     >>> divide(10, 5)
///     2
pub fn divide(a: f64, b: f64) -> PyResult<f64> {
    Ok(a / b)
}
