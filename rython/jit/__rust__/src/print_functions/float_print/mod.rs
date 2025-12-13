use pyo3::prelude::*;

pub fn print_type_of<T>(_: &T) -> String {
    format!("Type: {}", std::any::type_name::<T>())
}

#[pyfunction]
#[pyo3(name = "print_float", signature = (value))]
/// Print float value with prefix.
///
/// Example:
///     >>> print_float(3.14)
///     >>> 3.14

pub fn print_float(value: f64) {
    println!("{}", value);
    println!("{}", print_type_of(&value));
}
