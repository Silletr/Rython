use pyo3::prelude::*;

#[pyfunction]
pub fn multiply(a: i64, b: i64) -> PyResult<i64> {
    Ok(a * b)
}
