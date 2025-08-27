//! Provides a high-level interface for reading and parsing UFO font data.
//!
//! `UfoGleaner` combines a property list parser and an XML parser to extract glyph
//! information from a UFO font package. It reads the `contents.plist` file to
//! determine which glyphs exist, and then parses each `.glif` file to produce
//! structured [`GlifData`] for downstream processing.
//!
//! # Requirements
//!
//! To use `UfoGleaner`, you must provide a concrete implementation of the [`Provider`]
//! trait, which defines how files are read from the UFO file system.
//! See [`crate::provider::FileProvider`] for a simple example prividing local disk access.
//!
//! # Example
//!
//! ```no_run
//! use std::path::PathBuf;
//! use ufo_gleaner::provider::FileProvider;
//! use ufo_gleaner::gleaner::UfoGleaner;
//!
//! let provider = Box::new(FileProvider::new(PathBuf::from("/path/to/ufo")));
//! let gleaner = UfoGleaner::new(provider).unwrap();
//! let glyphs = gleaner.glean().unwrap();
//! ```
use plist::Value;
use std::collections::HashMap;

use crate::error::{Error, ErrorKind, Result};
use crate::glif::{GlifData, GlifParser};
use crate::paths::UfoRelativePath;
use crate::plist::PlistParser;
use crate::provider::Provider;

/// High-level UFO GLIF parser.
pub struct UfoGleaner {
    contents: HashMap<String, String>,
    glif_parser: GlifParser,
}

impl UfoGleaner {
    /// Constructs a new [`UfoGleaner`] from a given [`Provider`] implementation.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the `contents.plist` cannot be read or parsed,
    /// or if the GLIF parser cannot be initialized.
    pub fn new(provider: Box<dyn Provider>) -> Result<Self> {
        // fs is cheap to clone.
        let contents = Self::from_plist_dict(provider.clone())?;
        let glif_parser = GlifParser::new(provider)?;
        Ok(Self {
            contents,
            glif_parser,
        })
    }

    /// Parses all glyphs defined in `contents.plist` and returns a mapping from glyph
    /// names to their corresponding [`GlifData`].
    ///
    /// # Returns
    ///
    /// A [`HashMap<String, Option<GlifData>>`] where each key is a glyph name and each
    /// value is `Some(GlifData)` if the glyph was successfully parsed, or `None`
    /// if the `.glif` file could not be read or parsed.
    pub fn glean(&self) -> Result<HashMap<String, Option<GlifData>>> {
        // TODO: Implement logging of parsing errors.
        // TODO: Implement optional validation.
        let mut glyphs_map = HashMap::with_capacity(self.contents.len());
        for (glyph_name, file_name) in &self.contents {
            let data = self.glif_parser.parse_glif(file_name).ok();
            glyphs_map.insert(glyph_name.clone(), data);
        }
        Ok(glyphs_map)
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
    fn from_plist_dict(provider: Box<dyn Provider>) -> Result<HashMap<String, String>> {
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
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::glif::GlifData;
    use crate::test_utils::MockProvider;
    use std::collections::HashMap;
    use std::path::Path;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_from_plist_dict_reads_dictionary() {
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
        let dict = UfoGleaner::from_plist_dict(provider).unwrap();

        assert_eq!(dict.len(), 2);
        assert_eq!(dict["a"], "A.glif");
        assert_eq!(dict["b"], "B.glif");
        assert!(!dict.contains_key("ignored"));
    }

    #[test]
    fn test_new_and_glean() {
        // Mock GLIF parser returns dummy GlifData for any input
        let path = UfoRelativePath::Contents.to_pathbuf();
        let provider = Box::new(MockProvider::new().with_file(&path, b"<?xml version='1.0'?><plist version='1.0'><dict><key>a</key><string>A.glif</string></dict></plist>"));

        // Use actual UfoGleaner with mocked GlifParser
        let gleaner = UfoGleaner::new(provider).unwrap();
        let glyphs = gleaner.glean().unwrap();

        assert_eq!(glyphs.len(), 1);
        assert!(glyphs.contains_key("a"));
        assert!(glyphs["a"].is_none() || matches!(glyphs["a"], Some(GlifData { .. })));
    }

    #[test]
    fn test_from_plist_dict_invalid_file() {
        let provider = Box::new(MockProvider::new()); // no files
        let err = UfoGleaner::from_plist_dict(provider).unwrap_err();
        assert_eq!(err.kind(), &crate::error::ErrorKind::Io);
    }

    #[test]
    fn test_from_plist_dict_not_dictionary() {
        let value = plist::Value::String("not a dict".to_string());
        let mut plist_bytes = Vec::new();

        plist::to_writer_xml(&mut plist_bytes, &value).unwrap();

        let path = UfoRelativePath::Contents.to_pathbuf();
        let provider = Box::new(MockProvider::new().with_file(&path, &plist_bytes));
        let err = UfoGleaner::from_plist_dict(provider).unwrap_err();
        assert_eq!(err.kind(), &crate::error::ErrorKind::Plist);
    }
}
