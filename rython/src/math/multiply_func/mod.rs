use pyo3::prelude::*;

/// Multiply numbers, like a * b
/// dont try to a: int * b: str = its just crash (tested by Silletr)
/// Example:
///     >>> multiply(6, 7)
///     42
#[pyfunction]
pub fn multiply(a: f64, b: f64) -> f64 {
    let res = a * b;
    println!("{}", res);
    res
}
