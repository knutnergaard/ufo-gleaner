#[cfg(feature = "python")]
pub mod bindings;
pub mod core;
pub mod test_utils;

pub use core::*;

#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::types::PyModule;

#[cfg(feature = "python")]
#[pymodule]
fn ufo_gleaner(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Register classes defined in bindings
    bindings::register(py, m)
}
