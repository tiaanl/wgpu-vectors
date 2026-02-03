//! wgpu-vectors

mod commands;
mod fill;
mod math;
mod renderer;
mod shapes;
mod stroke;
mod vec_buffer;

pub mod prelude {
    pub use super::commands::CommandList;
    pub use super::fill::Fill;
    pub use super::math::*;
    pub use super::renderer::{Renderer, View};
    pub use super::shapes::*;
    pub use super::stroke::Stroke;
}
