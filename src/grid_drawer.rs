use wgpu::{util::DeviceExt, *};

use gol::*;

use std::rc::Rc;

const DEFAULT_SQUARE_COLOR: &[f32] = &[1.0, 0.0, 1.0, 1.0];

#[derive(Debug)]
pub struct GridDrawer {
    surface: Rc<Surface>,
    device: Rc<Device>,
    queue: Rc<Queue>,
    render_pipeline: RenderPipeline,
    sqclr_buffer: Buffer,
    bind_group: BindGroup,
}

impl GridDrawer {
    pub fn new(state: &WgpuState) -> Self {
        let shader = state
            .device
            .create_shader_module(&include_wgsl!("../grid_shaders.wgsl"));

        let sqclr_buffer = state
            .device
            .create_buffer_init(&util::BufferInitDescriptor {
                label: Some("square_color_buffer"),
                contents: bytemuck::cast_slice(DEFAULT_SQUARE_COLOR),
                usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            });

        let bind_group_layout = state
            .device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::all(),
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let bind_group = state.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: sqclr_buffer.as_entire_binding(),
            }],
        });

        let render_pipeline_layout =
            state
                .device
                .create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: Some("grid_drawer_render_pipeline_layout"),
                    bind_group_layouts: &[&bind_group_layout],
                    push_constant_ranges: &[],
                });

        let render_pipeline = state
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("grid_drawer_render_pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[],
                },
                primitive: PrimitiveState {
                    topology: PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: FrontFace::Ccw,
                    cull_mode: Some(Face::Back),
                    unclipped_depth: false,
                    polygon_mode: PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[ColorTargetState {
                        format: state.config.format,
                        blend: Some(BlendState::REPLACE),
                        write_mask: ColorWrites::ALL,
                    }],
                }),
                multiview: None,
            });

        Self {
            surface: Rc::clone(&state.surface),
            device: Rc::clone(&state.device),
            queue: Rc::clone(&state.queue),
            render_pipeline,
            sqclr_buffer,
            bind_group,
        }
    }

    pub fn draw(&self, surface_texture: &SurfaceTexture) -> Result<(), SurfaceError> {
        let view = surface_texture
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("grid_drawer_render_encoder"),
            });

        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("grid_drawer_render_pass"),
            color_attachments: &[RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..3, 0..1);

        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }

    pub fn set_square_color(&self, color: Color) {
        self.queue.write_buffer(
            &self.sqclr_buffer,
            0,
            bytemuck::cast_slice(&color_to_arr(color)),
        );
    }
}
