use crate::anchor::AnchorPosition;

#[derive(Clone, Debug)]
pub enum ElementType {
    Root,
    Anchor(AnchorPosition),
    Panel,
    Text(String),
    // TODO: add things like flex col and flex row
}

#[derive(Clone, Debug)]
pub struct Element {
    pub(crate) _type: ElementType, // 'type' is a reserved word in rust
    pub(crate) size: [u32; 2],
    pub(crate) frame_position: Option<[u32; 2]>, // element positions are None until the layout pass
    pub(crate) children: Vec<Element>,
}

impl Element {
    pub fn new(element_type: ElementType, size: [u32; 2]) -> Self {
        Self {
            _type: element_type,
            size,
            frame_position: None,
            children: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.children = Vec::new();
    }
}
