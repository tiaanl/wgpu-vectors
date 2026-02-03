use glam::Vec2;

use crate::{
    commands::{CommandEncoder, IsBoundingBox},
    math::BoundingBox,
};

pub struct Rectangle {
    center: Vec2,
    extent: Vec2,
    corner_radius: f32,
}

impl Rectangle {
    pub fn new(center: Vec2, extent: Vec2) -> Self {
        Self {
            center,
            extent,
            corner_radius: 0.0,
        }
    }

    pub fn with_corner_radius(self, corner_radius: f32) -> Self {
        Self {
            corner_radius,
            ..self
        }
    }
}

impl CommandEncoder for Rectangle {
    const ID: u32 = 1;

    fn encode(&self, out: &mut Vec<f32>) {
        out.extend_from_slice(&[
            self.center.x,
            self.center.y,
            self.extent.x,
            self.extent.y,
            self.corner_radius,
        ]);
    }
}

impl IsBoundingBox for Rectangle {
    fn bounding_box(&self) -> BoundingBox {
        BoundingBox::new(self.center - self.extent, self.center + self.extent)
    }
}

pub struct Ellipse {
    center: Vec2,
    radius: Vec2,
}

impl Ellipse {
    pub fn new(center: Vec2, radius: Vec2) -> Self {
        Self { center, radius }
    }
}

impl CommandEncoder for Ellipse {
    const ID: u32 = 2;

    fn encode(&self, out: &mut Vec<f32>) {
        out.extend_from_slice(&[self.center.x, self.center.y, self.radius.x, self.radius.y]);
    }
}

impl IsBoundingBox for Ellipse {
    fn bounding_box(&self) -> BoundingBox {
        BoundingBox::new(self.center - self.radius, self.center + self.radius)
    }
}
