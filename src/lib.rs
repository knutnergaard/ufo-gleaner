pub mod bindings;
pub mod core;

pub use core::*;

use pyo3::prelude::*;
use pyo3::types::PyModule;

#[pymodule]
fn ufo_gleaner(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Register classes defined in bindings
    bindings::register(py, m)
}
