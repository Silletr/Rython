use pyo3::prelude::*;

#[pyfunction]
#[pyo3(name = "multiply", signature = (a, b))]
/// Multiply two integers. Returns i64.
///
/// Example:
///     >>> multiply(6, 7)
///     42
pub fn multiply(a: i64, b: i64) -> i64 {
    a * b
}
