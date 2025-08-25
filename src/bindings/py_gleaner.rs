use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::bindings::{PyFileProvider, PyGlifData, PyProvider, ToPyErr};
use crate::gleaner::UfoGleaner;
use crate::provider::Provider;

/// High-level parser for UFO GLIF files in Python.
///
/// Wraps [`UfoGleaner`] to provide Python access to UFO glyph data.
/// Users must supply a provider object that implements the file-reading
/// interface (e.g., [`FileProvider`] for local files).
#[pyclass(unsendable, name = "UfoGleaner")]
pub struct PyUfoGleaner {
    inner: UfoGleaner,
}

#[pymethods]
impl PyUfoGleaner {
    /// Creates a new `UfoGleaner`.
    ///
    /// # Arguments
    ///
    /// * `provider` – A Python object implementing the provider interface (e.g., `FileProvider`).
    ///
    /// # Example
    ///
    /// ```python
    /// from ufo_gleaner import UfoGleaner, FileProvider
    ///
    /// provider = FileProvider("/path/to/myfont.ufo")
    /// gleaner = UfoGleaner(provider)
    /// ```
    #[new]
    pub fn new(py: Python<'_>, provider: Py<PyAny>) -> PyResult<Self> {
        // Try to downcast to PyFileProvider.
        // Return if Ok. If not, assume it's a custom PyProvider implementation.
        match provider.extract::<PyRef<PyFileProvider>>(py) {
            Ok(file_provider) => {
                let boxed = Box::new(file_provider.inner.clone());
                let gleaner = UfoGleaner::new(boxed).map_err(|e| e.to_pyerr())?;
                Ok(Self { inner: gleaner })
            }
            Err(_) => {
                let provider = PyProvider::new(py, provider).map_err(|e| e.to_pyerr())?;
                let boxed: Box<dyn Provider> = Box::new(provider);
                let gleaner = UfoGleaner::new(boxed).map_err(|e| e.to_pyerr())?;
                Ok(Self { inner: gleaner })
            }
        }
    }

    /// Parses all glyphs defined in `contents.plist` and returns a dictionary.
    ///
    /// # Returns
    ///
    /// A Python `dict` mapping glyph names (`str`) to either:
    /// * a `dict` representing [crate::glif::`GlifData`] if parsing succeeded, or
    /// * `None` if the `.glif` file could not be read or parsed.
    ///
    /// # Example
    ///
    /// ```python
    /// glyphs = gleaner.glean()
    /// print(glyphs["A"])  # Either a dict with glyph data or None
    /// ```
    pub fn glean(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let map = self.inner.glean().map_err(|e| e.to_pyerr())?;

        let py_dict = PyDict::new(py);

        for (key, maybe_glif) in map {
            if let Some(glif) = maybe_glif {
                // Wrap in PyGlifData so we can call to_pydict
                let py_glif = PyGlifData { inner: glif };
                let value = py_glif.to_pydict(py)?; // PyObject
                py_dict.set_item(key, value)?;
            } else {
                py_dict.set_item(key, py.None())?;
            }
        }

        Ok(py_dict.into())
    }
}
