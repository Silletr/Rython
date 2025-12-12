use pyo3::prelude::*;

/// Adding a number, like: a + b
#[pyfunction]
#[pyo3(name = "add", signature = (a, b))]
/// Add two numbers (int or float).
/// Automatically converts to float if needed.
///
/// Example:
///     >>> add(5, 3)
///     8
///     >>> add(2.5, 3)
///     5.5
pub fn add(a: f64, b: f64) -> f64 {
    a + b
}
