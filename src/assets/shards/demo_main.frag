// MainImage - Combines all buffers with audio-reactive color swapping
// Samples from Buffer A (red), B (blue), C (green), and D (vignette)
// Uses bass, mid, high to swap color channels

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    let uv = in.uv;

    // Sample all buffer textures
    let buffer_a = textureSample(buffer_a_texture, buffer_a_sampler, uv); // Red channel
    let buffer_b = textureSample(buffer_b_texture, buffer_b_sampler, uv); // Blue channel
    let buffer_c = textureSample(buffer_c_texture, buffer_c_sampler, uv); // Green channel
    let buffer_d = textureSample(buffer_d_texture, buffer_d_sampler, uv); // Vignette

    // Extract individual color channels
    let red = buffer_a.r;
    let blue = buffer_b.b;
    let green = buffer_c.g;

    // Get audio energy levels
    let bass = uniforms.audio_bass;
    let mid = uniforms.audio_mid;
    let high = uniforms.audio_high;

    // Audio-reactive color swapping
    // High bass: emphasize red
    // High mid: emphasize green
    // High high: emphasize blue

    var final_color: vec3<f32>;

    // Simple color mixing based on audio
    // When bass is high, boost red and reduce others
    let r = red * (1.0 + bass * 0.8) + green * (1.0 - bass) * 0.3;
    let g = green * (1.0 + mid * 0.8) + blue * (1.0 - mid) * 0.3;
    let b = blue * (1.0 + high * 0.8) + red * (1.0 - high) * 0.3;

    final_color = vec3<f32>(r, g, b);

    // Apply vignette from Buffer D
    let vignette = buffer_d.r; // Use red channel as vignette mask
    final_color = final_color * vignette;

    // Apply gamma from Buffer D alpha channel
    let gamma = buffer_d.a;
    final_color = pow(final_color, vec3<f32>(1.0 / gamma));

    // Normalize to prevent oversaturation
    final_color = clamp(final_color, vec3<f32>(0.0), vec3<f32>(1.0));

    return vec4<f32>(final_color, 1.0);
}
