use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::bindings::{PyFileProvider, PyGlifData, PyProvider};
use crate::gleaner::Gleaner;

/// Batch-parse UFO GLIF files Eagerly.
///
/// To use this class, you must provide a concrete implementation of the [`PyProvider`] protocol,
/// exemplified by the included [`PyFileProvider`] class.
#[pyclass(unsendable, name = "Gleaner")]
pub struct PyGleaner {
    inner: Gleaner,
}

#[pymethods]
impl PyGleaner {
    /// Creates a new `Gleaner`.
    ///
    /// # Arguments
    ///
    /// * `provider` – A Python object implementing the provider interface (e.g., `FileProvider`).
    ///
    /// # Example
    ///
    /// ```python
    /// from ufo_gleaner import Gleaner, FileProvider
    ///
    /// provider = FileProvider("/path/to/myfont.ufo")
    /// gleaner = Gleaner(provider)
    /// ```
    #[new]
    pub fn new(py: Python<'_>, provider: Py<PyAny>) -> PyResult<Self> {
        // Try to downcast to PyFileProvider.
        // Return if Ok. If not, assume it's a custom PyProvider implementation.
        match provider.extract::<PyRef<PyFileProvider>>(py) {
            Ok(file_provider) => {
                let gleaner = Gleaner::new(file_provider.inner.clone())?;
                Ok(Self { inner: gleaner })
            }
            Err(_) => {
                let provider = PyProvider::new(py, provider)?;
                let gleaner = Gleaner::new(provider)?;
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
        let map = self.inner.glean()?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use pyo3::types::PyDict;

    use crate::gleaner::Gleaner;
    use crate::paths;
    use crate::test_utils::MockProvider;

    #[test]
    fn test_pyufogleaner_new_and_glean() {
        Python::with_gil(|py| {
            // Mock contents.plist with one glyph
            let contents = br#"<?xml version='1.0'?><plist version='1.0'><dict><key>A</key><string>A.glif</string></dict></plist>"#;
            let mock_provider = MockProvider::new();
            mock_provider.with_file(&paths::UfoRelativePath::Contents.to_pathbuf(), contents);

            // Wrap the provider in PyGleaner manually via inner Gleaner
            let gleaner_inner = crate::gleaner::Gleaner::new(mock_provider).unwrap();
            let py_gleaner = PyGleaner {
                inner: gleaner_inner,
            };

            // Call `glean` and check results
            let dict = py_gleaner.glean(py).unwrap();
            let bound_dict = dict.as_ref().downcast_bound::<PyDict>(py).unwrap();
            assert_eq!(bound_dict.len(), 1);
            assert!(bound_dict.contains("A").unwrap());

            let value = bound_dict.get_item("A").unwrap();
            assert!(!value.is_none());
        });
    }

    #[test]
    fn test_pyufogleaner_glean_empty() {
        Python::with_gil(|_py| {
            let mock_provider = MockProvider::new();
            let result = Gleaner::new(mock_provider);
            assert!(result.is_err());

            let err = result.err().unwrap();

            assert!(matches!(err.kind(), crate::error::ErrorKind::Io));
        });
    }
}
