use crate::{anchor::AnchorPosition, element::ElementType, Element, FrameInfo};

const FRAME_ORIGIN: [u32; 2] = [0, 0];

pub fn layout_pass(root: &mut Element, frame_info: &FrameInfo) {
    // TODO: add some way to check that the tree passed in is valid? and will fit in the window size before rendering
    // TODO: think about a way to avoid running this every frame unless required (maybe only on data change etc) -> (ie) a super simple retained mode)?

    for child_element in root.children.iter_mut() {
        // for all top-level elements the parent position is the frame origin, and parent size is just the entire frame's resolution
        handle_element_layout(child_element, FRAME_ORIGIN, frame_info.size);
    }
}

fn handle_element_layout(
    element: &mut Element,
    allocated_origin: [u32; 2],
    allocated_size: [u32; 2],
) {
    match &element._type {
        ElementType::Root => {
            // TODO: should we panic here? or bubble up an error instead? (should probably have a check to make sure the tree is valid by not having 2 roots)
            for child_element in element.children.iter_mut() {
                handle_element_layout(child_element, allocated_origin, allocated_size);
            }
        }
        ElementType::Anchor(anchor_position) => {
            handle_anchor_element(element, *anchor_position, allocated_origin, allocated_size);
        }
        ElementType::Text(_) => {
            handle_text_element(element, allocated_origin, allocated_size);
        }
        ElementType::FlexRow => handle_flex_row(element, allocated_origin, allocated_size), // TODO: implement
        ElementType::Panel => {} // TODO: implement
    }
}

fn handle_anchor_element(
    element: &mut Element,
    anchor_position: AnchorPosition,
    allocated_origin: [u32; 2],
    allocated_size: [u32; 2],
) {
    let [aw, ah] = allocated_size;
    let [ew, eh] = element.size;

    // produces the relative x,y that all children elements should be anchored to for rendering
    let (rel_x, rel_y) = match anchor_position {
        AnchorPosition::TopLeft => (0.0, 0.0),
        AnchorPosition::TopCenter => ((aw - ew) as f64 / 2.0, 0.0),
        AnchorPosition::TopRight => ((aw - ew) as f64, 0.0),
        AnchorPosition::MiddleLeft => (0.0, (ah - eh) as f64 / 2.0),
        AnchorPosition::MiddleCenter => ((aw - ew) as f64 / 2.0, (ah - eh) as f64 / 2.0),
        AnchorPosition::MiddleRight => ((aw - ew) as f64, (ah - eh) as f64 / 2.0),
        AnchorPosition::BottomLeft => (0.0, (ah - eh) as f64),
        AnchorPosition::BottomCenter => ((aw - ew) as f64 / 2.0, (ah - eh) as f64),
        AnchorPosition::BottomRight => ((aw - ew) as f64, (ah - eh) as f64),
    };

    element.frame_position = Some([
        allocated_origin[0] + rel_x as u32,
        allocated_origin[1] + rel_y as u32,
    ]);

    for c in element.children.iter_mut() {
        handle_element_layout(
            c,
            element.frame_position.unwrap_or(FRAME_ORIGIN),
            element.size,
        );
    }
}

fn handle_text_element(
    element: &mut Element,
    allocated_origin: [u32; 2],
    allocated_size: [u32; 2],
) {
    let text = match &element._type {
        ElementType::Text(s) => s.as_str(),
        _ => return,
    };

    // TODO: these are placeholder values for now, should be dictated by styling
    let char_w: u32 = 8;
    let line_h: u32 = 16;

    let text_width = (text.chars().count() as u32).saturating_mul(char_w);
    let text_height = line_h;

    let [aw, ah] = allocated_size;
    let text_box_w = text_width.min(aw);
    let text_box_h = text_height.min(ah);

    element.frame_position = Some(allocated_origin);
    element.size = [text_box_w, text_box_h];
}

fn handle_flex_row(element: &mut Element, allocated_origin: [u32; 2], allocated_size: [u32; 2]) {
    let num_children = element.children.len();

    let padding: u32 = 8; // TODO: this is a placeholder for now, it should be set by styling

    // for flex elements we want to split the allocated space into even sizes for each of the child elements
    let [aw, ah] = allocated_size;
    let total_padding = if num_children > 1 {
        (num_children - 1) as u32 * padding
    } else {
        0
    };
    let available_width_after_padding = aw.saturating_sub(total_padding as u32);
    let width_per_child = if num_children > 0 {
        available_width_after_padding / num_children as u32
    } else {
        aw
    };
    let height_per_child = ah; // flex row only handles flex in width direction

    let [ax, ay] = allocated_origin;

    // left to right rendering order is assumed for now, but should be configurable in the future
    for (i, c) in element.children.iter_mut().enumerate() {
        let curr_child_origin = [ax + (width_per_child + padding) * i as u32, ay];
        handle_element_layout(c, curr_child_origin, [width_per_child, height_per_child]);
    }
}
