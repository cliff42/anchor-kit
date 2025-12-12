use crate::{
    anchor::AnchorPosition,
    style::{Style, TextStyle},
};

#[derive(Clone, Debug)]
pub enum ElementType {
    Root,
    Anchor(AnchorPosition),
    Text(String),
    FlexRow,
    FlexColumn,
    Pill,
    // TODO: add things like table, etc.
}

#[derive(Clone, Debug)]
pub struct Element {
    pub(crate) _type: ElementType, // 'type' is a reserved word in rust
    pub(crate) size: [u32; 2],
    pub(crate) style: Style,
    pub(crate) text_style: Option<TextStyle>,
    pub(crate) frame_position: Option<[u32; 2]>, // element positions are None until the layout pass
    pub(crate) children: Vec<Element>, // for now we will render all children first -> last = left -> right, but this could be configurable in future
}

impl Element {
    pub fn new(element_type: ElementType, style: Option<Style>) -> Self {
        Self {
            _type: element_type,
            size: [0, 0], // will be overwritten if using SizingPolicy::Auto in style
            style: style.unwrap_or_default(),
            text_style: None,
            frame_position: None,
            children: Vec::new(),
        }
    }

    pub fn new_text(text: String, style: Option<Style>, text_style: TextStyle) -> Self {
        Self {
            _type: ElementType::Text(text),
            size: [0, 0], // will be overwritten if using SizingPolicy::Auto in style
            style: style.unwrap_or_default(),
            text_style: Some(text_style),
            frame_position: None,
            children: Vec::new(),
        }
    }

    pub fn new_root(size: [u32; 2]) -> Self {
        Self {
            _type: ElementType::Root,
            size,
            style: Style::default(),
            text_style: None,
            frame_position: None,
            children: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.children = Vec::new();
    }
}
