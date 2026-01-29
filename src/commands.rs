use crate::{fill::Fill, stroke::Stroke};

pub trait CommandEncoder {
    const ID: u32;

    fn encode(&self, out: &mut Vec<f32>);
}

#[derive(Default)]
pub struct CommandList {
    pub data: Vec<f32>,
}

impl CommandList {
    pub fn draw<S: CommandEncoder>(&mut self, shape: S, fill: Fill, stroke: Stroke) {
        self.data.push(f32::from_bits(S::ID));
        shape.encode(&mut self.data);
        fill.encode(&mut self.data);
        stroke.encode(&mut self.data);
    }
}
