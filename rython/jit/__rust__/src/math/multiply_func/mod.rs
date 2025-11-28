use pyo3::prelude::*;

/// Multiply numbers, like a * b
/// dont try to a: int * b: str = its just crash (tested by Silletr)
#[pyfunction]
pub fn multiply(a: i64, b: i64) -> PyResult<i64> {
    Ok(a * b)
}
