struct VertexInput {
    @location(0) position: vec2<f32>; // normalized pos
    @location(1) color: vec4<f32>; // r, g, b, a
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>;
    @location(0) color: vec4<f32>;
};

@vertex
fn vs_main(
    model: VertexInput    
) -> VertexOutput {
    let ndc_x = model.position.x * 2.0 - 1.0;
    let ndc_y = 1.0 - model.position.y * 2.0; // flip y (to bottom up) for GPU coords

    var out: VertexOutput;
    out.position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0); // just set z to 0 since we only deal with 2d rendering, 1.0 for w
    out.color = model.color;
    return out;
}

// TODO: add texture sampling later for images etc.
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color; // return output color directly
}
