use pyo3::prelude::*;

/// Minus-function.. idk what to write here, really :D
#[pyfunction]
#[pyo3(name = "minus", signature = (a, b))]
/// Subtract two numbers.
///
/// Example:
///     >>> minus(10, 7)
///     3
///     >>> minus(1.5, 2.0)
///     -0.5
pub fn minus(a: f64, b: f64) -> f64 {
    a - b
}
