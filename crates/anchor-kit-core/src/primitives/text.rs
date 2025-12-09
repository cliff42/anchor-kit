use crate::primitives::color::Color;

#[derive(Clone, Debug)]
pub struct Text {
    pub text: String,

    pub position: [u32; 2], // x, y
    pub size: [u32; 2],     // w, h

    // TODO: we can add a style that directly influences things like glyphon font weight, type etc.
    pub color: Color, // TODO: add things like font etc
}
