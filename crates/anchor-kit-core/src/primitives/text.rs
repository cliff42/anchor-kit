use crate::{primitives::color::Color, style::TextStyle};

#[derive(Clone, Debug)]
pub struct Text {
    pub text: String,
    pub position: [u32; 2], // x, y
    pub size: [u32; 2],     // w, h
    pub color: Color,       // TODO: should this be a part of text style?
    pub text_style: TextStyle,
}
