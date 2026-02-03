use glam::Vec2;

pub struct BoundingBox {
    pub min: Vec2,
    pub max: Vec2,
}

impl BoundingBox {
    #[inline]
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }
}
