use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    *,
};

use gol::*;

mod buffers;
use buffers::*;

use std::rc::Rc;

use crate::settings::Settings;

const DEFAULT_GRID_SIZE: u16 = 5; // 5 x 5

#[derive(Debug)]
pub struct GridDrawer {
    // The wgpu stuff
    device: Rc<Device>,
    queue: Rc<Queue>,
    render_pipeline: RenderPipeline,
    bind_group: BindGroup,
    // Crucial buffers
    sqvert_buf: Buffer,
    sqind_buf: Buffer,
    // Vector of positions (x, y)
    // This also keeps track of the length of the grid implicitly
    instances: Vec<buffers::Instance>,
    instance_buf: Buffer,
    // Uniform buffers
    sqcolors_buf: Buffer,
    sqinfo: SquareInfo,
    sqinfo_buf: Buffer,
    grid_zoom: GridZoom,
    grid_zoom_buf: Buffer,
}

impl GridDrawer {
    pub fn new(wgpu_state: &WgpuState, settings: &Settings) -> Self {
        // --SHADER AND THE UNIFORM BUFFERS-- \\

        let shader = wgpu_state
            .device
            .create_shader_module(&include_wgsl!("../../grid_shaders.wgsl"));

        let sqcolor_off = settings.sqcolor_off().to_f32();
        let sqcolor_on = settings.sqcolor_on().to_f32();

        let sqcolors = SquareColors {
            color_off: [
                sqcolor_off[0],
                sqcolor_off[1],
                sqcolor_off[2],
                sqcolor_off[3],
            ],
            color_on: [sqcolor_on[0], sqcolor_on[1], sqcolor_on[2], sqcolor_on[3]],
        };

        let sqcolors_buf = wgpu_state.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("square_colors_buffer"),
            contents: bytemuck::cast_slice(&[sqcolors]),
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
        });

        // --TODO: ADJUST TO FIT TO SCREEN OR CENTER WITH ZOOM
        let sqinfo = DEFAULT_SQUARE_INFO;

        let sqinfo_buf = wgpu_state.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("square_information_buffer"),
            contents: bytemuck::cast_slice(&[sqinfo]),
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
        });

        // --SAME TODO HERE--
        let grid_zoom = GridZoom { z: 1.0 };

        let grid_zoom_buf = wgpu_state.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("grid_zoom_buffer"),
            contents: bytemuck::cast_slice(&[grid_zoom]),
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
        });

        // --BIND GROUP AND RENDER PIPELINE-- \\

        let bind_group_layout =
            wgpu_state
                .device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        BindGroupLayoutEntry {
                            binding: 0,
                            visibility: ShaderStages::FRAGMENT,
                            ty: BindingType::Buffer {
                                ty: BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            // sqinfo
                            binding: 1,
                            visibility: ShaderStages::all(),
                            ty: BindingType::Buffer {
                                ty: BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            // grid_zoom
                            binding: 2,
                            visibility: ShaderStages::VERTEX,
                            ty: BindingType::Buffer {
                                ty: BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });

        let bind_group = wgpu_state.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: sqcolors_buf.as_entire_binding(),
                },
                BindGroupEntry {
                    // sqinfo
                    binding: 1,
                    resource: sqinfo_buf.as_entire_binding(),
                },
                BindGroupEntry {
                    // grid_zoom
                    binding: 2,
                    resource: grid_zoom_buf.as_entire_binding(),
                },
            ],
        });

        let render_pipeline_layout =
            wgpu_state
                .device
                .create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: Some("grid_drawer_render_pipeline_layout"),
                    bind_group_layouts: &[&bind_group_layout],
                    push_constant_ranges: &[],
                });

        let render_pipeline = wgpu_state
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("grid_drawer_render_pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex::description(), buffers::Instance::description()],
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
                        format: wgpu_state.config.format,
                        blend: Some(BlendState::ALPHA_BLENDING),
                        write_mask: ColorWrites::ALL,
                    }],
                }),
                multiview: None,
            });

        // --OTHER BUFFERS THAT I COULD CREATE NOW-- \\

        let sqvert_buf = wgpu_state.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("square_vertex_buffer"),
            contents: bytemuck::cast_slice(&DEFAULT_SQUARE_VERTICES),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        let sqind_buf = wgpu_state.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("square_index_buffer"),
            contents: bytemuck::cast_slice(&DEFAULT_SQUARE_INDICES),
            usage: BufferUsages::INDEX,
        });

        let mut instances = Vec::with_capacity((DEFAULT_GRID_SIZE * DEFAULT_GRID_SIZE) as usize);

        for row in 0..DEFAULT_GRID_SIZE {
            for column in 0..DEFAULT_GRID_SIZE {
                instances.push(buffers::Instance {
                    pos: [column as u32, row as u32],
                });
            }
        }

        let instance_buf = wgpu_state.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("grid_drawer_instance_buffer"),
            contents: bytemuck::cast_slice(&instances),
            usage: BufferUsages::VERTEX,
        });

        Self {
            device: Rc::clone(&wgpu_state.device),
            queue: Rc::clone(&wgpu_state.queue),
            render_pipeline,
            bind_group,
            sqvert_buf,
            sqind_buf,
            sqinfo_buf,
            sqinfo,
            instances,
            instance_buf,
            grid_zoom,
            grid_zoom_buf,
            sqcolors_buf,
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
        render_pass.set_vertex_buffer(0, self.sqvert_buf.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buf.slice(..));
        render_pass.set_index_buffer(self.sqind_buf.slice(..), IndexFormat::Uint16);
        render_pass.draw_indexed(0..6, 0, 0..self.instances.len() as u32);

        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }

    pub fn set_square_color_off(&mut self, color: [f32; 4]) {
        self.queue.write_buffer(
            &self.sqcolors_buf,
            SQCOLOR_OFF_OFFSET,
            bytemuck::cast_slice(&color),
        );
    }

    pub fn set_square_color_on(&mut self, color: [f32; 4]) {
        self.queue.write_buffer(
            &self.sqcolors_buf,
            SQCOLOR_ON_OFFSET,
            bytemuck::cast_slice(&color),
        );
    }

    pub fn set_grid_translation(&mut self, translation: [f32; 2]) {
        self.sqinfo.translation[0] = translation[0];
        self.sqinfo.translation[1] = translation[1];
        self.queue.write_buffer(
            &self.sqinfo_buf,
            SQINFO_TRANSLATION_OFFSET,
            bytemuck::cast_slice(&self.sqinfo.translation),
        );
    }

    /// Will perform addition of the `translation` to the current translation
    pub fn translate_grid(&mut self, translation: [f32; 2]) {
        self.sqinfo.translation[0] += translation[0];
        self.sqinfo.translation[1] += translation[1];
        self.queue.write_buffer(
            &self.sqinfo_buf,
            SQINFO_TRANSLATION_OFFSET,
            bytemuck::cast_slice(&self.sqinfo.translation),
        );
    }

    pub fn set_square_scale(&mut self, scale: f32) {
        self.queue.write_buffer(
            &self.sqinfo_buf,
            SQINFO_SCALE_OFFSET,
            bytemuck::cast_slice(&[scale]),
        );
        self.sqinfo.scale = scale;
    }

    pub fn set_square_corner_radius(&mut self, corner_radius: f32) {
        self.queue.write_buffer(
            &self.sqinfo_buf,
            SQINFO_CORNER_RADIUS_OFFSET,
            bytemuck::cast_slice(&[corner_radius]),
        );
        self.sqinfo.corner_radius = corner_radius;
    }

    pub fn set_grid_zoom(&mut self, grid_zoom: f32) {
        self.queue
            .write_buffer(&self.grid_zoom_buf, 0, bytemuck::cast_slice(&[grid_zoom]));
        self.grid_zoom.z = grid_zoom;
    }

    pub fn change_grid_zoom(&mut self, change: f32) {
        self.grid_zoom.z += change;
        self.queue.write_buffer(
            &self.grid_zoom_buf,
            0,
            bytemuck::cast_slice(&[self.grid_zoom.z]),
        );
    }

    pub fn grid_zoom(&self) -> f32 {
        println!("{}", self.grid_zoom.z);
        self.grid_zoom.z
    }

    fn write_sqbuffer_vertices(&self, offsets: &[BufferAddress], value: f32) {
        for offset in offsets.iter() {
            self.queue
                .write_buffer(&self.sqvert_buf, *offset, bytemuck::cast_slice(&[value]));
        }
    }

    pub fn resize_window(&self, new_size: winit::dpi::PhysicalSize<u32>) {
        // The offsets to reach the x components of the vertices that are to the left
        const BUF_LEFT_X_OFFSETS: [BufferAddress; 2] = [0, 8];
        // Same but right
        const BUF_RIGHT_X_OFFSETS: [BufferAddress; 2] = [16, 24];

        // Same but the y components
        const BUF_UPPER_Y_OFFSETS: [BufferAddress; 2] = [4, 28];
        const BUF_LOWER_Y_OFFSETS: [BufferAddress; 2] = [12, 20];

        const DEFAULT_LEFT: f32 = DEFAULT_SQUARE_VERTICES[0].pos[0];
        const DEFAULT_RIGHT: f32 = DEFAULT_SQUARE_VERTICES[2].pos[0];
        const DEFAULT_UPPER: f32 = DEFAULT_SQUARE_VERTICES[0].pos[1];
        const DEFAULT_LOWER: f32 = DEFAULT_SQUARE_VERTICES[1].pos[1];

        let aspect_ratio = new_size.width as f32 / new_size.height as f32;

        if aspect_ratio > 1.0 {
            self.write_sqbuffer_vertices(&BUF_LEFT_X_OFFSETS, DEFAULT_LEFT / aspect_ratio);
            self.write_sqbuffer_vertices(&BUF_RIGHT_X_OFFSETS, DEFAULT_RIGHT / aspect_ratio);
        } else {
            self.write_sqbuffer_vertices(&BUF_UPPER_Y_OFFSETS, DEFAULT_UPPER * aspect_ratio);
            self.write_sqbuffer_vertices(&BUF_LOWER_Y_OFFSETS, DEFAULT_LOWER * aspect_ratio);
        }
    }

    pub fn resize_grid(&mut self, x: u32, y: u32) {
        self.instances.clear();

        for row in 0..y {
            for column in 0..x {
                self.instances
                    .push(buffers::Instance { pos: [column, row] });
            }
        }

        // recreate instance buffer
        self.instance_buf = self.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("grid_drawer_instance_buffer"),
            contents: bytemuck::cast_slice(&self.instances),
            usage: BufferUsages::VERTEX,
        });
    }
}
