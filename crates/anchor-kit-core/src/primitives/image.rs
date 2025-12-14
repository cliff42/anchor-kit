use crate::primitives::rectangle::Rectangle;
use uuid::Uuid;

#[derive(Clone, Copy, Debug)]
pub struct Image {
    pub texture_id: Uuid,
    pub rectangle: Rectangle,
}

impl Image {
    pub fn new(texture_id: Uuid, rectangle: Rectangle) -> Self {
        Self {
            texture_id,
            rectangle,
        }
    }
}
