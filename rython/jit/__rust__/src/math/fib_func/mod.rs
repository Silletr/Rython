use pyo3::prelude::*;

#[pyfunction]
#[pyo3(name = "fibonacci", signature = (n))]
/// Calculate fibonacci number iteratively (super fast).
///
/// Example:
///     >>> fibonacci(150)
///     9969216677189303386214405760200
pub fn fibonacci(n: i64) -> i64 {
    if n <= 1 {
        return n;
    }
    let mut a = 0i64;
    let mut b = 1i64;
    for _ in 2..=n {
        let temp = a + b;
        a = b;
        b = temp;
    }
    b
}
