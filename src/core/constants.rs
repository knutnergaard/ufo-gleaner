//! Internal and UFO-specific constants.
pub mod ufo {
    pub mod kerning_prefix {
        pub const PUBLIC_PREFIX: &str = "public.";
        pub const PUBLIC_KERN1_PREFIX: &str = "public.kern1.";
        pub const PUBLIC_KERN2_PREFIX: &str = "public.kern2.";
    }
}

pub mod xml {
    pub mod elem {
        pub const ADVANCE: &[u8] = b"advance";
        pub const ANCHOR: &[u8] = b"anchor";
        pub const COMPONENT: &[u8] = b"component";
        pub const CONTOUR: &[u8] = b"contour";
        pub const GLYPH: &[u8] = b"glyph";
        pub const GUIDELINE: &[u8] = b"guideline";
        pub const IMAGE: &[u8] = b"image";
        pub const LIB: &[u8] = b"lib";
        pub const NOTE: &[u8] = b"note";
        pub const OUTLINE: &[u8] = b"outline";
        pub const POINT: &[u8] = b"point";
        pub const UNICODE: &[u8] = b"unicode";
    }
    pub mod attr {
        pub const ANGLE: &[u8] = b"angle";
        pub const BASE: &[u8] = b"base";
        pub const COLOR: &[u8] = b"color";
        pub const FILE_NAME: &[u8] = b"fileName";
        pub const FORMAT: &[u8] = b"format";
        pub const FORMAT_MINOR: &[u8] = b"formatMinor";
        pub const HEIGHT: &[u8] = b"height";
        pub const HEX: &[u8] = b"hex";
        pub const IDENTIFIER: &[u8] = b"identifier";
        pub const NAME: &[u8] = b"name";
        pub const TYPE: &[u8] = b"type";
        pub const SMOOTH: &[u8] = b"smooth";
        pub const WIDTH: &[u8] = b"width";
        pub const X: &[u8] = b"x";
        pub const X_OFFSET: &[u8] = b"xOffset";
        pub const X_SCALE: &[u8] = b"xScale";
        pub const XY_SCALE: &[u8] = b"xyScale";
        pub const Y: &[u8] = b"y";
        pub const Y_OFFSET: &[u8] = b"yOffset";
        pub const Y_SCALE: &[u8] = b"yScale";
        pub const YX_SCALE: &[u8] = b"yxScale";
    }
    pub mod val {
        pub const CURVE: &str = "curve";
        pub const LINE: &str = "line";
        pub const MOVE: &str = "move";
        pub const NO: &str = "no";
        pub const OFFCURVE: &str = "offcurve";
        pub const QCURVE: &str = "qcurve";
        pub const YES: &str = "yes";
    }
}
