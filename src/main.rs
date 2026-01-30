use wgpu_vectors::prelude::*;

use glam::Vec2;
use granite::{glam, prelude::*, wgpu};

struct ExampleBuilder;

impl SceneBuilder for ExampleBuilder {
    type Target = Example;

    fn build(&self, renderer: &RenderContext, surface_config: &SurfaceConfig) -> Self::Target {
        Example {
            renderer: Renderer::new(&renderer.device, surface_config.format),
        }
    }
}

struct Example {
    renderer: Renderer,
}

impl Scene for Example {
    fn render(
        &mut self,
        renderer: &RenderContext,
        surface: &Surface,
    ) -> impl Iterator<Item = wgpu::CommandBuffer> {
        let mut encoder = renderer
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        let mut commands = CommandList::default();

        commands.draw(
            Rectangle::new(Vec2::new(105.0, 105.0), Vec2::new(40.0, 40.0)).with_corner_radius(10.0),
            Fill::solid(0.0, 0.0, 0.0, 0.5).with_feather(10.0),
            Stroke::none(),
        );

        commands.draw(
            Rectangle::new(Vec2::new(100.0, 100.0), Vec2::new(40.0, 40.0)).with_corner_radius(10.0),
            Fill::solid(1.0, 0.5, 1.0, 1.0),
            Stroke::solid(0.5, 0.25, 0.5, 1.0, 2.0),
        );

        commands.draw(
            Ellipse::new(Vec2::new(200.0, 200.0), Vec2::new(52.0, 52.0)),
            Fill::solid(0.0, 0.0, 0.0, 0.5).with_feather(10.0),
            Stroke::none(),
        );

        commands.draw(
            Ellipse::new(Vec2::new(200.0, 200.0), Vec2::new(50.0, 50.0)),
            Fill::solid(1.0, 0.0, 0.5, 1.0),
            Stroke::solid(0.5, 0.0, 0.25, 1.0, 2.0),
        );

        self.renderer.render(
            &renderer.device,
            &renderer.queue,
            &mut encoder,
            &surface.view,
            &commands,
        );

        std::iter::once(encoder.finish())
    }
}

fn main() {
    granite::run(ExampleBuilder).unwrap();
}
