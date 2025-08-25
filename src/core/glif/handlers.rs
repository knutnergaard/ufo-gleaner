//! Handlers for parsing UFO GLIF XML elements into a [`GlifParseContext`].
use quick_xml::events::{BytesStart, BytesText};

use crate::constants::xml::attr;
use crate::error::Result;
use crate::glif::{context::GlifParseContext, data::Contour, helpers};

// ----- Glyph -----

/// Handles a `<glyph>` start element and stores the glyph name.
pub fn handle_glyph_start(ctx: &mut GlifParseContext, e: BytesStart) -> Result<()> {
    for attr in e.attributes().with_checks(false).flatten() {
        match attr.key.as_ref() {
            attr::NAME => {
                ctx.glyph_name = attr.unescape_value()?.into_owned();
            }
            attr::FORMAT => {
                ctx.format = attr.unescape_value()?.into_owned();
            }
            attr::FORMAT_MINOR => {
                ctx.format = attr.unescape_value()?.into_owned();
            }
            _ => {}
        }
    }
    Ok(())
}

// ----- Advance & Unicode -----

/// Parses an `<advance>` element and stores its width and height.
pub fn handle_advance(ctx: &mut GlifParseContext, e: BytesStart) -> Result<()> {
    ctx.advance_width = helpers::attr_f64(&e, attr::WIDTH)?;
    ctx.advance_height = helpers::attr_f64(&e, attr::HEIGHT)?;

    Ok(())
}

/// Parses a `<unicode>` element and appends it to the glyph's unicode list.
pub fn handle_unicode(ctx: &mut GlifParseContext, e: BytesStart) -> Result<()> {
    let unicode = helpers::parse_unicode(&e)?;
    ctx.unicodes.push(unicode);

    Ok(())
}

// ----- Note -----

/// Marks that the parser has entered a `<note>` element.
pub fn handle_note_start(ctx: &mut GlifParseContext) -> Result<()> {
    ctx.inside_note = true;
    ctx.note.clear();

    Ok(())
}

// Parses the text inside a `<note>` element.
pub fn handle_note_contents(ctx: &mut GlifParseContext, e: BytesText) -> Result<()> {
    if ctx.inside_note {
        let text = std::str::from_utf8(e.as_ref())?;
        ctx.note.push_str(text);
    }
    Ok(())
}

/// Marks that the parser has exited an `<note>` element.
pub fn handle_note_end(ctx: &mut GlifParseContext) -> Result<()> {
    helpers::normalize_note(&ctx.note);
    ctx.inside_note = false;

    Ok(())
}

// ----- Image, Guideline & Anchor -----

/// Parses an `<image>` element and stores its attributes.
pub fn handle_image(ctx: &mut GlifParseContext, e: BytesStart) -> Result<()> {
    let image = helpers::parse_image(&e)?;
    ctx.image = Some(image);

    Ok(())
}

/// Parses a `<guideline>` element and stores its attributes.
pub fn handle_guideline(ctx: &mut GlifParseContext, e: BytesStart) -> Result<()> {
    let guideline = helpers::parse_guideline(&e)?;
    ctx.guidelines.push(guideline);

    Ok(())
}

/// Parses an `<anchor>` element and stores its attributes.
pub fn handle_anchor(ctx: &mut GlifParseContext, e: BytesStart) -> Result<()> {
    let anchor = helpers::parse_anchor(&e)?;
    ctx.anchors.push(anchor);

    Ok(())
}

// ----- Outline, Contours & Componenets -----

/// Marks that the parser has entered an `<outline>` element.
pub fn handle_outline_start(ctx: &mut GlifParseContext) -> Result<()> {
    ctx.inside_outline = true;

    Ok(())
}

/// Marks that the parser has exited an `<outline>` element.
pub fn handle_outline_end(ctx: &mut GlifParseContext) -> Result<()> {
    ctx.inside_outline = false;

    Ok(())
}

/// Parses a `<point>` element and adds it to the current contour.
pub fn handle_point(ctx: &mut GlifParseContext, e: BytesStart) -> Result<()> {
    if ctx.inside_outline {
        let point = helpers::parse_point(&e)?;
        ctx.current_contour.points.push(point);
    }
    Ok(())
}

/// Starts a new `<contour>` element and initializes the current contour.
pub fn handle_contour_start(ctx: &mut GlifParseContext, e: BytesStart) -> Result<()> {
    if ctx.inside_outline {
        let current_contour = Contour {
            identifier: helpers::attr_str(&e, attr::IDENTIFIER)?,
            points: Vec::new(),
        };
        ctx.current_contour = current_contour;
    }
    Ok(())
}

/// Completes the current `<contour>` element and appends it to the list of all contours.
pub fn handle_contour_end(ctx: &mut GlifParseContext) -> Result<()> {
    if ctx.inside_outline {
        let current_contour = std::mem::take(&mut ctx.current_contour);
        ctx.all_contours.push(current_contour);
    }

    Ok(())
}

/// Parses a `<component>` element and stores its attributes.
pub fn handle_component(ctx: &mut GlifParseContext, e: BytesStart) -> Result<()> {
    let component = helpers::parse_component(&e)?;
    ctx.components.push(component);

    Ok(())
}

// ----- Lib -----

/// Marks that the parser has entered a `<lib>` element.
pub fn handle_lib_start(ctx: &mut GlifParseContext) -> Result<()> {
    ctx.inside_lib = true;

    Ok(())
}

// Extracts the elements inside a `<note>` element.
pub fn handle_lib_contents(ctx: &mut GlifParseContext, e: &[u8]) -> Result<()> {
    ctx.lib_buffer.extend_from_slice(e);

    Ok(())
}

/// Marks that the parser has exited an `<lib>` element.
pub fn handle_lib_end(ctx: &mut GlifParseContext) -> Result<()> {
    ctx.inside_lib = false;

    Ok(())
}
