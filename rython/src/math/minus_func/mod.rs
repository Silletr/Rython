use pyo3::prelude::*;

/// Minus-function.. idk what to write here, really :D
/// Well... Subtract two numbers.
///
/// Example:
///     >>> minus(10, 7)
///     3
///     >>> minus(1.5, 2.0)
///     -0.5
#[pyfunction]
#[pyo3(name = "minus", signature = (a, b))]
pub fn minus(a: f64, b: f64) -> PyResult<f64> {
    Ok(a - b)
}
