use crate::primitives::color::Color;

#[derive(Clone, Copy, Debug)]
pub struct Rectangle {
    pub position: [u32; 2], // x, y
    pub size: [u32; 2],     // w, h
    pub color: Color,
}

impl Rectangle {
    pub fn new(position: [u32; 2], size: [u32; 2]) -> Self {
        Self { position, size }
    }
}
