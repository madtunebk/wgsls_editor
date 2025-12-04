@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    // Create pixel coordinates (0 to 1)
    let uv = in.uv;
    
    // Quantize coordinates for LED grid
    let bands = 30.0;
    let segs = 40.0;
    
    var p: vec2<f32>;
    p.x = floor(uv.x * bands) / bands;
    p.y = floor(uv.y * segs) / segs;
    
    // Flip vertically so bars grow UP from bottom (not down from top)
    let p_flipped = 1.0 - p.y;
    
    // Read frequency data based on horizontal position
    // Map the 3 frequency bands across the spectrum
    var fft: f32;
    let x_pos = p.x;
    
    if (x_pos < 0.33) {
        // Left third: bass
        fft = uniforms.audio_bass;
    } else if (x_pos < 0.66) {
        // Middle third: mid
        fft = uniforms.audio_mid;
    } else {
        // Right third: high
        fft = uniforms.audio_high;
    }
    
    // Scale FFT to reasonable range (FFT values are 0-1, need to map to 0-1 for display)
    // Add some variation across the band for visual interest
    let band_variation = sin(x_pos * 31.4159 + uniforms.time * 2.0) * 0.05 + 0.95;
    fft = clamp(fft * band_variation * 0.8, 0.0, 1.0);  // Scale down to prevent saturation
    
    // LED color: green at bottom to red at top (like classic spectrum analyzer)
    let color = mix(
        vec3<f32>(0.0, 2.0, 0.0),  // Green
        vec3<f32>(2.0, 0.0, 0.0),  // Red
        sqrt(uv.y)
    );
    
    // Mask for bar graph - LEDs below FFT level are bright, above are dim
    let mask = select(0.1, 1.0, p_flipped < fft);
    
    // LED shape - create rounded square LEDs
    let d = fract((uv - p) * vec2<f32>(bands, segs)) - 0.5;
    let led = smoothstep(0.5, 0.35, abs(d.x)) * 
              smoothstep(0.5, 0.35, abs(d.y));
    
    let ledColor = led * color * mask;
    
    // Debug removed - no need for top bars anymore
    return vec4<f32>(ledColor, 1.0);
}