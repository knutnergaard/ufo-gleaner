use std::io::Cursor;

use crate::error::Result;
use crate::glif::data::*;

/// Holds the intermediate state while parsing a single GLIF file.
#[derive(Debug, Default)]
pub struct GlifParseContext {
    pub glyph_name: String,
    pub format: String,
    pub format_minor: Option<String>,
    pub advance_width: Option<f64>,
    pub advance_height: Option<f64>,
    pub unicodes: Vec<u32>,
    pub note: String,
    pub inside_note: bool,
    pub image: Option<Image>,
    pub guidelines: Vec<Guideline>,
    pub anchors: Vec<Anchor>,
    pub inside_outline: bool,
    pub components: Vec<Component>,
    pub current_contour: Contour,
    pub all_contours: Vec<Contour>,
    pub lib_buffer: Vec<u8>,
    pub inside_lib: bool,
}

impl GlifParseContext {
    /// Converts the parse context into a finalized [`GlifData`] object.
    pub(crate) fn into_glif_data(self) -> Result<GlifData> {
        let mut data = GlifData::default();

        // glyph
        data.name = self.glyph_name;
        data.format = self.format;
        data.format_minor = self.format_minor;

        // advance
        let advance = match (self.advance_width, self.advance_height) {
            (None, None) => None,
            (w, h) => Some(Advance {
                width: w,
                height: h,
            }),
        };
        data.advance = advance;

        // outline
        let outline = Outline {
            contours: self.all_contours,
            components: self.components,
        };
        data.outline = Some(outline);

        // lib
        let cursor = Cursor::new(self.lib_buffer);
        let value = plist::Value::from_reader_xml(cursor)?;
        data.lib = Some(value);

        // other
        data.unicodes = self.unicodes;
        data.note = Some(self.note);
        data.image = self.image;
        data.guidelines = self.guidelines;
        data.anchors = self.anchors;

        Ok(data)
    }
}
