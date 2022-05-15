struct SquareInfo {
    [[location(0)]] color: vec4<f32>;
    [[location(1)]] translation: vec2<f32>;
    [[location(2)]] scale: f32;
    [[location(3)]] corner_radius: f32;
};

[[group(0), binding(0)]]
var<uniform> square_info: SquareInfo;

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

    position = position * square_info.scale;
    position = 
        position 
        + square_info.translation 
        + vec2<f32>(f32(instance_pos.x), f32(instance_pos.y)) * distance;

    out.clip_position = vec4<f32>(position, 0.0, 1.0);
    return out;
}

// Fragment shader main
[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return square_info.color;
}