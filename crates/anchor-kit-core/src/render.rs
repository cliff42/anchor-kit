use crate::{element::Element, primitives::rectangle::Rectangle};

#[derive(Clone, Default, Debug)]
pub struct RenderList {
    rectangles: Vec<Rectangle>,
}

pub fn render_pass(element: &Element, render_list: &mut RenderList) {}
