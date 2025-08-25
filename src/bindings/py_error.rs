//! Conversion utilities between Rust [`Error`] and Python exceptions (`PyErr`) for PyO3.
use pyo3::exceptions::PyIOError;
use pyo3::PyErr;

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
