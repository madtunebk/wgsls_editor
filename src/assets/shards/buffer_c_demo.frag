// Buffer C - Green Channel (High-reactive)
// Simple grid pattern for the green channel

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let t = uniforms.time;
    let high = uniforms.audio_high;

    // Create diagonal stripe pattern
    let diag = uv.x + uv.y + t * 0.2;
    let stripes = sin(diag * 15.0);

    // Modulate with high frequencies
    let intensity = 0.5 + 0.5 * stripes * (0.5 + high * 0.5);

    // Return green channel only (0, G, 0, 1)
    return vec4<f32>(0.0, intensity, 0.0, 1.0);
}
