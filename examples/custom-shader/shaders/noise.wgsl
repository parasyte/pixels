// Vertex shader bindings

struct VertexOutput {
    @location(0) tex_coord: vec2<f32>,
    @builtin(position) position: vec4<f32>,
}

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coord = fma(position, vec2<f32>(0.5, -0.5), vec2<f32>(0.5, 0.5));
    out.position = vec4<f32>(position, 0.0, 1.0);
    return out;
}

// Fragment shader bindings

@group(0) @binding(0) var r_tex_color: texture_2d<f32>;
@group(0) @binding(1) var r_tex_sampler: sampler;
struct Locals {
    time: f32,
}
@group(0) @binding(2) var<uniform> r_locals: Locals;

const tau = 6.283185307179586476925286766559;
const bias = 0.2376; // Offset the circular time input so it is never 0

// Random functions based on https://thebookofshaders.com/10/
const random_scale = 43758.5453123;
const random_x = 12.9898;
const random_y = 78.233;

fn random(x: f32) -> f32 {
    return fract(sin(x) * random_scale);
}

fn random_vec2(st: vec2<f32>) -> f32 {
    return random(dot(st, vec2<f32>(random_x, random_y)));
}

@fragment
fn fs_main(@location(0) tex_coord: vec2<f32>) -> @location(0) vec4<f32> {
    let sampled_color = textureSample(r_tex_color, r_tex_sampler, tex_coord);
    let noise_color = vec3<f32>(random_vec2(tex_coord.xy * vec2<f32>(r_locals.time % tau + bias)));

    return vec4<f32>(sampled_color.rgb * noise_color, sampled_color.a);
}
