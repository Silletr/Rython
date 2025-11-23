use pyo3::prelude::*;

#[pyfunction]
pub fn fib(n: i64) -> i64 {
    if n <= 1 {
        return n;
    }

    let mut a = 0;
    let mut b = 1;

    for _ in 2..=n {
        let tmp = a + b;
        a = b;
        b = tmp;
    }

    b
}
