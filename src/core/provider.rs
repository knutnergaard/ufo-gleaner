//! A file system abstraction and default implementation for reading files in a UFO package.
use std::fs;
use std::io::Read;
use std::path::Path;

use crate::error::Result;

/// Abstracts a read-only file system interface for accessing files relative to
/// a font package root.
///
/// Implementors of this trait provide a `read` method that returns the full
/// contents of a file as a `Vec<u8>`.
pub trait Provider: ProviderClone {
    /// Reads the file at `rel_path` relative to the provider's root and returns
    /// its contents as a `Vec<u8>`.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the file cannot be opened or read.
    fn read(&self, rel_path: &Path) -> Result<Vec<u8>>;
}

/// Provides a method to clone a boxed [`Provider`] trait object.
pub trait ProviderClone {
    fn clone_box(&self) -> Box<dyn Provider>;
}

impl<T> ProviderClone for T
where
    T: 'static + Provider + Clone,
{
    fn clone_box(&self) -> Box<dyn Provider> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Provider> {
    fn clone(&self) -> Box<dyn Provider> {
        self.as_ref().clone_box()
    }
}

impl Provider for Box<dyn Provider> {
    fn read(&self, path: &Path) -> Result<Vec<u8>> {
        (**self).read(path) // deref the Box and delegate
    }
}

/// A simple file-based [`Provider`] that reads files from a root directory.
///
/// Useful for testing or local file system access in UFO parsing.
#[derive(Clone)]
pub struct FileProvider {
    root: std::path::PathBuf,
}

impl FileProvider {
    /// Creates a new `FileProvider` with the given root directory.
    pub fn new<P: Into<std::path::PathBuf>>(root: P) -> Self {
        Self { root: root.into() }
    }
}

impl Provider for FileProvider {
    /// Reads a file relative to the root directory and returns its contents.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the file cannot be opened or read.
    fn read(&self, rel_path: &Path) -> Result<Vec<u8>> {
        let full_path = self.root.join(rel_path);
        let mut file = fs::File::open(full_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    use crate::test_utils::MockProvider;

    #[test]
    fn test_fileprovider_reads_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");

        // Write some content to the file
        let mut f = File::create(&file_path).unwrap();
        write!(f, "Hello, world!").unwrap();

        let provider = FileProvider::new(dir.path());
        let bytes = provider.read(Path::new("test.txt")).unwrap();
        assert_eq!(bytes, b"Hello, world!");
    }

    #[test]
    fn test_fileprovider_file_not_found() {
        let dir = tempdir().unwrap();
        let provider = FileProvider::new(dir.path());

        let err = provider.read(Path::new("missing.txt")).unwrap_err();
        assert_eq!(err.kind(), &crate::error::ErrorKind::FileNotFound);
    }

    #[test]
    fn test_boxed_provider_clone() {
        let mock = Box::new(MockProvider::new().with_file(Path::new("a.txt"), b"abc"));
        let cloned: Box<dyn Provider> = mock.clone();

        let content = cloned.read(Path::new("a.txt")).unwrap();
        assert_eq!(content, b"abc");
    }
}
