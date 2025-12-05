// Buffer B - Blue Channel (Mid-reactive)
// Simple circular pattern for the blue channel

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let t = uniforms.time;
    let mid = uniforms.audio_mid;

    // Center the UV coordinates
    let center = uv - vec2<f32>(0.5, 0.5);
    let dist = length(center);

    // Create circular wave pattern
    let wave = sin(dist * 20.0 - t * 2.0);

    // Modulate with mid frequencies
    let intensity = 0.5 + 0.5 * wave * (0.5 + mid * 0.5);

    // Return blue channel only (0, 0, B, 1)
    return vec4<f32>(0.0, 0.0, intensity, 1.0);
}
