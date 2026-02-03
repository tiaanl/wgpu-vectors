use granite::wgpu;

use crate::{commands::CommandList, vec_buffer::VecBuffer};

pub struct Renderer {
    pipeline: wgpu::RenderPipeline,

    draws_buffer: VecBuffer<gpu::Draw>,
    op_codes_buffer: VecBuffer<f32>,

    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl Renderer {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("shapes"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let draws_buffer =
            VecBuffer::with_capacity(device, 1024, "draws", wgpu::BufferUsages::STORAGE);
        let op_codes_buffer =
            VecBuffer::with_capacity(device, 1024, "op_codes", wgpu::BufferUsages::STORAGE);

        let bind_group = Self::create_bind_group(
            device,
            &bind_group_layout,
            &draws_buffer.buffer,
            &op_codes_buffer.buffer,
        );

        let module = device.create_shader_module(wgpu::include_wgsl!("screen.wgsl"));

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("screen_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
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

            draws_buffer,
            op_codes_buffer,

            bind_group_layout,
            bind_group,
        }
    }

    fn create_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        draws_buffer: &wgpu::Buffer,
        op_codes_buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shapes_bind_group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: draws_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: op_codes_buffer.as_entire_binding(),
                },
            ],
        })
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        surface_view: &wgpu::TextureView,
        commands: &mut CommandList,
    ) {
        // Update the buffers.
        let changed = self.draws_buffer.write(device, queue, &commands.draws)
            || self
                .op_codes_buffer
                .write(device, queue, &commands.op_codes);

        // Update the bind group if any of the buffers changed.
        if changed {
            self.bind_group = Self::create_bind_group(
                device,
                &self.bind_group_layout,
                &self.draws_buffer.buffer,
                &self.op_codes_buffer.buffer,
            )
        }

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

        // Setup the draw.
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}

pub mod gpu {
    use bytemuck::NoUninit;

    #[derive(Clone, Copy, NoUninit)]
    #[repr(C)]
    pub struct Draw {
        pub left: f32,
        pub top: f32,
        pub right: f32,
        pub bottom: f32,
        pub op_code_index: u32,
    }
}
