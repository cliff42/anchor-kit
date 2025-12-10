use crate::{anchor::AnchorPosition, element::ElementType, Element, FrameInfo};

const FRAME_ORIGIN: [u32; 2] = [0, 0];

pub fn layout_pass(root: &mut Element, frame_info: &FrameInfo) {
    // TODO: add some way to check that the tree passed in is valid? and will fit in the window size before rendering
    // TODO: think about a way to avoid running this every frame unless required (maybe only on data change etc) -> (ie) a super simple retained mode)?

    for c in root.children.iter_mut() {
        // for all top-level elements the parent position is the frame origin, and parent size is just the entire frame's resolution
        handle_element_layout(c, FRAME_ORIGIN, frame_info.size);
    }
}

fn handle_element_layout(
    element: &mut Element,
    allocated_origin: [u32; 2],
    allocated_size: [u32; 2],
) {
    match element._type.clone() {
        ElementType::Root => {
            // TODO: should we panic here? or bubble up an error instead? (should probably have a check to make sure the tree is valid by not having 2 roots)
            for c in element.children.iter_mut() {
                handle_element_layout(c, allocated_origin, allocated_size);
            }
        }
        ElementType::Anchor(anchor_position) => {
            handle_anchor_element(element, anchor_position, allocated_origin, allocated_size);
        }
        ElementType::Text(_) => {
            handle_text_element(element, allocated_origin);
        }
        ElementType::FlexRow => handle_flex_row(element, allocated_origin),
        ElementType::FlexColumn => handle_flex_column(element, allocated_origin),
    }
}

fn handle_anchor_element(
    element: &mut Element,
    anchor_position: AnchorPosition,
    allocated_origin: [u32; 2],
    allocated_size: [u32; 2],
) {
    let [aw, ah] = allocated_size;
    let [ew, eh] = element.size; // set in measure pass

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
            element.size, // TODO: do we need to account for padding on the anchor position here?
        );
    }
}

fn handle_text_element(element: &mut Element, allocated_origin: [u32; 2]) {
    element.frame_position = Some(allocated_origin);
}

fn handle_flex_row(element: &mut Element, allocated_origin: [u32; 2]) {
    let padding_between_children: u32 = 0; // TODO: this is a placeholder for now, it should be set by styling

    let [ax, ay] = allocated_origin;
    element.frame_position = Some(allocated_origin);

    let mut x_offset = ax; // current offset of where to place the next child

    // left to right rendering order is assumed for now, but should be configurable in the future
    for (i, c) in element.children.iter_mut().enumerate() {
        x_offset = x_offset.saturating_add(c.style.margin.left); // add margin of the child

        if i > 0 {
            x_offset = x_offset.saturating_add(padding_between_children);
        }

        let curr_child_origin = [x_offset, ay];
        handle_element_layout(c, curr_child_origin, c.size);

        x_offset = x_offset
            .saturating_add(c.size[0])
            .saturating_add(c.style.margin.right); // add the current child's width and its margin so the next child is offset correctly
    }
}

fn handle_flex_column(element: &mut Element, allocated_origin: [u32; 2]) {
    let padding_between_children: u32 = 0; // TODO: set by styling

    let [ax, ay] = allocated_origin;
    element.frame_position = Some(allocated_origin);

    let mut y_offset = ay; // vertical offset for placing children

    // top down rendering order is assumed for now, we can make this configurable in the future
    for (i, c) in element.children.iter_mut().enumerate() {
        y_offset = y_offset.saturating_add(c.style.margin.top);

        if i > 0 {
            y_offset = y_offset.saturating_add(padding_between_children);
        }

        let curr_child_origin = [ax, y_offset];
        handle_element_layout(c, curr_child_origin, c.size);

        y_offset = y_offset
            .saturating_add(c.size[1])
            .saturating_add(c.style.margin.bottom);
    }
}
