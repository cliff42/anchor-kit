use crate::{anchor::AnchorPosition, element::ElementType, Element, FrameInfo, RenderList};

pub fn layout_pass(root: &Element, frame_info: &FrameInfo, render_list: &mut RenderList) {
    for child_element in &root.children {
        handle_element_layout(child_element, frame_info, render_list);
    }
}

fn handle_element_layout(element: &Element, frame_info: &FrameInfo, render_list: &mut RenderList) {
    match element._type {
        ElementType::Root => {
            // TODO: should we panic here? or bubble up an error instead?
            for child_element in &element.children {
                handle_element_layout(child_element, frame_info, render_list);
            }
        }
        ElementType::Anchor(anchor_position) => {}
        ElementType::Panel => {} // TODO: implement
    }
}

fn hanlde_anchor_element(
    element: &Element,
    anchor_position: AnchorPosition,
    frame_info: &FrameInfo,
    render_list: &mut RenderList,
) {
    let [fw, fh] = frame_info.size;
    let [ew, eh] = element.size;

    let (element_x, element_y) = match anchor_position {
        AnchorPosition::TopLeft => (0.0, 0.0),
        AnchorPosition::TopCenter => ((fw - ew) as f64 / 2.0, 0.0),
        AnchorPosition::TopRight => ((fw - ew) as f64, 0.0),
        AnchorPosition::MiddleLeft => (0.0, (fh - eh) as f64 / 2.0),
        AnchorPosition::MiddleCenter => ((fw - ew) as f64 / 2.0, (fh - eh) as f64 / 2.0),
        AnchorPosition::MiddleRight => ((fw - ew) as f64, (fh - eh) as f64 / 2.0),
        AnchorPosition::BottomLeft => (0.0, (fh - eh) as f64),
        AnchorPosition::BottomCenter => ((fw - ew) as f64 / 2.0, (fh - eh) as f64),
        AnchorPosition::BottomRight => ((fw - ew) as f64, (fh - eh) as f64),
    };
}
