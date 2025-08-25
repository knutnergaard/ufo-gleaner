use pyo3::prelude::*;

pub mod py_data;
pub mod py_error;
pub mod py_gleaner;
pub mod py_provider;

pub use py_data::*;
pub use py_error::*;
pub use py_gleaner::*;
pub use py_provider::*;

pub fn register(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<py_gleaner::PyUfoGleaner>()?;
    m.add_class::<py_provider::PyFileProvider>()?;
    Ok(())
}
