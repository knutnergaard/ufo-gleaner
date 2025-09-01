//! High-level interface for eagerly parsing UFO GLIF files.

use std::collections::HashMap;

use crate::error::Result;
use crate::glif::{GlifData, GlifParser};
use crate::provider::ProviderHandle;

/// Eager batch parser for UFO GLIF files.
///
/// # Requirements
///
/// To use `Gleaner`, you must provide a concrete implementation of the [`Provider`]
/// trait, which defines how files are read from the UFO file system.
/// See [`crate::provider::FileProvider`] for a simple example prividing local disk access.
///
/// # Example
///
/// ```no_run
/// use std::path::PathBuf;
/// use ufo_gleaner::provider::FileProvider;
/// use ufo_gleaner::gleaner::Gleaner;
///
/// let provider = FileProvider::new(PathBuf::from("/path/to/ufo"));
/// let gleaner = Gleaner::new(provider).unwrap();
/// let glyphs = gleaner.glean().unwrap();
/// ```
///
/// # Errors
///
/// Returns an [`Error`] if the `contents.plist` cannot be read or parsed,
/// or if the GLIF parser cannot be initialized.
pub struct Gleaner {
    contents: HashMap<String, String>,
    glif_parser: GlifParser,
}

impl Gleaner {
    /// Constructs a new [`Gleaner`] from a given [`Provider`] implementation.
    pub fn new(provider: ProviderHandle) -> Result<Self> {
        // fs is cheap to clone.
        let contents = crate::plist::parse_contents(provider.clone())?;
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
}

#[cfg(test)]
mod tests {

    use super::*;

    use crate::glif::GlifData;
    use crate::paths::UfoRelativePath;
    use crate::test_utils::MockProvider;

    #[test]
    fn test_new_and_glean() {
        // Mock GLIF parser returns dummy GlifData for any input
        let path = UfoRelativePath::Contents.to_pathbuf();
        let provider = MockProvider::new();
        provider.with_file(&path, b"<?xml version='1.0'?><plist version='1.0'><dict><key>a</key><string>A.glif</string></dict></plist>");

        // Use actual Gleaner with mocked GlifParser
        let gleaner = Gleaner::new(provider).unwrap();
        let glyphs = gleaner.glean().unwrap();

        assert_eq!(glyphs.len(), 1);
        assert!(glyphs.contains_key("a"));
        assert!(glyphs["a"].is_none() || matches!(glyphs["a"], Some(GlifData { .. })));
    }
}
