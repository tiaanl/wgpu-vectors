use granite::glam::Vec2;

use crate::commands::CommandEncoder;

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

impl crate::commands::CommandEncoder for Rectangle {
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
