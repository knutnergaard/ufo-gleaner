use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;

use crate::provider::Provider;

#[derive(Clone)]
pub struct MockProvider {
    root: PathBuf,
    files: Rc<RefCell<HashMap<String, Vec<u8>>>>,
}

impl MockProvider {
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            root: PathBuf::new(),
            files: Rc::new(RefCell::new(HashMap::new())),
        })
    }

    pub fn with_file(&self, path: &Path, content: &[u8]) -> &Self {
        self.files
            .borrow_mut()
            .insert(path.display().to_string(), content.to_vec());
        self
    }
}

impl Provider for MockProvider {
    fn root(&self) -> &Path {
        &self.root
    }
    fn read(&self, rel_path: &Path) -> crate::error::Result<Vec<u8>> {
        let files = self.files.borrow_mut();
        files
            .get(&rel_path.to_string_lossy().to_string())
            .cloned()
            .ok_or_else(|| {
                crate::error::Error::new(crate::error::ErrorKind::Io)
                    .with_context(|| format!("file not found: {}", rel_path.display()))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mockprovider_read() {
        let provider = MockProvider::new();
        provider
            .with_file(Path::new("foo.txt"), b"123")
            .with_file(Path::new("bar.txt"), b"xyz");

        let content = provider.read(Path::new("foo.txt")).unwrap();
        assert_eq!(content, b"123");

        let content2 = provider.read(Path::new("bar.txt")).unwrap();
        assert_eq!(content2, b"xyz");

        let err = provider.read(Path::new("missing.txt")).unwrap_err();
        assert_eq!(err.kind(), &crate::error::ErrorKind::Io);
    }
}
