use crate::{
    element::{Element, ElementType},
    style::SizingPolicy,
    FrameInfo,
};

#[derive(Clone, Copy, Debug)]
struct Constraints {
    pub max_size: [u32; 2], // w, h
}

pub fn measure_pass(root: &mut Element, frame_info: &FrameInfo) {
    let frame_constraints = Constraints {
        max_size: frame_info.size,
    };
    measure_element_size(root, &frame_constraints);
}

// returns the required size of the given element based on its content and style
fn measure_element_size(element: &mut Element, constraints: &Constraints) -> [u32; 2] {
    match element._type.clone() {
        ElementType::Root => {
            for c in element.children.iter_mut() {
                measure_element_size(c, constraints); // we don't care about the output here since root elements keep the entire frame size
            }
            element.size = constraints.max_size; // just use the top-level constraints size for root (frame size)
            element.size
        }
        ElementType::Anchor(_) => measure_anchor_element_size(element, constraints),
        ElementType::Text(text) => measure_text_element_size(&text, element, constraints),
        ElementType::FlexRow => measure_flex_row_element_size(element, constraints),
        ElementType::FlexColumn => measure_flex_column_size(element, constraints),
    }
}

fn size_from_policy(sizing_policy: SizingPolicy, children_size: u32, parent_size: u32) -> u32 {
    match sizing_policy {
        SizingPolicy::Auto => children_size.min(parent_size), // if size of children is larger than the parent we should still go with the parent size
        SizingPolicy::FillParent => parent_size.min(children_size),
        SizingPolicy::Fixed(s) => s,
    }
}

fn measure_text_element_size(
    text: &String,
    element: &mut Element,
    constraints: &Constraints,
) -> [u32; 2] {
    let style = element.style;

    // TODO: these are placeholder values for now, should be dictated by style
    let char_w: u32 = 8;
    let line_h: u32 = 16;

    let text_width = (text.chars().count() as u32).saturating_mul(char_w);
    let text_height = line_h;

    let padded_width = text_width + style.padding.left + style.padding.right;
    let padded_height = text_height + style.padding.top + style.padding.bottom;

    let element_width = size_from_policy(style.width, padded_width, constraints.max_size[0]);
    let element_height = size_from_policy(style.height, padded_height, constraints.max_size[1]);

    element.size = [element_width, element_height]; // set the element size to use in the layout pass
    element.size
}

fn measure_anchor_element_size(element: &mut Element, constraints: &Constraints) -> [u32; 2] {
    let style = element.style;

    // for anchors their children are either constrained by their fixed size of their parents size
    let child_constraints_w = match style.width {
        SizingPolicy::Fixed(w) => w, // TODO: add padding here (from style)
        _ => constraints.max_size[0],
    };
    let child_constraints_h = match style.height {
        SizingPolicy::Fixed(h) => h, // TODO: add padding here (from style)
        _ => constraints.max_size[1],
    };
    let child_constraints = Constraints {
        max_size: [child_constraints_w, child_constraints_h],
    };

    // anchor element's total width is based on the max of their children's sizes (bounding box of max size)
    let mut max_child_width = 0;
    let mut max_child_height = 0;
    // measure child elements first to get their sizes
    for c in element.children.iter_mut() {
        let child_size = measure_element_size(c, &child_constraints);

        let child_margin_width = child_size[0]
            .saturating_add(c.style.margin.left)
            .saturating_add(c.style.margin.right);
        let child_margin_height = child_size[1]
            .saturating_add(c.style.margin.top)
            .saturating_add(c.style.margin.bottom);

        max_child_width = max_child_width.max(child_margin_width);
        max_child_height = max_child_height.max(child_margin_height);
    }

    let padded_width = max_child_width + style.padding.left + style.padding.right;
    let padded_height = max_child_height + style.padding.top + style.padding.bottom;

    let element_width = size_from_policy(style.width, padded_width, constraints.max_size[0]);
    let element_height = size_from_policy(style.height, padded_height, constraints.max_size[1]);

    element.size = [element_width, element_height];
    element.size
}

fn measure_flex_row_element_size(element: &mut Element, constraints: &Constraints) -> [u32; 2] {
    let style = element.style;
    let num_children = element.children.len();

    let padding_between_children: u32 = 0; // TODO: this is a placeholder for now, it should be set by style

    let max_width = constraints.max_size[0];
    let max_height = constraints.max_size[1];

    // all children of the flex row need to fit within the constraints of the row with its padding
    let child_constraints = Constraints {
        max_size: [
            max_width.saturating_sub(style.padding.left + style.padding.right),
            max_height.saturating_sub(style.padding.top + style.padding.bottom),
        ],
    };

    // for flex row we sum all children widths
    let mut total_child_width: u32 = 0;
    let mut max_child_height: u32 = 0; // we can just use the max height of the children

    for c in element.children.iter_mut() {
        let child_size = measure_element_size(c, &child_constraints);

        let child_margin_width = child_size[0]
            .saturating_add(c.style.margin.left)
            .saturating_add(c.style.margin.right);
        let child_margin_height = child_size[1]
            .saturating_add(c.style.margin.top)
            .saturating_add(c.style.margin.bottom);

        // add to total width, but just take max of height
        total_child_width = total_child_width.saturating_add(child_margin_width);
        max_child_height = max_child_height.max(child_margin_height);
    }

    // add padding between child elements if required
    if num_children > 1 && padding_between_children != 0 {
        let child_padding = padding_between_children * (num_children as u32 - 1);
        total_child_width = total_child_width.saturating_add(child_padding);
    }

    let padded_width = total_child_width + style.padding.left + style.padding.right;
    let padded_height = max_child_height + style.padding.top + style.padding.bottom;

    let element_width = size_from_policy(style.width, padded_width, max_width);
    let element_height = size_from_policy(style.height, padded_height, max_height);

    element.size = [element_width, element_height];
    element.size
}

fn measure_flex_column_size(element: &mut Element, constraints: &Constraints) -> [u32; 2] {
    let style = element.style;
    let num_children = element.children.len();

    let padding_between_children: u32 = 0; // TODO: add this to style

    let max_width = constraints.max_size[0];
    let max_height = constraints.max_size[1];

    let child_constraints = Constraints {
        max_size: [
            max_width.saturating_sub(style.padding.left + style.padding.right),
            max_height.saturating_sub(style.padding.top + style.padding.bottom),
        ],
    };

    // for flex column we sum all children heights
    let mut total_child_height: u32 = 0;
    let mut max_child_width: u32 = 0; // we can just use the max width of the children

    for c in element.children.iter_mut() {
        let child_size = measure_element_size(c, &child_constraints);

        let child_margin_width = child_size[0]
            .saturating_add(c.style.margin.left)
            .saturating_add(c.style.margin.right);
        let child_margin_height = child_size[1]
            .saturating_add(c.style.margin.top)
            .saturating_add(c.style.margin.bottom);

        // add to total height but just keep max width
        max_child_width = max_child_width.max(child_margin_width);
        total_child_height = total_child_height.saturating_add(child_margin_height);
    }

    // add padding between child elements if required
    if num_children > 1 && padding_between_children != 0 {
        let child_padding = padding_between_children * (num_children as u32 - 1);
        total_child_height = total_child_height.saturating_add(child_padding);
    }

    let padded_width = max_child_width + style.padding.left + style.padding.right;
    let padded_height = total_child_height + style.padding.top + style.padding.bottom;

    let element_width = size_from_policy(style.width, padded_width, max_width);
    let element_height = size_from_policy(style.height, padded_height, max_height);

    element.size = [element_width, element_height];
    element.size
}
