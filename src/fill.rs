use glam::Vec4;

pub struct Fill {
    pub color: Vec4,
    pub feather: f32,
}

impl Fill {
    pub fn new(color: Vec4) -> Self {
        Self {
            color,
            feather: 1.0,
        }
    }

    #[inline]
    pub fn solid(r: f32, g: f32, b: f32, alpha: f32) -> Self {
        Self::new(Vec4::new(r, g, b, alpha))
    }

    pub fn with_feather(self, feather: f32) -> Self {
        Self { feather, ..self }
    }

    pub fn encode(&self, out: &mut Vec<f32>) {
        out.extend_from_slice(&[
            self.color.x,
            self.color.y,
            self.color.z,
            self.color.w,
            self.feather,
        ]);
    }
}
