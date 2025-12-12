use crate::{
    element::{Element, ElementType},
    primitives::{rectangle::Rectangle, text::Text},
};

#[derive(Clone, Default, Debug)]
pub struct RenderList {
    pub rectangles: Vec<Rectangle>,
    pub text: Vec<Text>,
}

pub fn render_pass(root: &Element, render_list: &mut RenderList) {
    for c in root.children.iter() {
        handle_element_render(c, render_list)
    }
}

pub fn handle_element_render(element: &Element, render_list: &mut RenderList) {
    match &element._type {
        ElementType::Root => {
            // TODO: should eventually error here as well
            for c in element.children.iter() {
                handle_element_render(c, render_list);
            }
        }
        ElementType::Anchor(_) => {
            for c in element.children.iter() {
                handle_element_render(c, render_list);
            }
        }
        ElementType::Text(_) => {
            handle_text_element(element, render_list);
        }
        ElementType::FlexRow => {
            for c in element.children.iter() {
                handle_element_render(c, render_list);
            }
        }
        ElementType::FlexColumn => {
            for c in element.children.iter() {
                handle_element_render(c, render_list);
            }
        }
        ElementType::Pill => {
            handle_pill_element(element, render_list);
            for c in element.children.iter() {
                handle_element_render(c, render_list);
            }
        }
    }
}

fn handle_text_element(element: &Element, render_list: &mut RenderList) {
    // for each of these we skip rendering the text if the values are None
    let text = match &element._type {
        ElementType::Text(s) => s.to_string(),
        _ => return,
    };
    let position = match &element.frame_position {
        Some(pos) => *pos,
        None => return,
    };

    let text_prim = Text {
        text,
        position,
        size: element.size,
        text_style: element.text_style.clone().unwrap_or_default(),
    };
    render_list.text.push(text_prim);
}

fn handle_pill_element(element: &Element, render_list: &mut RenderList) {
    let position = match &element.frame_position {
        Some(pos) => *pos,
        None => return,
    };

    render_list.rectangles.push(Rectangle {
        position,
        size: element.size,
        style: element.style, // TODO: for pill we should probably default to rounded corners
    });
}
