use pyo3::prelude::*;

/// Will be used for defining functions.
/// !!! NOW THIS EMPTY, DONT BE WONDERFULED WHEN YOU WILL HAVE EMPTY IN ASWER !!!
#[pyfunction]
#[pyo3(name = "function_define")]
pub fn function_define(func_name: String) -> PyResult<String> {
    println!("Received code: {}", func_name);
    Ok(func_name)
}
