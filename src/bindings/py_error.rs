//! Conversion utilities between Rust [`Error`] and Python exceptions (`PyErr`) for PyO3.
use pyo3::PyErr;
use pyo3::exceptions::PyIOError;

use crate::error::{Error, ErrorKind};

// Extension trait to convert a Python exception (`PyErr`) into a Rust [`Error`].
///
/// This allows integrating Python-originated errors into the Rust error handling chain,
/// preserving the original exception as the cause.
pub trait PyErrExt {
    /// Converts the Python exception into a [`Error`] with [`ErrorKind::Other`].
    ///
    /// The original `PyErr` is stored as the cause for debugging purposes.
    fn to_ufo(self) -> Error;
}

impl PyErrExt for PyErr {
    fn to_ufo(self) -> Error {
        let msg = self.to_string();
        Error::new(ErrorKind::Other(msg)).with_cause(self)
    }
}

/// Extension trait to convert a Rust [`Error`] into a Python exception (`PyErr`).
///
/// This is used when returning results from PyO3 functions so that Rust errors can
/// be raised as Python exceptions.
pub trait ToPyErr {
    /// Converts the Rust [`Error`] into a [`PyIOError`] by default.
    ///
    /// The error message includes context and path information from the Rust [`Error`].
    fn to_pyerr(self) -> PyErr;
}

impl ToPyErr for Error {
    fn to_pyerr(self) -> PyErr {
        PyIOError::new_err(self.to_string())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use pyo3::exceptions::PyIOError;
    use pyo3::prelude::*;

    use std::error::Error as StdError;

    #[test]
    fn pyerr_to_ufo_sets_cause_and_kind() {
        Python::with_gil(|_py| {
            let pyerr = PyIOError::new_err("test error");
            let err = pyerr.to_ufo();
            // Check that the kind contains the original message
            assert!(matches!(err.kind(), ErrorKind::Other(msg) if msg.contains("test error")));
            // Check that the cause is stored as PyErr
            assert!(err.source().is_some());
        });
    }

    #[test]
    fn error_to_pyerr_returns_pyioerror_with_message() {
        let rust_err = Error::new(ErrorKind::Other("some context".to_string()));
        let py_err = rust_err.to_pyerr();
        Python::with_gil(|_py| {
            // Ensure the PyErr is a PyIOError
            assert!(py_err.is_instance_of::<PyIOError>(_py));
            assert!(py_err.to_string().contains("some context"));
        });
    }
}
