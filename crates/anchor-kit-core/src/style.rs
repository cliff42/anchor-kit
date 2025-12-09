#[derive(Debug, Clone, Copy)]
pub enum SizingPolicy {
    Auto,
    Fixed(u32), // individual policy for width and height so only need 1 u32 here
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
    pub align_x: Align,
    pub align_y: Align,
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
        }
    }
}
