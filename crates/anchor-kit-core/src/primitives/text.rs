use crate::style::TextStyle;

#[derive(Clone, Debug)]
pub struct Text {
    pub text: String,
    pub position: [u32; 2], // x, y
    pub size: [u32; 2],     // w, h
    pub text_style: TextStyle,
}
