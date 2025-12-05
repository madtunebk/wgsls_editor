// Audio-reactive tunnel effect

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    var uv = (in.uv * 2.0 - 1.0) * vec2<f32>(uniforms.resolution.x / uniforms.resolution.y, 1.0);
    
    let t = uniforms.time;
    let bass = uniforms.audio_bass;
    let mid = uniforms.audio_mid;
    let high = uniforms.audio_high;
    
    // Tunnel effect
    let r = length(uv);
    let a = atan2(uv.y, uv.x);
    
    // Audio reactive tunnel depth
    let depth = 1.0 / (r + 0.1) + t * (0.5 + bass);
    let twist = a * (3.0 + mid * 5.0) + depth * 2.0;
    
    // Create spiraling patterns
    var col = vec3<f32>(0.0);
    
    for (var i = 0.0; i < 5.0; i += 1.0) {
        let layer_depth = depth + i * 0.2;
        let layer_twist = twist + i * 0.5;
        
        // Grid pattern
        let grid_x = fract(layer_depth * 5.0 + sin(layer_twist) * 2.0);
        let grid_y = fract(layer_twist * 3.0 + cos(layer_depth * 10.0));
        
        // Color each layer differently
        let layer_col = vec3<f32>(
            0.5 + 0.5 * sin(i + t + bass * 3.14),
            0.5 + 0.5 * sin(i * 2.0 + t * 1.5 + mid * 3.14),
            0.5 + 0.5 * sin(i * 3.0 + t * 2.0 + high * 3.14)
        );
        
        // Grid lines
        let line = smoothstep(0.9, 0.95, max(grid_x, grid_y));
        col += layer_col * line * (1.0 - i * 0.15);
    }
    
    // Vignette
    col *= 1.0 - r * 0.5;
    
    // Audio reactive brightness
    col *= 0.8 + 0.2 * (bass + mid + high) / 3.0;
    
    return vec4<f32>(col, 1.0);
}
