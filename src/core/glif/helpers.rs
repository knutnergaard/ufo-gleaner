//! Attribute & parsing helpers.
use std::io::Cursor;

use quick_xml::Writer;
use quick_xml::events::{BytesStart, Event};

use crate::constants::xml::{attr, val};
use crate::error::{Error, ErrorKind, Result};
use crate::glif::data::{Anchor, Component, Guideline, Image, Point, PointType};
use crate::glif::helpers;

/// Retrieves the value of a specific attribute as a [`String`].
pub fn attr_str(e: &BytesStart, key: &[u8]) -> Result<Option<String>> {
    for attr in e.attributes().with_checks(false).flatten() {
        if attr.key.as_ref() == key {
            let s = attr.unescape_value()?.into_owned();
            return Ok(Some(s));
        }
    }
    Ok(None)
}

/// Retrieves the value of a numeric attribute and parses it as [`f64`].
pub fn attr_f64(e: &BytesStart, key: &[u8]) -> Result<Option<f64>> {
    for attr in e.attributes().with_checks(false).flatten() {
        if attr.key.as_ref() == key {
            let s = std::str::from_utf8(&attr.value)?;
            return Ok(Some(s.parse()?));
        }
    }
    Ok(None)
}

/// Parses a `<unicode>` element into a [`Unicode`] value.
pub fn parse_unicode(e: &BytesStart) -> Result<u32> {
    let attr = attr::HEX;
    let hex = helpers::attr_str(e, attr)?.ok_or_else(|| {
        Error::new(ErrorKind::MissingAttribute(
            std::str::from_utf8(attr).unwrap().to_string(),
        ))
    })?;

    u32::from_str_radix(&hex, 16).map_err(Error::from)
}

/// Parses an `<image>` element into a [`Image`] object.
pub fn parse_image(e: &BytesStart) -> Result<Image> {
    let image = Image {
        file_name: helpers::attr_str(e, attr::FILE_NAME)?,
        x_scale: helpers::attr_f64(e, attr::X_SCALE)?,
        xy_scale: helpers::attr_f64(e, attr::XY_SCALE)?,
        yx_scale: helpers::attr_f64(e, attr::YX_SCALE)?,
        y_scale: helpers::attr_f64(e, attr::Y_SCALE)?,
        x_offset: helpers::attr_f64(e, attr::X_OFFSET)?,
        y_offset: helpers::attr_f64(e, attr::Y_OFFSET)?,
        color: helpers::attr_str(e, attr::COLOR)?,
    };
    Ok(image)
}

/// Parses a `<guideline>` element into a [`Guideline`] object.
pub fn parse_guideline(e: &BytesStart) -> Result<Guideline> {
    let guideline = Guideline {
        x: helpers::attr_f64(e, attr::X)?,
        y: helpers::attr_f64(e, attr::Y)?,
        angle: helpers::attr_f64(e, attr::ANGLE)?,
        name: helpers::attr_str(e, attr::NAME)?,
        color: helpers::attr_str(e, attr::COLOR)?,
        identifier: helpers::attr_str(e, attr::IDENTIFIER)?,
    };

    Ok(guideline)
}

/// Parses an `<anchor>` element into a [`Anchor`] object.
pub fn parse_anchor(e: &BytesStart) -> Result<Anchor> {
    let anchor = Anchor {
        x: helpers::attr_f64(e, attr::X)?,
        y: helpers::attr_f64(e, attr::Y)?,
        name: helpers::attr_str(e, attr::NAME)?,
        color: helpers::attr_str(e, attr::COLOR)?,
        identifier: helpers::attr_str(e, attr::IDENTIFIER)?,
    };

    Ok(anchor)
}

/// Parses a `<point>` element into a [`Point`] object.
pub fn parse_point(e: &BytesStart) -> Result<Point> {
    let mut point = Point::default();

    point.x = helpers::attr_f64(e, attr::X)?;
    point.y = helpers::attr_f64(e, attr::Y)?;

    let point_type = helpers::attr_str(e, attr::TYPE)?.and_then(|t| match t.as_str() {
        val::MOVE => Some(PointType::Move),
        val::LINE => Some(PointType::Line),
        val::CURVE => Some(PointType::Curve),
        val::QCURVE => Some(PointType::QCurve),
        val::OFFCURVE => Some(PointType::OffCurve),
        _ => None,
    });
    point.point_type = point_type;

    let smooth = helpers::attr_str(&e, attr::SMOOTH)?.and_then(|v| match v.as_str() {
        val::YES => Some(true),
        val::NO => Some(false),
        _ => None,
    });
    point.smooth = smooth;

    let name = helpers::attr_str(&e, attr::NAME)?;
    point.name = name;

    let identifier = helpers::attr_str(&e, attr::IDENTIFIER)?;
    point.identifier = identifier;

    Ok(point)
}

/// Parses a `<component>` element into a [`Component`] object.
pub fn parse_component(e: &BytesStart) -> Result<Component> {
    let component = Component {
        base: helpers::attr_str(e, attr::BASE)?,
        x_scale: helpers::attr_f64(e, attr::X_SCALE)?,
        xy_scale: helpers::attr_f64(e, attr::XY_SCALE)?,
        yx_scale: helpers::attr_f64(e, attr::YX_SCALE)?,
        y_scale: helpers::attr_f64(e, attr::Y_SCALE)?,
        x_offset: helpers::attr_f64(e, attr::X_OFFSET)?,
        y_offset: helpers::attr_f64(e, attr::Y_OFFSET)?,
        identifier: helpers::attr_str(e, attr::IDENTIFIER)?,
    };

    Ok(component)
}

/// Normalizes a note string by standardizing line endings, trimming whitespace,
/// and removing empty lines.
pub fn normalize_note(note: &str) -> String {
    note.replace("\r\n", "\n") // normalize Windows endings first
        .replace('\r', "\n") // normalize any lone \r
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Turns an [`Event`] into XML bytes.
pub fn serialize_event(ev: &Event) -> Vec<u8> {
    let buffer = Cursor::new(Vec::new());
    let mut writer = Writer::new(buffer);
    writer.write_event(ev.to_owned()).unwrap_or_default();
    writer.into_inner().into_inner()
}
