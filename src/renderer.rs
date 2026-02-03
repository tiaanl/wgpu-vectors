use granite::wgpu;

use crate::commands::CommandList;

pub struct Renderer {
    pipeline: wgpu::RenderPipeline,
    buffer_size: u32,

    shapes_buffer: wgpu::Buffer,
    shapes_bind_group_layout: wgpu::BindGroupLayout,
    shapes_bind_group: wgpu::BindGroup,
}

impl Renderer {
    const INITIAL_BUFFER_SIZE: u32 = 1024;

    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
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
            ..Default::default()
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

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        surface_view: &wgpu::TextureView,
        commands: &CommandList,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("shapes"),
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
        if self.buffer_size < commands.data.len() as u32 {
            let (buffer, bind_group) = Self::create_shapes_buffer(
                device,
                &self.shapes_bind_group_layout,
                self.buffer_size.next_power_of_two(),
            );

            self.shapes_buffer = buffer;
            self.shapes_bind_group = bind_group;
        }

        // Upload the data to the buffer.
        queue.write_buffer(&self.shapes_buffer, 0, bytemuck::cast_slice(&commands.data));

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
