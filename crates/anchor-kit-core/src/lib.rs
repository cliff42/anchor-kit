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

use crate::measure::measure_pass;

pub struct FrameInfo {
    pub size: [u32; 2], // width, height
    pub time_ns: f32,   // TODO: should we have a struct for this? -> something like SystemTime?
}

// UIState stores the actual elements
pub struct UIState {
    root: Element,
}

// UI is used for building the element tree with closures
pub struct UI<'a> {
    current_element: &'a mut Element,
}

// user code will look something like this:
// let render_list = ui_state.generate_frame(frame_info, |ui| {
//      ui.anchor(AnchorPosition::TopLeft, [100, 100], |ui| {
//          ui.text("hello world!");
//      })
// })

impl UIState {
    pub fn new(size: [u32; 2]) -> Self {
        Self {
            root: Element::new(element::ElementType::Root, size),
        }
    }

    /// Returns a render list of primatives to send to the renderer backend integrations to draw the frame
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
    pub fn anchor<F>(&mut self, anchor_position: AnchorPosition, size: [u32; 2], f: F)
    where
        F: FnOnce(&mut UI),
    {
        let mut anchor_element = Element::new(element::ElementType::Anchor(anchor_position), size);
        f(&mut UI {
            current_element: &mut anchor_element,
        }); // handle all child elements of the anchor position
        self.current_element.children.push(anchor_element);
    }

    // TODO: add styling as param
    pub fn text(&mut self, text: String) {
        let text_element = Element::new(element::ElementType::Text(text), [0, 0]);
        self.current_element.children.push(text_element);
    }

    pub fn flex_row<F>(&mut self, f: F)
    where
        F: FnOnce(&mut UI),
    {
        let mut flex_row_element = Element::new(element::ElementType::FlexRow, [0, 0]); // TODO: should flex row size be [0, 0]? - size should probably be the size of the parent element
        f(&mut UI {
            current_element: &mut flex_row_element,
        });
        self.current_element.children.push(flex_row_element);
    }

    // TODO: flex col, grid, text, panel, image ...
}
