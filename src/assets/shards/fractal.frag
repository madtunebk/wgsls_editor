// Audio-reactive Julia set fractal

// Complex number multiplication
fn complex_mul(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(a.x * b.x - a.y * b.y, a.x * b.y + a.y * b.x);
}

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    var uv = (in.uv * 2.0 - 1.0) * vec2<f32>(uniforms.resolution.x / uniforms.resolution.y, 1.0);
    
    let t = uniforms.time * 0.3;
    let bass = uniforms.audio_bass;
    let mid = uniforms.audio_mid;
    let high = uniforms.audio_high;
    
    // Zoom animation with audio
    let zoom = 0.5 + sin(t) * 0.3 - bass * 0.5;
    uv *= zoom;
    
    // Pan the view
    let center = vec2<f32>(-0.5 + sin(t * 0.3) * 0.3, cos(t * 0.2) * 0.3);
    
    // Audio reactive Julia set parameter
    let c = vec2<f32>(
        -0.4 + sin(t * 0.5) * 0.3 + mid * 0.2,
        0.6 + cos(t * 0.3) * 0.3 + high * 0.2
    );
    
    // Julia set iteration
    var z = uv + center;
    var iterations = 0.0;
    let max_iter = 100.0;
    
    for (var i = 0.0; i < max_iter; i += 1.0) {
        if length(z) > 2.0 {
            break;
        }
        z = complex_mul(z, z) + c;
        iterations += 1.0;
    }
    
    // Color based on iteration count
    let smooth_iter = iterations / max_iter;
    
    var col = vec3<f32>(0.0);
    if iterations < max_iter {
        // Create colorful patterns
        col = vec3<f32>(
            0.5 + 0.5 * sin(smooth_iter * 10.0 + t + bass * 3.14),
            0.5 + 0.5 * sin(smooth_iter * 10.0 + t * 1.3 + mid * 3.14),
            0.5 + 0.5 * sin(smooth_iter * 10.0 + t * 1.7 + high * 3.14)
        );
        
        // Add brightness variation
        col *= 0.5 + 0.5 * smooth_iter;
    } else {
        // Interior color (audio reactive)
        col = vec3<f32>(
            bass * 0.2,
            mid * 0.2,
            high * 0.5
        );
    }
    
    // Audio reactive brightness pulsing
    col *= 0.8 + 0.2 * sin(t * 2.0 + (bass + mid + high) * 3.14);
    
    return vec4<f32>(col, 1.0);
}
