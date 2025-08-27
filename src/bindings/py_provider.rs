//! Python-backed [`Provider`] implementation for UFO file access.
//!
//! This module defines [`PyProvider`], which wraps a Python object exposing a
//! `read` method. It allows Rust code (e.g., [`UfoGleaner`]) to read files from
//! a Python-managed file system or other custom storage.
//!
//! The Rust [`Provider`] trait is implemented, so `PyProvider` can be used wherever
//! a standard `Provider` is expected.
use std::path::Path;

use pyo3::prelude::*;

use crate::bindings::PyErrExt;
use crate::error::Result;
use crate::provider::{FileProvider, Provider};

/// A [`Provider`] implementation backed by a Python object.
///
/// The Python object must implement a method `read(path: str) -> bytes` that
/// returns the content of the requested file as a byte string.
pub struct PyProvider {
    root: PyObject,
}

/// Constructs a new [`PyProvider`] from a Python object.
///
/// # Arguments
///
/// * `root` – A Python object implementing a `read(path: str) -> bytes` method.
///
/// # Errors
///
/// Returns an [`Error`] if the provided object does not meet the expected interface.
impl PyProvider {
    pub fn new(_py: Python<'_>, root: PyObject) -> Result<Self> {
        Ok(PyProvider { root })
    }
}

impl Provider for PyProvider {
    /// Reads a file from the Python object.
    ///
    /// This delegates to the Python object's `read(path: str)` method, converts
    /// the result into a Rust `Vec<u8>`, and propagates any Python exceptions
    /// as [`rustcore::error::Error`].
    fn read(&self, path: &Path) -> Result<Vec<u8>> {
        Python::with_gil(|py| {
            let obj = self.root.as_ref();
            let result = obj
                .call_method1(py, "read", (path.to_string_lossy().as_ref(),))
                .map_err(|e| e.to_ufo())?;

            let bytes: Vec<u8> = result.extract(py).map_err(|e| e.to_ufo())?;
            Ok(bytes)
        })
    }
}

impl Clone for PyProvider {
    /// Clones the [`PyProvider`], creating a new reference to the same Python object.
    fn clone(&self) -> Self {
        Python::with_gil(|py| PyProvider {
            root: self.root.clone_ref(py),
        })
    }
}

#[pyclass(name = "FileProvider")]
pub struct PyFileProvider {
    pub inner: FileProvider,
}

#[pymethods]
impl PyFileProvider {
    #[new]
    pub fn new(path: String) -> Self {
        let inner = FileProvider::new(path);
        Self { inner }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::Provider;
    use pyo3::types::PyBytes;

    use std::collections::HashMap;

    /// Mock Python object with a `read` method
    #[pyclass]
    struct MockPyRoot {
        files: HashMap<String, Vec<u8>>,
    }

    #[pymethods]
    impl MockPyRoot {
        #[new]
        fn new() -> Self {
            Self {
                files: HashMap::new(),
            }
        }

        fn add_file(&mut self, path: String, contents: Vec<u8>) {
            self.files.insert(path, contents);
        }

        fn read(&self, path: &str) -> PyResult<Py<PyBytes>> {
            Python::with_gil(|py| match self.files.get(path) {
                Some(data) => Ok(PyBytes::new(py, data).into()),
                None => Err(pyo3::exceptions::PyIOError::new_err(format!(
                    "file not found: {}",
                    path
                ))),
            })
        }
    }

    #[test]
    fn pyprovider_reads_existing_file() {
        Python::with_gil(|py| {
            let root = Py::new(py, MockPyRoot::new()).unwrap();
            let mut root_ref: PyRefMut<MockPyRoot> = root.extract(py).unwrap();
            root_ref.add_file("test.txt".to_string(), b"hello".to_vec());

            // let root2 = root.clone_ref(py).into_any();
            let provider = PyProvider::new(py, root.into_any()).unwrap();
            let bytes = provider.read(std::path::Path::new("test.txt")).unwrap();
            assert_eq!(bytes, b"hello");
        });
    }

    #[test]
    fn pyprovider_read_missing_file_errors() {
        Python::with_gil(|py| {
            let root = Py::new(py, MockPyRoot::new()).unwrap();
            let provider = PyProvider::new(py, root.into_any()).unwrap();
            let result = provider.read(std::path::Path::new("missing.txt"));
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(err.kind(), crate::error::ErrorKind::Other(_)));
        });
    }

    #[test]
    fn pyprovider_clone_preserves_data() {
        Python::with_gil(|py| {
            let root = Py::new(py, MockPyRoot::new()).unwrap();
            let mut root_ref: PyRefMut<MockPyRoot> = root.extract(py).unwrap();
            root_ref.add_file("clone.txt".to_string(), b"clonedata".to_vec());

            let provider = PyProvider::new(py, root.into_any()).unwrap();
            let cloned = provider.clone();
            let bytes = cloned.read(std::path::Path::new("clone.txt")).unwrap();
            assert_eq!(bytes, b"clonedata");
        });
    }

    #[test]
    fn pyfileprovider_reads_local_file() {
        use std::io::Write;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("file.txt");
        let mut f = std::fs::File::create(&file_path).unwrap();
        f.write_all(b"diskdata").unwrap();

        let pyfile_provider = PyFileProvider::new(dir.path().to_string_lossy().to_string());
        let bytes = pyfile_provider
            .inner
            .read(std::path::Path::new("file.txt"))
            .unwrap();
        assert_eq!(bytes, b"diskdata");
    }
}
