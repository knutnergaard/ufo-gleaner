//! Defines an enumeration of standard relative file paths inside a UFO package.
//!
//! Each variant corresponds to a well-known file or folder in the UFO structure.
//! The `to_pathbuf` method converts the variant into a [`PathBuf`] relative to the
//! root of the UFO package.
use std::path::PathBuf;

/// Represents a file or directory path relative to the root of a UFO font package.
pub enum UfoRelativePath {
    Contents,
    FontInfo,
    Groups,
    Kerning,
    LayerContents,
    LayerInfo,
    Lib,
    MetaInfo,
    GlifFile(String), // for individual glyphs, e.g. "A.glif"
}

impl UfoRelativePath {
    /// Returns a [`PathBuf`] relative to the UFO root.
    pub fn to_pathbuf(&self) -> PathBuf {
        match self {
            UfoRelativePath::Contents => PathBuf::from("glyphs").join("contents.plist"),
            UfoRelativePath::FontInfo => PathBuf::from("fontinfo.plist"),
            UfoRelativePath::Groups => PathBuf::from("groups.plist"),
            UfoRelativePath::Kerning => PathBuf::from("kerning.plist"),
            UfoRelativePath::LayerContents => PathBuf::from("layercontents.plist"),
            UfoRelativePath::LayerInfo => PathBuf::from("glyphs").join("layerinfo.plist"),
            UfoRelativePath::Lib => PathBuf::from("lib.plist"),
            UfoRelativePath::MetaInfo => PathBuf::from("metainfo.plist"),
            UfoRelativePath::GlifFile(name) => PathBuf::from("glyphs").join(format!("{}", name)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_standard_paths() {
        assert_eq!(
            UfoRelativePath::Contents.to_pathbuf(),
            Path::new("glyphs/contents.plist")
        );
        assert_eq!(
            UfoRelativePath::FontInfo.to_pathbuf(),
            Path::new("fontinfo.plist")
        );
        assert_eq!(
            UfoRelativePath::Groups.to_pathbuf(),
            Path::new("groups.plist")
        );
        assert_eq!(
            UfoRelativePath::Kerning.to_pathbuf(),
            Path::new("kerning.plist")
        );
        assert_eq!(
            UfoRelativePath::LayerContents.to_pathbuf(),
            Path::new("layercontents.plist")
        );
        assert_eq!(
            UfoRelativePath::LayerInfo.to_pathbuf(),
            Path::new("glyphs/layerinfo.plist")
        );
        assert_eq!(UfoRelativePath::Lib.to_pathbuf(), Path::new("lib.plist"));
        assert_eq!(
            UfoRelativePath::MetaInfo.to_pathbuf(),
            Path::new("metainfo.plist")
        );
    }

    #[test]
    fn test_glif_file_path() {
        let path = UfoRelativePath::GlifFile("A.glif".to_string()).to_pathbuf();
        assert_eq!(path, Path::new("glyphs/A.glif"));
    }
}
