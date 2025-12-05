// Buffer A - Red Channel (Bass-reactive)
// Simple animated pattern for the red channel

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let t = uniforms.time;
    let bass = uniforms.audio_bass;

    // Create animated pattern based on UV and time
    let pattern = sin(uv.x * 10.0 + t) * cos(uv.y * 10.0 + t);

    // Modulate with bass energy
    let intensity = 0.5 + 0.5 * pattern * (0.5 + bass * 0.5);

    // Return red channel only (R, 0, 0, 1)
    return vec4<f32>(intensity, 0.0, 0.0, 1.0);
}
