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
    // root: PyObject,
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
