struct VertexInput {
    @location(0) position: vec2<f32>, // normalized pos
    @location(1) local_uv: vec2<f32>, // normalized uv within the object
    @location(2) background_color: vec4<f32>, // r, g, b, a
    @location(3) border_radius_local: vec4<f32>, // top-left, top-right, bottom-right, bottom-left (clockwise) in local units (0-1)
    @location(4) border_width_local: f32, // in local units (0-1)
    @location(5) border_color: vec4<f32>, // r, g, b, a
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) local_uv: vec2<f32>,
    @location(1) background_color: vec4<f32>,
    @location(2) border_radius_local: vec4<f32>, // top-left, top-right, bottom-right, bottom-left (clockwise)
    @location(3) border_width_local: f32,
    @location(4) border_color: vec4<f32>, // r, g, b, a
};

@vertex
fn vs_main(
    model: VertexInput    
) -> VertexOutput {
    let ndc_x = model.position.x * 2.0 - 1.0;
    let ndc_y = 1.0 - model.position.y * 2.0; // flip y (to bottom up) for GPU coords

    var out: VertexOutput;
    out.position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0); // just set z to 0 since we only deal with 2d rendering, 1.0 for w
    out.local_uv = model.local_uv;
    out.background_color = model.background_color;
    out.border_radius_local = model.border_radius_local;
    out.border_width_local = model.border_width_local;
    out.border_color = model.border_color;
    return out;
}

struct SDFInput {
    position: vec2<f32>, // normalized
    half_size: vec2<f32>, // SDFs work with half sizes
    border_radius: vec4<f32>, // top-left, top-right, bottom-right, bottom-left (clockwise)s
}

// sdf (signed distance function) inspired by: https://iquilezles.org/articles/distfunctions2d/
fn sdf_rounded(in: SDFInput) -> f32 {
    var radius_to_choose: f32;
    // we need to pick which radius we want to calculate the sdf on based on the position of the pixel (which quadrent of the box we are in)
    if in.position[0] >= 0.0 {
        if in.position[1] >= 0.0 {
            // top-right
            radius_to_choose = in.border_radius[1];
        } else {
            // bottom-right
            radius_to_choose = in.border_radius[2];
        }
    } else {
        if in.position[1] >= 0.0 {
            // top-left
            radius_to_choose = in.border_radius[0];
        } else {
            // bottom-left
            radius_to_choose = in.border_radius[3];
        }
    }

    // we need to make a vec of radius to choose to the math works here
    let q = abs(in.position) - in.half_size + vec2<f32>(radius_to_choose, radius_to_choose); 
    return min(max(q[0], q[1]), 0.0) + length(max(q, vec2<f32>(0.0, 0.0))) - radius_to_choose;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // center at 0,0
    let position = in.local_uv - vec2<f32>(0.5, 0.5);
    let half_size = vec2<f32>(0.5, 0.5);

    var sdf_input: SDFInput;
    sdf_input.position = position;
    sdf_input.half_size = half_size;
    sdf_input.border_radius = in.border_radius_local;
    let d = sdf_rounded(sdf_input);

    // basic anti aliasing to get smooth corners
    let anti_aliasing = 1.0 / 50;
    let alpha_mul = 1.0 - smoothstep(0.0, anti_aliasing, d);

    // we need negative border width since with sdf d < 0.0 means inside the shape
    let border = (smoothstep(-in.border_width_local - anti_aliasing, -in.border_width_local + anti_aliasing, d) * (1.0 - smoothstep(0.0 - anti_aliasing, 0.0 + anti_aliasing, d))) * alpha_mul;
    let fill = (1.0 - smoothstep(-in.border_width_local - anti_aliasing, -in.border_width_local + anti_aliasing, d)) * alpha_mul;

    var output_color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    output_color += border * in.border_color;
    output_color += fill * in.background_color;
    return output_color;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_image(in: VertexOutput) -> @location(0) vec4<f32> {
    // center at 0,0
    let position = in.local_uv - vec2<f32>(0.5, 0.5);
    let half_size = vec2<f32>(0.5, 0.5);

    var sdf_input: SDFInput;
    sdf_input.position = position;
    sdf_input.half_size = half_size;
    sdf_input.border_radius = in.border_radius_local;
    let d = sdf_rounded(sdf_input);

    // basic anti aliasing to get smooth corners
    let anti_aliasing = 1.0 / 50;
    let alpha_mul = 1.0 - smoothstep(0.0, anti_aliasing, d);

    // we need negative border width since with sdf d < 0.0 means inside the shape
    let border = (smoothstep(-in.border_width_local - anti_aliasing, -in.border_width_local + anti_aliasing, d) * (1.0 - smoothstep(0.0 - anti_aliasing, 0.0 + anti_aliasing, d))) * alpha_mul;
    let fill = (1.0 - smoothstep(-in.border_width_local - anti_aliasing, -in.border_width_local + anti_aliasing, d)) * alpha_mul;

    var output_color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    output_color += border * in.border_color;
    output_color += fill * textureSample(t_diffuse, s_diffuse, in.local_uv); // use the texture from the bindings for the fill colour
    return output_color;
}
