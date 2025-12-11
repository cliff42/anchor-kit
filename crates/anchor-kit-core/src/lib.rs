pub mod anchor;
pub mod element;
pub mod layout;
pub mod measure;
pub mod primitives;
pub mod render;
pub mod style;

use anchor::AnchorPosition;
use element::Element;
use layout::layout_pass;
use render::{render_pass, RenderList};

use crate::{
    measure::measure_pass,
    style::{Style, TextStyle},
};

pub struct FrameInfo {
    pub size: [u32; 2], // width, height
}

// UIState stores the actual elements
pub struct UIState {
    root: Element,
}

// UI is used for building the element tree with closures
pub struct UI<'a> {
    current_element: &'a mut Element,
}

// TODO: add more clear comments
impl UIState {
    pub fn new(size: [u32; 2]) -> Self {
        Self {
            root: Element::new_root(size),
        }
    }

    pub fn generate_frame<F>(&mut self, frame_info: FrameInfo, f: F) -> RenderList
    where
        F: FnOnce(&mut UI),
    {
        self.root.clear(); // clear the previous frame's element tree

        f(&mut UI {
            current_element: &mut self.root,
        });

        let mut render_list = RenderList::default();

        measure_pass(&mut self.root, &frame_info);
        layout_pass(&mut self.root, &frame_info);
        render_pass(&self.root, &mut render_list);

        render_list
    }
}

impl<'a> UI<'a> {
    pub fn anchor<F>(&mut self, anchor_position: AnchorPosition, style: Option<Style>, f: F)
    where
        F: FnOnce(&mut UI),
    {
        let mut anchor_element = Element::new(element::ElementType::Anchor(anchor_position), style);
        f(&mut UI {
            current_element: &mut anchor_element,
        }); // handle all child elements of the anchor position
        self.current_element.children.push(anchor_element);
    }

    pub fn text(&mut self, text: String, style: Option<Style>, text_style: Option<TextStyle>) {
        let text_element =
            Element::new_text(text, style, text_style.unwrap_or(TextStyle::default()));
        self.current_element.children.push(text_element);
    }

    pub fn flex_row<F>(&mut self, style: Option<Style>, f: F)
    where
        F: FnOnce(&mut UI),
    {
        let mut flex_row_element = Element::new(element::ElementType::FlexRow, style);
        f(&mut UI {
            current_element: &mut flex_row_element,
        });
        self.current_element.children.push(flex_row_element);
    }

    pub fn flex_column<F>(&mut self, style: Option<Style>, f: F)
    where
        F: FnOnce(&mut UI),
    {
        let mut flex_column_element = Element::new(element::ElementType::FlexColumn, style);
        f(&mut UI {
            current_element: &mut flex_column_element,
        });
        self.current_element.children.push(flex_column_element);
    }

    // TODO: grid, panel, image ...
}
