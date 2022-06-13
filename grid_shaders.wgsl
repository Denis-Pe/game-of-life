struct SquareColors {
    [[location(0)]] color_off: vec4<f32>;
    [[location(1)]] color_on: vec4<f32>;
};

[[group(0), binding(0)]]
var<uniform> square_colors: SquareColors;

struct SquareInfo {
    [[location(0)]] translation: vec2<f32>;
    [[location(1)]] scale: f32;
    [[location(2)]] corner_radius: f32; // --TODO: USE THIS--
};

[[group(0), binding(1)]]
var<uniform> square_info: SquareInfo;

struct GridZoom {
    [[location(0)]] z: f32;
};

[[group(0), binding(2)]]
var<uniform> zoom: GridZoom;

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
};

// Vertex shader main
[[stage(vertex)]]
fn vs_main(
    [[location(0)]] vertex_pos: vec2<f32>,
    [[location(1)]] instance_pos: vec2<u32>
) -> VertexOutput {
    var out: VertexOutput;

    var position: vec2<f32> = vertex_pos;

    // the x and y distances between the squares
    let distance = vec2<f32>(abs(vertex_pos.x) * 2.0, abs(vertex_pos.y) * 2.0);

    let inst_pos_float = vec2<f32>(f32(instance_pos.x), f32(instance_pos.y)) * distance * zoom.z;

    position = position * square_info.scale * zoom.z;
    position = 
        position 
        + square_info.translation 
        + inst_pos_float;

    out.clip_position = vec4<f32>(position, 0.0, 1.0);
    return out;
}

// Fragment shader main
[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return square_colors.color_off;
}