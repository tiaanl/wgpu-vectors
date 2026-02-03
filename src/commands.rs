use crate::{fill::Fill, math::BoundingBox, renderer::gpu, stroke::Stroke};

pub trait CommandEncoder {
    const ID: u32;

    fn encode(&self, out: &mut Vec<f32>);
}

pub trait IsBoundingBox {
    fn bounding_box(&self) -> BoundingBox;
}

#[derive(Default)]
pub struct CommandList {
    pub draws: Vec<gpu::Draw>,
    pub op_codes: Vec<f32>,
}

impl CommandList {
    pub fn draw<S: CommandEncoder + IsBoundingBox>(
        &mut self,
        shape: S,
        fill: Fill,
        stroke: Stroke,
    ) {
        let bounding_box = shape.bounding_box();

        self.draws.push(gpu::Draw {
            left: bounding_box.min.x,
            top: bounding_box.min.y,
            right: bounding_box.max.x,
            bottom: bounding_box.max.y,
            op_code_index: self.op_codes.len() as u32,
        });

        self.op_codes.push(f32::from_bits(S::ID));
        shape.encode(&mut self.op_codes);
        fill.encode(&mut self.op_codes);
        stroke.encode(&mut self.op_codes);
    }
}
