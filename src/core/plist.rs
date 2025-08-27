//! Property list (`.plist`) file parser.
use std::io::BufReader;
use std::io::Cursor;
use std::path::Path;

use plist::Value;

use crate::error::Result;
use crate::provider::Provider;

/// Parser for reading and querying property list (`.plist`) files inside a UFO font file system.
pub struct PlistParser {
    provider: Box<dyn Provider>,
}

impl PlistParser {
    /// Creates a new parser from a [`Provider`] instance.
    pub fn new(provider: Box<dyn Provider>) -> Result<Self> {
        Ok(Self { provider })
    }

    /// Reads a plist file at `path` and parses it into a [`Value`].
    pub fn parse_plist(&self, path: &Path) -> Result<Value> {
        let reader = self.open_plist(path)?;
        let value = Value::from_reader(reader)?;
        Ok(value)
    }

    /// Opens a `.plist` file from the UFO file system and returns a buffered reader.
    fn open_plist(&self, path: &Path) -> Result<BufReader<Cursor<Vec<u8>>>> {
        let data = self.provider.read(path)?;
        let cursor = Cursor::new(data);
        let file = BufReader::new(cursor);
        Ok(file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plist::Value;
    use std::collections::BTreeMap;
    use std::path::Path;

    use crate::test_utils::MockProvider;

    #[test]
    fn parses_valid_plist() {
        // Create a tiny plist dictionary
        let mut map = BTreeMap::new();
        map.insert("a".to_string(), Value::String("A".to_string()));

        let dict: plist::Dictionary = map.into_iter().collect();
        let value = Value::Dictionary(dict);

        // Serialize into XML bytes
        let mut plist_bytes = Vec::new();
        plist::to_writer_xml(&mut plist_bytes, &value).unwrap();

        let provider =
            Box::new(MockProvider::new().with_file(Path::new("test.plist"), &plist_bytes));
        let parser = PlistParser::new(provider).unwrap();

        let parsed = parser.parse_plist(Path::new("test.plist")).unwrap();
        match parsed {
            Value::Dictionary(map) => {
                assert_eq!(map.get("a"), Some(&Value::String("A".to_string())));
            }
            _ => panic!("Expected dictionary value"),
        }
    }

    #[test]
    fn returns_error_when_missing_file() {
        let provider = Box::new(MockProvider::new());
        let parser = PlistParser::new(provider).unwrap();
        let err = parser.parse_plist(Path::new("missing.plist")).unwrap_err();
        assert_eq!(err.kind(), &crate::error::ErrorKind::Io);
    }

    #[test]
    fn returns_error_when_invalid_plist() {
        let provider =
            Box::new(MockProvider::new().with_file(Path::new("bad.plist"), b"not valid plist"));
        let parser = PlistParser::new(provider).unwrap();
        let err = parser.parse_plist(Path::new("bad.plist")).unwrap_err();
        assert_eq!(err.kind(), &crate::error::ErrorKind::Plist);
    }
}
