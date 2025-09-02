//! Lazy UFO font data model.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use plist::Value;

use crate::error::Result;
use crate::error::{Error, ErrorKind};
use crate::glif::{Advance, Anchor, GlifData, GlifParser, Guideline, Image, Outline};
use crate::paths::UfoRelativePath;
use crate::provider::Provider;
use crate::provider::ProviderHandle;

/// A UFO font object that loads glyph data on demand.
///
/// This struct stores the `Glyph` objects for each GLIF file and provides lazy, on-demand
/// data access with minimal load time and memory footprint.
///
/// # Requirements
///
/// To use [`Font`], you must provide a concrete implementation of the [`Provider`] trait, which
/// defines how files are read from the UFO file system. See [`crate::provider::FileProvider`] for a
/// simple example providing local disk access.
#[derive(Clone)]
pub struct Font {
    provider: ProviderHandle,
    contents: HashMap<String, String>,
    glyphs: RefCell<HashMap<String, Rc<Glyph>>>,
}

impl Font {
    /// Creates a new [`Font`] object by parsing `contents.plist` via the given provider.
    pub fn new(provider: ProviderHandle) -> Result<Rc<Self>> {
        let contents = crate::plist::parse_contents(provider.clone())?;
        Ok(Rc::new(Self {
            provider,
            contents,
            glyphs: RefCell::new(HashMap::new()),
        }))
    }

    /// Returns a clone of the provider handle used by this font.
    pub fn provider(&self) -> ProviderHandle {
        self.provider.clone()
    }

    /// Returns a clone of the glyph contents map (`contents.plist`).
    pub fn contents(&self) -> HashMap<String, String> {
        self.contents.clone()
    }

    /// Returns a new [`Iter`] object.
    pub fn iter(self: &Rc<Self>) -> Iter {
        Iter::new(Rc::clone(self))
    }

    /// Returns a single glyph by name, if present in the font.
    ///
    /// This ensures that the glyph cache is initialized first.
    pub fn glyph(self: &Rc<Self>, name: &str) -> Option<Rc<Glyph>> {
        // Ensure the glyph map is initialized.
        let mut cache = self.glyphs.borrow_mut();
        Some(
            cache
                .entry(name.to_string())
                .or_insert_with(|| Glyph::new(Rc::clone(self), name.to_string()))
                .clone(),
        )
    }

    /// Returns a map of all glyphs in the font.
    ///
    /// The glyph cache is initialized on first call by creating a [`Glyph`] for each entry in
    /// `contents.plist`.
    pub fn glyphs(self: &Rc<Self>) -> HashMap<String, Rc<Glyph>> {
        let mut glyph_map = self.glyphs.borrow_mut();
        if glyph_map.is_empty() {
            for name in self.contents.keys() {
                let glyph = Glyph::new(Rc::clone(&self), name.clone());
                glyph_map.insert(name.clone(), glyph);
            }
        }
        glyph_map.clone()
    }
}

/// An iterator that yields [`Glyph`] objects on demand.
#[derive(Clone)]
pub struct Iter {
    font: Rc<Font>,
    keys: std::vec::IntoIter<String>,
}

impl Iter {
    pub fn new(font: Rc<Font>) -> Self {
        let keys = font
            .contents
            .keys()
            .cloned()
            .collect::<Vec<_>>()
            .into_iter();

        Self { font, keys }
    }
}

impl Iterator for Iter {
    type Item = Rc<Glyph>;

    fn next(&mut self) -> Option<Self::Item> {
        self.keys.next().map(|name| {
            let mut cache = self.font.glyphs.borrow_mut();
            cache
                .entry(name.clone())
                .or_insert_with(|| Glyph::new(Rc::clone(&self.font), name.clone()))
                .clone()
        })
    }
}

impl IntoIterator for Font {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;
    /// Converts a `Font` into an iterator over its glyph names.
    fn into_iter(self) -> Self::IntoIter {
        // Wrap self in new Rc, since `glyphs` requires Rc<Self>.
        let arc_font = Rc::new(self);
        arc_font
            .glyphs()
            .into_keys()
            .collect::<Vec<_>>()
            .into_iter()
    }
}

/// A lazily loaded glyph within a [`Font`].
///
/// The glyph’s `.glif` data is parsed only when first accessed and then cached for subsequent
/// calls. It holds an [`Rc`] reference to its parent font to resolve glyph data on demand.
#[derive(Clone)]
pub struct Glyph {
    font: Rc<Font>,
    name: String,
    cache: RefCell<Option<GlifData>>,
}

impl Glyph {
    /// Creates a new glyph wrapper with an empty cache.
    pub fn new(font: Rc<Font>, name: String) -> Rc<Self> {
        Rc::new(Self {
            font,
            name,
            cache: RefCell::new(None),
        })
    }

    /// Returns the glyph name.
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Returns the major format version string of the glyph's GLIF file.
    pub fn format(&self) -> Result<String> {
        Ok(self.load()?.format)
    }

    /// Returns the minor format version string of the glyph's GLIF file.
    pub fn format_minor(&self) -> Result<Option<String>> {
        Ok(self.load()?.format_minor)
    }

    /// Returns the glyph's advance width/height data (if any).
    pub fn advance(&self) -> Result<Option<Advance>> {
        Ok(self.load()?.advance)
    }

    /// Returns the glyph's list of Unicode code points.
    pub fn unicodes(&self) -> Result<Vec<u32>> {
        Ok(self.load()?.unicodes)
    }

    /// Returns an optional note attached to the glyph.
    pub fn note(&self) -> Result<Option<String>> {
        Ok(self.load()?.note)
    }

    /// Returns the glyph's associated image, if present.
    pub fn image(&self) -> Result<Option<Image>> {
        Ok(self.load()?.image)
    }

    /// Returns the glyph's guidelines.
    pub fn guidelines(&self) -> Result<Vec<Guideline>> {
        Ok(self.load()?.guidelines)
    }

    /// Returns the glyph's anchors.
    pub fn anchors(&self) -> Result<Vec<Anchor>> {
        Ok(self.load()?.anchors)
    }

    /// Returns the glyph's outline, if present.
    pub fn outline(&self) -> Result<Option<Outline>> {
        Ok(self.load()?.outline)
    }

    /// Returns the glyph's lib dictionary (if present).
    pub fn lib(&self) -> Result<Option<Value>> {
        Ok(self.load()?.lib)
    }

    /// Loads and caches glyph data on first access.
    ///
    /// If the glyph has already been loaded, returns the cached data. Otherwise, parses the
    /// corresponding `.glif` file using the font's provider and stores it for subsequent use.
    fn load(&self) -> Result<GlifData> {
        if let Some(data) = &*self.cache.borrow() {
            return Ok(data.clone());
        }

        let contents = self.font.contents();
        let file_name = contents.get(&self.name).ok_or(
            Error::new(ErrorKind::MissingAttribute(self.name.clone())).with_path(
                self.font
                    .provider()
                    .root()
                    .join(UfoRelativePath::Contents.to_pathbuf())
                    .to_string_lossy(),
            ),
        )?;

        let parser = GlifParser::new(self.font.provider())?;
        let parsed = parser.parse_glif(file_name)?;

        *self.cache.borrow_mut() = Some(parsed.clone());
        Ok(parsed)
    }
}
