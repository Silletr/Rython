use pyo3::prelude::*;

#[pyfunction]
pub fn fib(n: i64) -> PyResult<i64> {
    if n <= 1 {
        return Ok(n);
    }
    let a = fib(n - 1)?;
    let b = fib(n - 2)?;
    Ok(a + b)
}
