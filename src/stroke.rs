use granite::glam::Vec4;

pub struct Stroke {
    color: Vec4,
    thickness: f32,
    feather: f32,
}

impl Stroke {
    pub fn new(color: Vec4, thickness: f32) -> Self {
        Self {
            color,
            thickness,
            feather: 1.0,
        }
    }

    #[inline]
    pub fn solid(r: f32, g: f32, b: f32, alpha: f32, thickness: f32) -> Self {
        Self::new(Vec4::new(r, g, b, alpha), thickness)
    }

    #[inline]
    pub fn none() -> Self {
        Self {
            color: Vec4::ZERO,
            thickness: 0.0,
            feather: 0.0,
        }
    }

    pub fn encode(&self, out: &mut Vec<f32>) {
        out.extend_from_slice(&[
            self.color.x,
            self.color.y,
            self.color.z,
            self.color.w,
            self.thickness,
            self.feather,
        ]);
    }
}
