// Vertex shader bindings

struct VertexOutput {
    [[location(0)]] tex_coord: vec2<f32>;
    [[builtin(position)]] position: vec4<f32>;
};

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
    out.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    return out;
}

// Fragment shader bindings

[[group(0), binding(0)]] var r_tex_color: texture_2d<f32>;
[[group(0), binding(1)]] var r_tex_sampler: sampler;
[[block]] struct Locals {
    time: f32;
};
[[group(0), binding(2)]] var r_locals: Locals;

let tau: f32 = 6.283185307179586476925286766559;
let bias: f32 = 0.2376; // Offset the circular time input so it is never 0

// Random functions based on https://thebookofshaders.com/10/
let random_scale: f32 = 43758.5453123;
let random_x: f32 = 12.9898;
let random_y: f32 = 78.233;

fn random(x: f32) -> f32 {
    return fract(sin(x) * random_scale);
}

fn random_vec2(st: vec2<f32>) -> f32 {
    return random(dot(st, vec2<f32>(random_x, random_y)));
}

[[stage(fragment)]]
fn fs_main([[location(0)]] tex_coord: vec2<f32>) -> [[location(0)]] vec4<f32> {
    let sampled_color: vec4<f32> = textureSample(r_tex_color, r_tex_sampler, tex_coord);
    let noise_color: vec3<f32> = vec3<f32>(random_vec2(
        tex_coord.xy * vec2<f32>(r_locals.time % tau + bias)
    ));

    return vec4<f32>(sampled_color.rgb * noise_color, sampled_color.a);
}
