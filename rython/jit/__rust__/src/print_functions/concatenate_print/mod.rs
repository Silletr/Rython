use pyo3::prelude::*;

/*
pub fn print_type_of<T>(_: &T) -> String {
    format!("Type: {}", std::any::type_name::<T>())
}
*/

#[pyfunction]
#[pyo3(name = "print_con", signature = (value))]
/// Prints variables with different types
///
/// Example:
///     >>> a: int = 10
///     >>> print_con(a)
///     >>> 10
pub fn print_con(value: &Bound<'_, PyAny>) -> PyResult<()> {
    println!("{}", value.str()?.to_str()?);
    Ok(())
}
