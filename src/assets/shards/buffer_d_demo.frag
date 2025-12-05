// Buffer D - Alpha/Gamma Effect
// Creates a vignette and gamma correction layer

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    let uv = in.uv;

    // Center the UV coordinates
    let center = uv - vec2<f32>(0.5, 0.5);
    let dist = length(center);

    // Create vignette effect (darker at edges)
    let vignette = 1.0 - smoothstep(0.3, 0.8, dist);

    // Store vignette in all channels for easy multiplication
    // Alpha channel stores gamma value
    let gamma = 1.2; // Slight gamma boost

    return vec4<f32>(vignette, vignette, vignette, gamma);
}
