//! Property list (`.plist`) file parser.
use std::collections::HashMap;
use std::io::BufReader;
use std::io::Cursor;
use std::path::Path;

use plist::Value;

use crate::error::{Error, ErrorKind, Result};
use crate::paths::UfoRelativePath;
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

/// Reads `contents.plist` from the UFO package and converts it into a mapping
/// from glyph names to `.glif` file names.
///
/// Only entries where the value is a string are included; other types are ignored.
///
/// # Errors
///
/// Returns an [`Error`] if `contents.plist` cannot be read, is not a [`plist::Dictionary`],
/// or if parsing fails for other reasons.
pub fn parse_contents(provider: Box<dyn Provider>) -> Result<HashMap<String, String>> {
    let plist_parser = PlistParser::new(provider)?;
    let contents_path = UfoRelativePath::Contents.to_pathbuf();
    let plist_value = plist_parser.parse_plist(contents_path.as_ref())?;
    let contents: HashMap<String, String> = match plist_value {
        Value::Dictionary(dict) => dict
            .into_iter()
            .filter_map(|(k, v)| {
                if let Value::String(s) = v {
                    Some((k, s))
                } else {
                    None
                }
            })
            .collect(),
        _ => {
            return Err(Error::new(ErrorKind::Plist)
                .with_context(|| "contents.plist is not a dictionary")
                .with_path(contents_path.to_string_lossy()));
        }
    };
    Ok(contents)
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

    #[test]
    fn test_parse_contents_reads_dictionary() {
        // contents.plist: {"a": "A.glif", "b": "B.glif", "ignored": 123}
        let mut plist_bytes = Vec::new();
        let value = plist::Value::Dictionary(
            [
                ("a".to_string(), plist::Value::from("A.glif".to_string())),
                ("b".to_string(), plist::Value::from("B.glif".to_string())),
                ("ignored".to_string(), plist::Value::from(123)),
            ]
            .iter()
            .cloned()
            .collect(),
        );
        plist::to_writer_xml(&mut plist_bytes, &value).unwrap();

        let path = UfoRelativePath::Contents.to_pathbuf();
        let provider = Box::new(MockProvider::new().with_file(&path, &plist_bytes));
        let dict = parse_contents(provider).unwrap();

        assert_eq!(dict.len(), 2);
        assert_eq!(dict["a"], "A.glif");
        assert_eq!(dict["b"], "B.glif");
        assert!(!dict.contains_key("ignored"));
    }

    #[test]
    fn test_parse_contents_invalid_file() {
        let provider = Box::new(MockProvider::new()); // no files
        let err = parse_contents(provider).unwrap_err();
        assert_eq!(err.kind(), &crate::error::ErrorKind::Io);
    }

    #[test]
    fn test_parse_contents_not_dictionary() {
        let value = plist::Value::String("not a dict".to_string());
        let mut plist_bytes = Vec::new();

        plist::to_writer_xml(&mut plist_bytes, &value).unwrap();

        let path = UfoRelativePath::Contents.to_pathbuf();
        let provider = Box::new(MockProvider::new().with_file(&path, &plist_bytes));
        let err = parse_contents(provider).unwrap_err();
        assert_eq!(err.kind(), &crate::error::ErrorKind::Plist);
    }
}
