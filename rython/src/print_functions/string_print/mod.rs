use pyo3::prelude::*;

pub fn print_type_of<T>(_: &T) -> String {
    format!("Type: {}", std::any::type_name::<T>())
}

#[pyfunction]
#[pyo3(name = "print_string", signature = (value))]
/// Print string.
///
/// Example:
///     >>> print_string("Hello from Rython!")
///     Hello from Rython!
pub fn print_string(value: String) {
    println!("{}", value);
    println!("Type: {}", print_type_of(&value));
}
