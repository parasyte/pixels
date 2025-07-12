// Vertex shader bindings

struct VertexOutput {
    @location(0) tex_coord: vec2<f32>,
    @builtin(position) position: vec4<f32>,
}

struct Locals {
    transform: mat4x4<f32>,
    input_size: vec4<f32>
}
@group(0) @binding(2) var<uniform> r_locals: Locals;

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    // Output tex coord in texel coordinates (0..width, 0..height)
    out.tex_coord = fma(position, vec2<f32>(0.5, -0.5), vec2<f32>(0.5, 0.5)) * r_locals.input_size.xy;
    out.position = r_locals.transform * vec4<f32>(position, 0.0, 1.0);
    return out;
}

// Fragment shader bindings

@group(0) @binding(0) var r_tex_color: texture_2d<f32>;
@group(0) @binding(1) var r_tex_sampler: sampler;

@fragment
fn fs_main(@location(0) tex_coord: vec2<f32>) -> @location(0) vec4<f32> {
    let half = vec2<f32>(0.5);
    let one = vec2<f32>(1.0);
    let zero = vec2<f32>(0.0);
    let texels_per_pixel = vec2<f32>(dpdx(tex_coord.x), dpdy(tex_coord.y));
    let tex_coord_fract = fract(tex_coord);
    let tex_coord_x = clamp(tex_coord_fract / texels_per_pixel, zero, half) + clamp((tex_coord_fract - one) / texels_per_pixel + half, zero, half);
    let tex_coord_final = (floor(tex_coord) + tex_coord_x) * r_locals.input_size.zw;
    return textureSample(r_tex_color, r_tex_sampler, tex_coord_final);
}
