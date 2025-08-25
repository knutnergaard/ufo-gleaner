//! Streaming XML parser to convert UFO GLIF files into [`GlifData`].
use std::io::{BufReader, Cursor};
use std::path::PathBuf;

use quick_xml::events::Event;

use crate::constants::xml::elem;
use crate::error::Result;
use crate::glif::{GlifData, GlifEventReader, GlifParseContext, handlers, helpers};

use crate::paths::UfoRelativePath;
use crate::provider::Provider;

/// A parser for UFO GLIF files that produces [`GlifData`] from `.glif` files.
pub struct GlifParser {
    provider: Box<dyn Provider>,
}

impl GlifParser {
    /// Creates a new parser from a [`Provider`] instance.
    pub fn new(provider: Box<dyn Provider>) -> Result<Self> {
        Ok(Self { provider })
    }

    /// Parses a single GLIF file and returns the corresponding [`GlifData`].
    pub fn parse_glif(&self, file_name: &str) -> Result<GlifData> {
        let path = UfoRelativePath::GlifFile(file_name.to_string()).to_pathbuf();
        let file = self.open_xml(&path)?;
        let mut ctx = GlifParseContext::default();

        for ev in GlifEventReader::new(file) {
            let ev = ev?;
            if ctx.inside_lib {
                match ev {
                    Event::End(e) if e.name().as_ref() == elem::LIB => {
                        handlers::handle_lib_end(&mut ctx)?
                    }
                    _ => {
                        let bytes = helpers::serialize_event(&ev);
                        handlers::handle_lib_contents(&mut ctx, &bytes)?
                    }
                }
            } else {
                match ev {
                    // ----- Glyph -----
                    Event::Start(e) if e.name().as_ref() == elem::GLYPH => {
                        handlers::handle_glyph_start(&mut ctx, e)?
                    }

                    // ----- Advance & Unicode -----
                    Event::Empty(e) if e.name().as_ref() == elem::ADVANCE => {
                        handlers::handle_advance(&mut ctx, e)?
                    }
                    Event::Empty(e) if e.name().as_ref() == elem::UNICODE => {
                        handlers::handle_unicode(&mut ctx, e)?
                    }

                    // ----- Note & Text -----
                    Event::Start(e) if e.name().as_ref() == elem::NOTE => {
                        handlers::handle_note_start(&mut ctx)?
                    }
                    Event::Text(e) => handlers::handle_note_contents(&mut ctx, e)?,
                    Event::End(e) if e.name().as_ref() == elem::NOTE => {
                        handlers::handle_note_end(&mut ctx)?
                    }

                    // ----- Image, Guideline & Anchor -----
                    Event::Empty(e) if e.name().as_ref() == elem::IMAGE => {
                        handlers::handle_image(&mut ctx, e)?
                    }
                    Event::Empty(e) if e.name().as_ref() == elem::GUIDELINE => {
                        handlers::handle_guideline(&mut ctx, e)?
                    }
                    Event::Empty(e) if e.name().as_ref() == elem::ANCHOR => {
                        handlers::handle_anchor(&mut ctx, e)?
                    }

                    // ----- Outline, Contours & Componenets -----
                    Event::Start(e) if e.name().as_ref() == elem::OUTLINE => {
                        handlers::handle_outline_start(&mut ctx)?
                    }
                    Event::End(e) if e.name().as_ref() == elem::OUTLINE => {
                        handlers::handle_outline_end(&mut ctx)?
                    }
                    Event::Empty(e) if e.name().as_ref() == elem::POINT => {
                        handlers::handle_point(&mut ctx, e)?
                    }
                    Event::Start(e) if e.name().as_ref() == elem::CONTOUR => {
                        handlers::handle_contour_start(&mut ctx, e)?
                    }
                    Event::End(e) if e.name().as_ref() == elem::CONTOUR => {
                        handlers::handle_contour_end(&mut ctx)?
                    }
                    Event::Empty(e) if e.name().as_ref() == elem::COMPONENT => {
                        handlers::handle_component(&mut ctx, e)?
                    }

                    // ----- Lib -----
                    Event::Start(e) if e.name().as_ref() == elem::LIB => {
                        handlers::handle_lib_start(&mut ctx)?
                    }

                    // ----- Unknown -----
                    _ => {}
                }
            }
        }
        Ok(ctx.into_glif_data()?)
    }

    /// Opens a GLIF file as a buffered reader from the UFO file system.
    fn open_xml(&self, path: &PathBuf) -> Result<BufReader<Cursor<Vec<u8>>>> {
        let data = self.provider.read(path)?;
        let cursor = Cursor::new(data);
        let reader = BufReader::new(cursor);

        Ok(reader)
    }
}
