// IMPORTANT: This shader needs to be compiled out-of-band to SPIR-V
// See: https://github.com/parasyte/pixels/issues/9

#version 450

layout(location = 0) in vec2 v_TexCoord;
layout(location = 0) out vec4 outColor;
layout(set = 0, binding = 0) uniform texture2D t_Color;
layout(set = 0, binding = 1) uniform sampler s_Color;
layout(set = 0, binding = 2) uniform Locals {
    float u_Time;
};

#define PI 3.1415926535897932384626433832795
#define TAU PI * 2.0

// Offset the circular time input so it is never 0
#define BIAS 0.2376

// Random functions based on https://thebookofshaders.com/10/
#define RANDOM_SCALE 43758.5453123
#define RANDOM_X 12.9898
#define RANDOM_Y 78.233

float random(float x) {
    return fract(sin(x) * RANDOM_SCALE);
}

float random_vec2(vec2 st) {
    return random(dot(st.xy, vec2(RANDOM_X, RANDOM_Y)));
}

void main() {
    vec4 sampledColor = texture(sampler2D(t_Color, s_Color), v_TexCoord.xy);
    vec3 noiseColor = vec3(random_vec2(v_TexCoord.xy * vec2(mod(u_Time, TAU) + BIAS)));

    outColor = vec4(sampledColor.rgb * noiseColor, sampledColor.a);
}
