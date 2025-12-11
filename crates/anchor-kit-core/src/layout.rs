use crate::{anchor::AnchorPosition, element::ElementType, style::Align, Element, FrameInfo};

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
    let style = element.style;
    let num_children = element.children.len();
    let padding_between_children: u32 = 0; // TODO: this is a placeholder for now, it should be set by styling

    let [ax, ay] = allocated_origin;
    element.frame_position = Some(allocated_origin);

    let mut content_x_start = ax + style.padding.left;
    let content_y_start = ay + style.padding.top;
    let total_content_width =
        element.size[0].saturating_sub(style.padding.left + style.padding.right);
    let total_content_height =
        element.size[1].saturating_sub(style.padding.top + style.padding.bottom);

    // for determining the justify style, we need to iterate over all children to acount for margins to distribute elements correctly
    let mut content_width_with_margin: u32 = 0;
    for c in element.children.iter() {
        content_width_with_margin = content_width_with_margin
            .saturating_add(c.style.margin.left)
            .saturating_add(c.size[0])
            .saturating_add(c.style.margin.right);
    }
    // need to add padding between content as well
    if num_children > 1 {
        let child_padding = padding_between_children * (num_children as u32 - 1);
        content_width_with_margin = content_width_with_margin.saturating_add(child_padding);
    }

    content_x_start = match style.justify_x {
        Align::Start => content_x_start,
        Align::Middle => {
            content_x_start + (total_content_width.saturating_sub(content_width_with_margin) / 2)
        }
        Align::End => {
            content_x_start + total_content_width.saturating_sub(content_width_with_margin)
        }
    };

    let mut x_offset = content_x_start; // current offset of where to place the next child

    // left to right rendering order is assumed for now, but should be configurable in the future
    for (i, c) in element.children.iter_mut().enumerate() {
        x_offset = x_offset.saturating_add(c.style.margin.left); // add margin of the child

        if i > 0 {
            x_offset = x_offset.saturating_add(padding_between_children);
        }

        let cy = match c.style.align_y {
            Align::Start => content_y_start + c.style.margin.top,
            Align::Middle => {
                content_y_start
                    + c.style.margin.top
                    + (total_content_height
                        .saturating_sub(c.style.margin.top + c.style.margin.bottom) // can only use space without the child elements margins
                        .saturating_sub(c.size[1])
                        / 2)
            }
            Align::End => {
                content_y_start
                    + total_content_height
                        .saturating_sub(c.size[1])
                        .saturating_sub(c.style.margin.bottom)
            }
        };

        let curr_child_origin = [x_offset, cy];
        handle_element_layout(c, curr_child_origin, c.size);

        x_offset = x_offset
            .saturating_add(c.size[0])
            .saturating_add(c.style.margin.right); // add the current child's width and its margin so the next child is offset correctly
    }
}

fn handle_flex_column(element: &mut Element, allocated_origin: [u32; 2]) {
    let style = element.style;
    let num_children = element.children.len();
    let padding_between_children: u32 = 0; // TODO: set by styling

    let [ax, ay] = allocated_origin;
    element.frame_position = Some(allocated_origin);

    let content_x_start = ax + style.padding.left;
    let mut content_y_start = ay + style.padding.top;
    let total_content_width =
        element.size[0].saturating_sub(style.padding.left + style.padding.right);
    let total_content_height =
        element.size[1].saturating_sub(style.padding.top + style.padding.bottom);

    let mut content_height_with_margin: u32 = 0;
    for c in element.children.iter() {
        content_height_with_margin = content_height_with_margin
            .saturating_add(c.style.margin.top)
            .saturating_add(c.size[1])
            .saturating_add(c.style.margin.bottom);
    }
    if num_children > 1 {
        let child_padding = padding_between_children * (num_children as u32 - 1);
        content_height_with_margin = content_height_with_margin.saturating_add(child_padding);
    }

    content_y_start = match style.justify_y {
        Align::Start => content_y_start,
        Align::Middle => {
            content_y_start + (total_content_height.saturating_sub(content_height_with_margin) / 2)
        }
        Align::End => {
            content_y_start + total_content_height.saturating_sub(content_height_with_margin)
        }
    };

    let mut y_offset = content_y_start; // vertical offset for placing children

    // top down rendering order is assumed for now, we can make this configurable in the future
    for (i, c) in element.children.iter_mut().enumerate() {
        y_offset = y_offset.saturating_add(c.style.margin.top);

        if i > 0 {
            y_offset = y_offset.saturating_add(padding_between_children);
        }

        let cx = match c.style.align_x {
            Align::Start => content_x_start + c.style.margin.left,
            Align::Middle => {
                content_x_start
                    + c.style.margin.left
                    + (total_content_width
                        .saturating_sub(c.style.margin.left + c.style.margin.right) // for middle case we need to use only half of the with after margins
                        .saturating_sub(c.size[0])
                        / 2)
            }
            Align::End => {
                content_x_start
                    + total_content_width
                        .saturating_sub(c.size[0])
                        .saturating_sub(c.style.margin.right)
            }
        };

        let curr_child_origin = [cx, y_offset];
        handle_element_layout(c, curr_child_origin, c.size);

        y_offset = y_offset
            .saturating_add(c.size[1])
            .saturating_add(c.style.margin.bottom);
    }
}
