use pyo3::prelude::*;

#[pyfunction]
fn hello_rust(name: &str) -> PyResult<String> {
    Ok(format!("Hello from Rust, {}!", name))
}

#[pymodule]
fn rython_jit(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(pyo3::wrap_pyfunction_bound!(hello_rust, m))?;
    Ok(())
}
