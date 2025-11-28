use pyo3::prelude::*;

/// Adding a number, like: a + b
#[pyfunction]
pub fn add(a: i64, b: i64) -> PyResult<i64> {
    Ok(a + b)
}
