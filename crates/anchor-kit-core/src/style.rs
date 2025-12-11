use crate::primitives::color::Color;

#[derive(Debug, Clone, Copy)]
pub enum SizingPolicy {
    Auto,       // hug to child elements
    Fixed(u32), // individual policy for width and height so only need 1 u32 here
    FillParent, // take up entire space of parent element
}

#[derive(Debug, Clone, Copy)]
pub enum Align {
    Start, // left for rows, top for cols
    Middle,
    End, // right for rows, bottom for cols
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Insets {
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
    pub left: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct Style {
    pub padding: Insets,
    pub margin: Insets,
    pub width: SizingPolicy,
    pub height: SizingPolicy,
    pub align_x: Align, // element alignment
    pub align_y: Align,
    pub justify_x: Align, // content within element alignment
    pub justify_y: Align,
    pub background_color: Color,
    pub border_color: Color,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            padding: Insets::default(),
            margin: Insets::default(),
            width: SizingPolicy::Auto,
            height: SizingPolicy::Auto,
            align_x: Align::Start,
            align_y: Align::Start,
            justify_x: Align::Start,
            justify_y: Align::Start,
            background_color: Color::default(),
            border_color: Color::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum FontFamily {
    Name(String),
    Serif,
    SansSerif,
    Cursive,
    Fantasy,
    Monospace,
}

#[derive(Clone, Debug)]
pub enum FontWeight {
    Thin,
    ExtraLight,
    Light,
    Normal,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
    Black,
}

#[derive(Clone, Debug)]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique,
}

// text style is pretty different (specific to text rendering) so we should keep it seperate
// the items in this struct will be generic, and then integrate with glyphon in the wgpu integration (to allow support for other rendering frameworks in the future)
#[derive(Clone, Debug)]
pub struct TextStyle {
    pub font_size: f32,
    pub line_height: f32,
    pub font_family: FontFamily,
    pub font_weight: FontWeight,
    pub font_style: FontStyle,
    pub text_color: Color,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_size: 16.0,
            line_height: 20.0,
            font_family: FontFamily::SansSerif,
            font_weight: FontWeight::Normal,
            font_style: FontStyle::Normal,
            text_color: Color::default(),
        }
    }
}
