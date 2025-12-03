use crate::{anchor::AnchorPosition, style::Style};

#[derive(Clone, Debug)]
pub enum ElementType {
    Root,
    Anchor(AnchorPosition),
    Text(String),
    FlexRow,
    // TODO: add things like flex col, table, etc.
}

#[derive(Clone, Debug)]
pub struct Element {
    pub(crate) _type: ElementType, // 'type' is a reserved word in rust
    pub(crate) size: [u32; 2],
    pub(crate) style: Style,
    pub(crate) frame_position: Option<[u32; 2]>, // element positions are None until the layout pass
    pub(crate) children: Vec<Element>, // for now we will render all children first -> last = left -> right, but this could be configurable in future
}

impl Element {
    pub fn new(element_type: ElementType, size: [u32; 2]) -> Self {
        Self {
            _type: element_type,
            size,                    // will be overwritten if using SizingPolicy::Auto in style
            style: Style::default(), // TODO: placeholder for now -> we should pass this in eventually
            frame_position: None,
            children: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.children = Vec::new();
    }
}
