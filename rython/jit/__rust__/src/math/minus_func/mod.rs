use pyo3::prelude::*;

/// Minus-function.. idk what to write here, really :D
#[pyfunction]
pub fn minus(a: i64, b: i64) -> PyResult<i64> {
    Ok(a - b)
}
