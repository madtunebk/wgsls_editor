// Uniforms struct
struct Uniforms {
    time: f32,
    audio_bass: f32,
    audio_mid: f32,
    audio_high: f32,
    resolution: vec2<f32>,
    _pad0: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

// Complex number operations for Mandelbrot
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
    
    // Starting point
    var z = uv + center;
    
    // Iteration count
    var iterations = 0.0;
    let max_iter = 100.0;
    
    // Orbit trap for coloring
    var min_dist = 1000.0;
    
    for (var i = 0.0; i < max_iter; i += 1.0) {
        // Julia set formula: z = z^2 + c
        z = complex_mul(z, z) + c;
        
        // Track minimum distance to origin (orbit trap)
        min_dist = min(min_dist, length(z));
        
        // Escape condition
        if (length(z) > 2.0) {
            break;
        }
        iterations += 1.0;
    }
    
    // Smooth coloring
    let smooth_iter = iterations - log2(log2(length(z)));
    let escape_ratio = smooth_iter / max_iter;
    
    // Create psychedelic colors based on iteration count and orbit trap
    var col = vec3<f32>(0.0);
    
    if (iterations < max_iter - 1.0) {
        // Escaped - use smooth iteration and orbit trap for coloring
        let hue1 = escape_ratio * 3.0 + t * 0.5 + bass;
        let hue2 = min_dist * 2.0 + mid;
        let hue3 = (1.0 - escape_ratio) * 2.0 + high;
        
        col = vec3<f32>(
            0.5 + 0.5 * sin(hue1 * 6.28),
            0.5 + 0.5 * sin(hue2 * 6.28 + 2.09),
            0.5 + 0.5 * sin(hue3 * 6.28 + 4.19)
        );
        
        // Brightness based on how quickly it escaped
        let brightness = 0.5 + (1.0 - escape_ratio) * 0.5;
        col *= brightness;
        
        // Audio reactive glow on edges
        let edge_glow = smoothstep(0.9, 1.0, escape_ratio);
        col += edge_glow * vec3<f32>(bass * 2.0, mid * 2.0, high * 2.0);
        
    } else {
        // Inside the set - dark with subtle color from orbit trap
        col = vec3<f32>(min_dist * 0.1) * vec3<f32>(bass, mid, high);
    }
    
    // Vignette
    let vignette = 1.0 - length(uv) * 0.3;
    col *= vignette;
    
    // Overall brightness boost with audio
    col *= 0.8 + (bass + mid + high) * 0.2;
    
    return vec4<f32>(col, 1.0);
}
