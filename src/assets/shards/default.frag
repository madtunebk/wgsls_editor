// Default audio visualizer - no boilerplate needed!
// Uniforms and VSOut are auto-injected

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
    
    // Scale FFT to reasonable range
    let band_variation = sin(x_pos * 31.4159 + uniforms.time * 2.0) * 0.05 + 0.95;
    fft = clamp(fft * band_variation * 0.8, 0.0, 1.0);
    
    // Draw bars that grow from bottom
    var col: vec3<f32>;
    if (p_flipped < fft) {
        // Inside bar - create gradient
        let bar_height = p_flipped / fft;  // 0 at bottom, 1 at top
        
        // Color gradient: bass=red, mid=green, high=blue
        if (x_pos < 0.33) {
            col = vec3<f32>(1.0, bar_height * 0.5, bar_height * 0.2);  // Red to yellow
        } else if (x_pos < 0.66) {
            col = vec3<f32>(bar_height * 0.2, 1.0, bar_height * 0.5);  // Green to cyan
        } else {
            col = vec3<f32>(bar_height * 0.5, bar_height * 0.2, 1.0);  // Blue to magenta
        }
        
        // Brighten the top of each bar
        if (bar_height > 0.85) {
            col = col * 1.5;
        }
    } else {
        // Background
        col = vec3<f32>(0.05, 0.05, 0.1);
    }
    
    // Add grid lines
    let grid_thick = 0.02;
    if (fract(uv.x * bands) < grid_thick || fract(uv.y * segs) < grid_thick) {
        col = col * 0.5;  // Darken grid lines
    }
    
    // Subtle time-based pulse on entire scene
    let pulse = sin(uniforms.time * 2.0) * 0.05 + 1.0;
    col = col * pulse;
    
    return vec4<f32>(col, 1.0);
}
