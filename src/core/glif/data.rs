use plist::Value;
use serde::Serialize;
use std::default::Default;

/// Represents all data contained in a single `.glif` glyph file.
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GlifData {
    pub name: String,
    pub format: String,
    pub format_minor: Option<String>,
    pub advance: Option<Advance>,
    pub unicodes: Vec<u32>,
    pub note: Option<String>,
    pub image: Option<Image>,
    pub guidelines: Vec<Guideline>,
    pub anchors: Vec<Anchor>,
    pub outline: Option<Outline>,
    pub lib: Option<Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
/// Represents advance width/height information for a glyph.
pub struct Advance {
    pub width: Option<f64>,
    pub height: Option<f64>,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
/// Represents optional image data embedded in a glyph.
pub struct Image {
    pub file_name: Option<String>,
    pub x_scale: Option<f64>,
    pub xy_scale: Option<f64>,
    pub yx_scale: Option<f64>,
    pub y_scale: Option<f64>,
    pub x_offset: Option<f64>,
    pub y_offset: Option<f64>,
    pub color: Option<String>,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
/// Represents a guideline element within a glyph.
pub struct Guideline {
    pub y: Option<f64>,
    pub x: Option<f64>,
    pub angle: Option<f64>,
    pub name: Option<String>,
    pub color: Option<String>,
    pub identifier: Option<String>,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
/// Represents an anchor point in the glyph.
pub struct Anchor {
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub name: Option<String>,
    pub color: Option<String>,
    pub identifier: Option<String>,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
/// Represents the full outline of a glyph: components + contours.
pub struct Outline {
    pub components: Vec<Component>,
    pub contours: Vec<Contour>,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
/// Represents a single component reference within an outline.
pub struct Component {
    pub base: Option<String>,
    pub x_scale: Option<f64>,
    pub xy_scale: Option<f64>,
    pub yx_scale: Option<f64>,
    pub y_scale: Option<f64>,
    pub x_offset: Option<f64>,
    pub y_offset: Option<f64>,
    pub identifier: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
/// Represents a contour composed of individual points.
pub struct Contour {
    pub identifier: Option<String>,
    pub points: Vec<Point>,
}

#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
/// Represents a single point in a glyph contour.
pub struct Point {
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub point_type: Option<PointType>,
    pub smooth: Option<bool>,
    pub name: Option<String>,
    pub identifier: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
/// Enumeration of valid point types in a glyph contour.
pub enum PointType {
    Move,
    Line,
    OffCurve,
    Curve,
    QCurve,
}
