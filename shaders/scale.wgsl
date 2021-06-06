// Vertex shader bindings

struct VertexOutput {
    [[location(0)]] tex_coord: vec2<f32>;
    [[builtin(position)]] position: vec4<f32>;
};

[[block]] struct Locals {
    transform: mat4x4<f32>;
};
[[group(0), binding(2)]] var r_locals: Locals;

let positions: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    // Upper left triangle
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(1.0, -1.0),
    vec2<f32>(-1.0, 1.0),

    // Lower right triangle
    vec2<f32>(-1.0, 1.0),
    vec2<f32>(1.0, -1.0),
    vec2<f32>(1.0, 1.0),
);

let uv: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    // Upper left triangle
    vec2<f32>(0.0, 0.0),
    vec2<f32>(1.0, 0.0),
    vec2<f32>(0.0, 1.0),

    // Lower right triangle
    vec2<f32>(0.0, 1.0),
    vec2<f32>(1.0, 0.0),
    vec2<f32>(1.0, 1.0),
);

[[stage(vertex)]]
fn vs_main([[builtin(vertex_index)]] vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coord = uv[vertex_index];
    out.position = r_locals.transform * vec4<f32>(positions[vertex_index], 0.0, 1.0);
    return out;
}

// Fragment shader bindings

[[group(0), binding(0)]] var r_tex_color: texture_2d<f32>;
[[group(0), binding(1)]] var r_tex_sampler: sampler;

[[stage(fragment)]]
fn fs_main([[location(0)]] tex_coord: vec2<f32>) -> [[location(0)]] vec4<f32> {
    return textureSample(r_tex_color, r_tex_sampler, tex_coord);
}
