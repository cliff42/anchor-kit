use crate::style::Style;

#[derive(Clone, Copy, Debug)]
pub struct Rectangle {
    pub position: [u32; 2], // x, y
    pub size: [u32; 2],     // w, h
    pub style: Style,
}

impl Rectangle {
    pub fn new(position: [u32; 2], size: [u32; 2], style: Option<Style>) -> Self {
        Self {
            position,
            size,
            style: style.unwrap_or_default(),
        }
    }
}
