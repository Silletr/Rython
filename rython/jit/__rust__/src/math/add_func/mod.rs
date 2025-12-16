use pyo3::prelude::*;

/// Add two numbers (int or float).
/// Automatically converts to float if needed.
///
/// Example:
///     >>> add(5, 3)
///     8
///     >>> add(2.5, 3)
///     5.5
#[pyfunction]
#[pyo3(name = "add", signature = (a, b))]
pub fn add(a: f64, b: f64) -> PyResult<f64> {
    Ok(a + b)
}
