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

// Hash function for procedural noise
fn hash(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.xyx) * 0.13);
    p3 += dot(p3, p3.yzx + 3.333);
    return fract((p3.x + p3.y) * p3.z);
}

// Smooth noise
fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(
        mix(hash(i + vec2<f32>(0.0, 0.0)), hash(i + vec2<f32>(1.0, 0.0)), u.x),
        mix(hash(i + vec2<f32>(0.0, 1.0)), hash(i + vec2<f32>(1.0, 1.0)), u.x),
        u.y
    );
}

// Fractal Brownian Motion
fn fbm(p: vec2<f32>) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 1.0;
    var p_var = p;
    
    for (var i = 0; i < 6; i++) {
        value += amplitude * noise(p_var * frequency);
        frequency *= 2.0;
        amplitude *= 0.5;
    }
    return value;
}

// Palette function for crazy colors
fn palette(t: f32) -> vec3<f32> {
    let a = vec3<f32>(0.5, 0.5, 0.5);
    let b = vec3<f32>(0.5, 0.5, 0.5);
    let c = vec3<f32>(1.0, 1.0, 1.0);
    let d = vec3<f32>(0.263, 0.416, 0.557);
    return a + b * cos(6.28318 * (c * t + d));
}

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    // Normalized pixel coordinates (from 0 to 1)
    var uv = in.uv * 2.0 - 1.0;
    uv.x *= uniforms.resolution.x / uniforms.resolution.y;
    
    // Audio reactive scaling and rotation
    let bass_pulse = uniforms.audio_bass * 2.0;
    let mid_pulse = uniforms.audio_mid * 1.5;
    let high_pulse = uniforms.audio_high;
    
    // Time with audio modulation
    let t = uniforms.time * 0.5 + bass_pulse * 2.0;
    
    // Rotate UV based on audio
    let angle = t + mid_pulse * 3.14159;
    let s = sin(angle);
    let c = cos(angle);
    let rot = mat2x2<f32>(c, -s, s, c);
    uv = rot * uv;
    
    // Zoom with bass
    uv *= 1.0 + bass_pulse * 0.5;
    
    // Initialize color
    var col = vec3<f32>(0.0);
    
    // Create multiple layers of animated patterns
    for (var i = 0.0; i < 3.0; i += 1.0) {
        // Spiral distortion
        let r = length(uv);
        let a = atan2(uv.y, uv.x);
        
        // Audio reactive spiral
        let spiral = a + r * (3.0 + bass_pulse * 2.0) - t * (1.0 + i * 0.3);
        
        // Fractal noise coordinates
        var p = vec2<f32>(
            cos(spiral) * r + t * 0.2,
            sin(spiral) * r - t * 0.15
        );
        
        // Add high frequency jitter
        p += vec2<f32>(sin(t * 2.0 + i), cos(t * 1.5 + i)) * high_pulse * 0.2;
        
        // Layered FBM with audio modulation
        let scale = 3.0 + i * 2.0;
        let n1 = fbm(p * scale + t * 0.3);
        let n2 = fbm(p * scale * 1.5 - t * 0.2 + vec2<f32>(n1));
        
        // Combine noise layers
        let pattern = n1 * n2;
        
        // Audio reactive color shifting
        let color_shift = (i + t * 0.5 + mid_pulse * 2.0 + pattern * 2.0);
        let layer_col = palette(color_shift);
        
        // Blend layers with varying intensity
        let intensity = (1.0 - i / 3.0) * (0.5 + bass_pulse * 0.5);
        col += layer_col * pattern * intensity;
    }
    
    // Add radial gradient with audio
    let vignette = 1.0 - length(uv * 0.5);
    col *= 0.5 + vignette * (0.5 + mid_pulse * 0.3);
    
    // Pulsing overlay based on all frequencies
    let pulse = (bass_pulse + mid_pulse + high_pulse) / 3.0;
    let flash = sin(t * 10.0) * 0.5 + 0.5;
    col += vec3<f32>(flash * pulse * 0.2);
    
    // Chromatic aberration on the edges
    let dist = length(uv);
    let aberration = dist * 0.01 * (1.0 + high_pulse);
    
    // Add some bloom/glow
    col = pow(col, vec3<f32>(0.8)); // Brighten
    col *= 1.2 + bass_pulse * 0.3;  // Boost with bass
    
    // Final color adjustments
    col = clamp(col, vec3<f32>(0.0), vec3<f32>(1.0));
    
    // Add subtle scanlines
    let scanline = sin(in.uv.y * uniforms.resolution.y * 0.5) * 0.03;
    col *= 1.0 - scanline;
    
    return vec4<f32>(col, 1.0);
}
