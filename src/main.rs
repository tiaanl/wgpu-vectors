use std::sync::Arc;

use glam::{Vec2, Vec4};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

#[allow(clippy::large_enum_variant)]
enum App {
    Suspended,
    Resumed {
        window: Arc<Window>,
        device: wgpu::Device,
        queue: wgpu::Queue,
        surface: wgpu::Surface<'static>,
        surface_config: wgpu::SurfaceConfiguration,

        screen_pipeline: ScreenPipeline,
    },
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(WindowAttributes::default())
                .unwrap(),
        );

        let instance = wgpu::Instance::default();

        let PhysicalSize { width, height } = window.inner_size();

        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
                .unwrap();

        let surface = instance.create_surface(Arc::clone(&window)).unwrap();
        let surface_config = surface.get_default_config(&adapter, width, height).unwrap();

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).unwrap();

        surface.configure(&device, &surface_config);

        let screen_pipeline = ScreenPipeline::new(&device, surface_config.format);

        *self = Self::Resumed {
            window,
            surface,
            surface_config,
            device,
            queue,

            screen_pipeline,
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Self::Resumed {
            window,
            surface,
            device,
            queue,
            surface_config,
            screen_pipeline,
            ..
        } = self
        else {
            return;
        };

        if window_id != window.id() {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                let PhysicalSize { width, height } = window.inner_size();

                if width != surface_config.width || height != surface_config.height {
                    println!("Resizing to {width}x{height}");
                    surface_config.width = width;
                    surface_config.height = height;
                    surface.configure(device, surface_config);
                }

                let surface_texture = surface.get_current_texture().unwrap();
                let surface_view = surface_texture
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

                let mut shapes = Shapes::default();

                shapes.rectangle(
                    Vec4::new(0.5, 0.0, 0.0, 1.0),
                    Vec2::new(400.0, 300.0),
                    Vec2::new(100.0, 10.0),
                    10.0,
                    0.0,
                    0.0,
                );

                shapes.ring(
                    Vec4::new(0.0, 0.0, 0.0, 1.0),
                    Vec2::new(405.0, 305.0),
                    10.0,
                    20.0,
                    30.0,
                );

                shapes.ring(
                    Vec4::new(1.0, 0.0, 0.0, 1.0),
                    Vec2::new(400.0, 300.0),
                    10.0,
                    20.0,
                    1.0,
                );

                screen_pipeline.render(device, queue, &mut encoder, &surface_view, &shapes);

                queue.submit(std::iter::once(encoder.finish()));

                surface_texture.present();

                window.request_redraw();
            }
            _ => {}
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.run_app(&mut App::Suspended).unwrap();
}

struct ScreenPipeline {
    pipeline: wgpu::RenderPipeline,
    buffer_size: u32,

    shapes_buffer: wgpu::Buffer,
    shapes_bind_group_layout: wgpu::BindGroupLayout,
    shapes_bind_group: wgpu::BindGroup,
}

impl ScreenPipeline {
    const INITIAL_BUFFER_SIZE: u32 = 1024;

    fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        let shapes_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("shapes"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let (shapes_buffer, shapes_bind_group) = Self::create_shapes_buffer(
            device,
            &shapes_bind_group_layout,
            Self::INITIAL_BUFFER_SIZE,
        );

        let module = device.create_shader_module(wgpu::include_wgsl!("screen.wgsl"));

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("screen_pipeline_layout"),
            bind_group_layouts: &[&shapes_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("screen_pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: None,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: None,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
            cache: None,
        });

        Self {
            pipeline,
            buffer_size: Self::INITIAL_BUFFER_SIZE,

            shapes_buffer,
            shapes_bind_group_layout,
            shapes_bind_group,
        }
    }

    fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        surface_view: &wgpu::TextureView,
        shapes: &Shapes,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("screen_render_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: surface_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // Make sure the buffer is big enough.
        if self.buffer_size < shapes.data.len() as u32 {
            let (buffer, bind_group) = Self::create_shapes_buffer(
                device,
                &self.shapes_bind_group_layout,
                self.buffer_size.next_power_of_two(),
            );

            self.shapes_buffer = buffer;
            self.shapes_bind_group = bind_group;
        }

        // Upload the data to the buffer.
        queue.write_buffer(&self.shapes_buffer, 0, bytemuck::cast_slice(&shapes.data));

        // Setup the draw.
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.shapes_bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }

    fn create_shapes_buffer(
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        size: u32,
    ) -> (wgpu::Buffer, wgpu::BindGroup) {
        println!("Creating shapes buffer with size: {size}");

        let buffer = device.create_buffer(&wgpu::wgt::BufferDescriptor {
            label: Some("shapes"),
            size: size as wgpu::BufferAddress * std::mem::size_of::<f32>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shapes"),
            layout: bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        (buffer, bind_group)
    }
}

#[derive(Default)]
struct Shapes {
    data: Vec<f32>,
}

impl Shapes {
    const END_ID: u32 = 0;
    const CIRCLE_ID: u32 = 1;
    const RING_ID: u32 = 2;
    const RECTANGLE_ID: u32 = 3;

    fn circle(&mut self, color: Vec4, center: Vec2, radius: f32, feather: f32) {
        self.command(
            Self::CIRCLE_ID,
            &[
                color.x, color.y, color.z, color.w, center.x, center.y, radius, feather,
            ],
        );
    }

    fn ring(
        &mut self,
        color: Vec4,
        center: Vec2,
        radius_inner: f32,
        radius_outer: f32,
        feather: f32,
    ) {
        self.command(
            Self::RING_ID,
            &[
                color.x,
                color.y,
                color.z,
                color.w,
                center.x,
                center.y,
                radius_inner,
                radius_outer,
                feather,
            ],
        );
    }

    fn rectangle(
        &mut self,
        color: Vec4,
        center: Vec2,
        half_size: Vec2,
        radius: f32,
        angle: f32,
        feather: f32,
    ) {
        self.command(
            Self::RECTANGLE_ID,
            &[
                color.x,
                color.y,
                color.z,
                color.w,
                center.x,
                center.y,
                half_size.x,
                half_size.y,
                radius,
                angle,
                feather,
            ],
        );
    }

    fn command(&mut self, command: u32, data: &[f32]) {
        self.data.reserve(data.len() + 2);
        self.data.push(f32::from_bits(command));
        self.data.extend_from_slice(data);
    }
}
