use crate::anchor::AnchorPosition;

#[derive(Clone, Copy, Debug)]
pub enum ElementType {
    Root,
    Anchor(AnchorPosition),
    Panel,
    // TODO: add things like flex col and flex row
}

#[derive(Clone, Debug)]
pub struct Element {
    pub _type: ElementType, // 'type' is a reserved word in rust
    pub size: [u32; 2],
    pub children: Vec<Element>,
}

impl Element {
    pub fn new_root(size: [u32; 2]) -> Self {
        Self {
            _type: ElementType::Root,
            size,
            children: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.children = Vec::new();
    }
}
