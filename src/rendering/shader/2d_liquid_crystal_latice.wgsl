// Vertex shader

// struct Camera {
//     view_proj: mat4x4<f32>,
// }
// @group(1) @binding(0)
// var<uniform> camera: Camera;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) offset: vec2<f32>,
}
// struct InstanceInput {
//     @location(5) model_matrix_0: vec4<f32>,
//     @location(6) model_matrix_1: vec4<f32>,
//     @location(7) model_matrix_2: vec4<f32>,
//     @location(8) model_matrix_3: vec4<f32>,
// }

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(
    input: VertexInput,
    // instance: InstanceInput,
) -> VertexOutput {
    let identity_matrix = mat4x4<f32>(vec4<f32>(1,0,0,0), vec4<f32>(0,1,0,0), vec4<f32>(0,0,1,0), vec4<f32>(0,0,0,1));
    let instanceAngle: f32 = 0.0;
    let pos = vec2<f32>(input.position.xy);
    let offset = vec2<f32>(input.offset.xy);
    let translated_pos = pos - offset;
    let sin_a = sin(instanceAngle);
    let cos_a = cos(instanceAngle);
    let rotation_matrix = mat2x2<f32>(
        cos_a, -sin_a,
        sin_a, cos_a
    );

    // Rotate and offset the rectangle
    let rotated_pos = rotation_matrix * translated_pos + offset;
    var out: VertexOutput;
    out.tex_coords = input.tex_coords;
    out.clip_position = vec4<f32>(rotated_pos, 0.0, 1.0);
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0)@binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
