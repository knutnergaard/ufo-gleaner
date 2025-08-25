//! Error handling for UFO/GLIF parsing and related I/O operations.
//!
//! This module defines the [`Error`] type, its associated [`ErrorKind`], and a
//! [`Result`] alias for convenience. It centralizes error construction and conversion
//! from common sources (I/O, XML, plist parsing, and numeric/string parsing).
//!
//! Key features:
//! - Structured error kinds for categorizing failure modes.
//! - Optional path and context fields for richer error messages.
//! - Automatic conversions (`From` impls) from standard and library errors.
//! - Compatible with the standard [`std::error::Error`] trait for use with `?`.

use std::error;
use std::fmt;
use std::io;
use std::num::{ParseFloatError, ParseIntError};
use std::str::Utf8Error;

/// A specialized [`Result`] type for operations that return [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

/// Categories of errors that can occur while parsing UFO or GLIF data.
#[derive(Debug, PartialEq, Eq)]
pub enum ErrorKind {
    /// Underlying I/O error (file access, reading, etc.).
    Io,
    /// Failure while parsing a property list (`.plist`).
    Plist,
    /// Failure while parsing XML.
    Xml,
    /// Failure while parsing integers, floats, or UTF-8 strings.
    Parse,
    /// A requested file was not found.
    FileNotFound,
    /// An expected XML attribute was missing.
    MissingAttribute(String),
    /// Any other error not covered by the above kinds.
    Other(String),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io => write!(f, "I/O error"),
            Self::Plist => write!(f, "Plist parsing error"),
            Self::Xml => write!(f, "Xml parsing error"),
            Self::Parse => write!(f, "Parsing error"),
            Self::FileNotFound => write!(f, "File not found"),
            Self::MissingAttribute(attr) => write!(f, "Missing attribute: {}", attr),
            Self::Other(msg) => write!(f, "{}", msg),
        }
    }
}

/// A structured error type for UFO/GLIF operations.
///
/// This type wraps an [`ErrorKind`] and may include:
/// - An optional file path (`path`) indicating where the error occurred.
/// - An optional context message (`context`) describing what was being done.
/// - An optional underlying cause (`cause`) implementing [`std::error::Error`].
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    path: Option<String>,
    context: Option<String>,
    cause: Option<Box<dyn error::Error + Send + Sync>>,
}

impl Error {
    /// Creates a new error with the given kind and no additional context.
    pub fn new(kind: ErrorKind) -> Self {
        Self {
            kind,
            path: None,
            context: None,
            cause: None,
        }
    }

    /// Adds a file path to the error for more detailed reporting.
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Adds a context message to the error, describing what was happening.
    pub fn with_context<C, F>(mut self, context: F) -> Self
    where
        C: fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.context = Some(context().to_string());
        self
    }

    /// Adds an underlying cause to the error.
    ///
    /// This is typically used when converting from another error type so that its details are
    /// preserved.
    pub fn with_cause(mut self, cause: impl error::Error + Send + Sync + 'static) -> Self {
        self.cause = Some(Box::new(cause));
        self
    }

    /// Returns the kind of this error.
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    /// Returns the optional file path associated with this error.
    pub fn path(&self) -> &Option<String> {
        &self.path
    }

    /// Returns the optional context message associated with this error.
    pub fn context(&self) -> &Option<String> {
        &self.context
    }
}

impl fmt::Display for Error {
    /// Formats the error into a human-readable string, including
    /// context and file path if available.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.context, &self.path) {
            (Some(ctx), Some(path)) => write!(f, "{} for '{}': {}", ctx, path, self.kind),
            (Some(ctx), None) => write!(f, "{}: {}", ctx, self.kind),
            (None, Some(path)) => write!(f, "An error occurred for '{}': {}", path, self.kind),
            (None, None) => write!(f, "{}", self.kind),
        }
    }
}

impl error::Error for Error {
    /// Returns the underlying cause, if one exists.
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        // Cast the reference to &(dyn Error) (no Send + Sync) to satisfy error::Error
        self.cause
            .as_ref()
            .map(|e| e.as_ref() as &(dyn error::Error + 'static))
    }
}

impl From<io::Error> for Error {
    /// Converts an I/O error into [`Error`], mapping `NotFound` to [`ErrorKind::FileNotFound`].
    fn from(err: io::Error) -> Self {
        let kind = match err.kind() {
            io::ErrorKind::NotFound => ErrorKind::FileNotFound,
            _ => ErrorKind::Io,
        };
        Self::new(kind).with_cause(err)
    }
}

impl From<plist::Error> for Error {
    /// Converts a plist parsing error into [`ErrorKind::Plist`].
    fn from(err: plist::Error) -> Self {
        Self::new(ErrorKind::Plist).with_cause(err)
    }
}

impl From<quick_xml::Error> for Error {
    /// Converts an XML parsing error into [`ErrorKind::Xml`].
    fn from(err: quick_xml::Error) -> Self {
        Self::new(ErrorKind::Xml).with_cause(err)
    }
}

impl From<ParseIntError> for Error {
    /// Converts an integer parsing error into [`ErrorKind::Parse`].
    fn from(err: ParseIntError) -> Self {
        Self::new(ErrorKind::Parse).with_cause(err)
    }
}

impl From<ParseFloatError> for Error {
    /// Converts a floating-point parsing error into [`ErrorKind::Parse`].
    fn from(err: ParseFloatError) -> Self {
        Self::new(ErrorKind::Parse).with_cause(err)
    }
}

impl From<Utf8Error> for Error {
    /// Converts a UTF-8 decoding error into [`ErrorKind::Parse`].
    fn from(err: Utf8Error) -> Self {
        Self::new(ErrorKind::Parse).with_cause(err)
    }
}
