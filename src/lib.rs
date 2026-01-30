//! wgpu-vectors

mod commands;
mod fill;
mod renderer;
mod shapes;
mod stroke;

pub mod prelude {
    pub use super::commands::CommandList;
    pub use super::fill::Fill;
    pub use super::renderer::Renderer;
    pub use super::shapes::*;
    pub use super::stroke::Stroke;
}
