use wgpu::*;

use std::mem::size_of;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub pos: [f32; 2],
}

impl Vertex {
    pub fn description() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<Vertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &vertex_attr_array![0 => Float32x2],
        }
    }
}

pub const DEFAULT_SQUARE_VERTICES: [Vertex; 4] = [
    Vertex { pos: [-0.05, 0.05] },
    Vertex {
        pos: [-0.05, -0.05],
    },
    Vertex { pos: [0.05, -0.05] },
    Vertex { pos: [0.05, 0.05] },
];

pub const DEFAULT_SQUARE_INDICES: [u16; 6] = [0, 1, 2, 3, 0, 2];

// Alignment for wgpu:
// Two colors taking up 16 bytes = 32
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SquareColors {
    pub color_off: [f32; 4],
    pub color_on: [f32; 4],
}

pub const SQCOLOR_OFF_OFFSET: BufferAddress = 0;
pub const SQCOLOR_ON_OFFSET: BufferAddress = 16;

// Alignment for wgpu:
// Translation 8 + scale 4 + corner radius 4 = 16
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SquareInfo {
    pub translation: [f32; 2],
    pub scale: f32,
    pub corner_radius: f32,
}

pub const SQINFO_TRANSLATION_OFFSET: BufferAddress = 0;
pub const SQINFO_SCALE_OFFSET: BufferAddress = 8;
pub const SQINFO_CORNER_RADIUS_OFFSET: BufferAddress = 12;

pub const DEFAULT_SQUARE_INFO: SquareInfo = SquareInfo {
    translation: [0.0, 0.0],
    scale: 0.85,
    corner_radius: 1.0,
};

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance {
    // TODO: Use u16s
    pub pos: [u32; 2], // [0, 0] is the first square at the top-left
}

impl Instance {
    pub fn description() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<Instance>() as BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &vertex_attr_array![1 => Uint32x2],
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GridZoom {
    pub z: f32,
}
