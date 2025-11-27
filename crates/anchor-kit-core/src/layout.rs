use crate::{Element, FrameInfo, RenderList, anchor::AnchorPosition, element::{self, ElementType}};

const FRAME_ORIGIN: [u32; 2] = [0, 0];

pub fn layout_pass(root: &Element, frame_info: &FrameInfo, render_list: &mut RenderList) {
    // TODO: add some way to check that the tree passed in is valid? and will fit in the window size before rendering
    // TODO: think about a way to avoid running this every frame unless required (maybe only on data change etc) -> (ie) a super simple retained mode)?

    for child_element in &root.children {
        // for all top-level elements the parent position is the frame origin, and parent size is just the entire frame's resolution
        handle_element_layout(child_element, FRAME_ORIGIN, frame_info.size, render_list);
    }
}

fn handle_element_layout(element: &Element, parent_frame_position: [u32; 2], parent_size: [u32; 2], render_list: &mut RenderList) {
    match &element._type {
        ElementType::Root => {
            // TODO: should we panic here? or bubble up an error instead? (should probably have a check to make sure the tree is valid by not having 2 roots)
            for child_element in &element.children {
                handle_element_layout(child_element, parent_frame_position, parent_size, render_list);
            }
        }
        ElementType::Anchor(anchor_position) => {}
        ElementType::Panel => {} // TODO: implement
        ElementType::Text(text) => {} // TODO: implement
    }
}

fn handle_anchor_element(
    element: &mut Element,
    anchor_position: AnchorPosition,
    parent_frame_position: [u32; 2],
    parent_size: [u32; 2],
    render_list: &mut RenderList,
) {
    let [pw, ph] = parent_size;
    let [ew, eh] = element.size;

    // produces the relative x,y that all children elements should be anchored to for rendering
    let (rel_x, rel_y) = match anchor_position {
        AnchorPosition::TopLeft => (0.0, 0.0),
        AnchorPosition::TopCenter => ((pw - ew) as f64 / 2.0, 0.0),
        AnchorPosition::TopRight => ((pw - ew) as f64, 0.0),
        AnchorPosition::MiddleLeft => (0.0, (ph - eh) as f64 / 2.0),
        AnchorPosition::MiddleCenter => ((pw - ew) as f64 / 2.0, (ph - eh) as f64 / 2.0),
        AnchorPosition::MiddleRight => ((pw - ew) as f64, (ph - eh) as f64 / 2.0),
        AnchorPosition::BottomLeft => (0.0, (ph - eh) as f64),
        AnchorPosition::BottomCenter => ((pw - ew) as f64 / 2.0, (ph - eh) as f64),
        AnchorPosition::BottomRight => ((pw - ew) as f64, (ph - eh) as f64),
    };

    element.frame_position = Some([
        parent_frame_position[0] + rel_x as u32,
        parent_frame_position[1] + rel_y as u32,
    ]);

    for c in element.children.iter_mut() {
        handle_element_layout(&c, element.frame_position.unwrap_or(FRAME_ORIGIN), element.size, render_list);
    }
}
