mod anchor;
mod element;
mod layout;
mod primitives;

use element::Element;
use layout::layout_pass;
use primitives::rectangle::Rectangle;

// TODO: origin (0,0) should be top-left

// TODO: this should be moved
pub struct FrameInfo {
    pub size: [u32; 2], // width, height
    pub time_ns: f32, // TODO: should we have a struct for this? -> something like SystemTime?
}

#[derive(Clone, Default, Debug)]
pub struct RenderList {
    rectangles: Vec<Rectangle>,
}

#[derive(Clone, Debug)]
pub struct UI {
    root: Element,
}

impl UI {
    pub fn new(size: [u32; 2]) -> Self {
        Self {
            root: Element::new_root(size),
        }
    }

    // TODO: frame info should be size of window, timestamp etc. (another struct)
    /// Returns a render list of primatives to send to the renderer backend integrations to draw the frame
    pub fn generate_frame<T: FnOnce(&mut UI)>(
        &mut self,
        frame_info: FrameInfo,
        populate_elements: T,
    ) -> RenderList {
        self.root.clear(); // clear the previous frame's element tree

        populate_elements(self); // use closure to populate the UIs elements based on the user's code -> this is the (|ui| { ... }) part -> this is the actual user code that builds out the tree

        let mut render_list = RenderList::default();
        layout_pass(&self.root, &frame_info, &mut render_list); // TODO: implement this, but handles all flex, layout stuff etc. based on the UIs element tree // TODO: this is where all of the flex based on data size and font size, margins etc would happen -> should probably live in another file
        render_list // TODO: output the render/ draw list to the actual winit or wgpu rendering code
    }

    // TODO: frame implementation?

    // pub fn anchor() -> {

    // }

    // pub fn flew_row() -> {

    // }

    // TODO: flex col, grid, text, panel, image ...
}
