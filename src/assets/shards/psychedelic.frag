// Psychedelic audio-reactive shader

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

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    var uv = (in.uv * 2.0 - 1.0) * vec2<f32>(uniforms.resolution.x / uniforms.resolution.y, 1.0);
    
    let t = uniforms.time * 0.5;
    let bass = uniforms.audio_bass;
    let mid = uniforms.audio_mid;
    let high = uniforms.audio_high;
    
    // Create flowing patterns
    let flow = fbm(uv * 2.0 + vec2<f32>(t * 0.3, t * 0.2)) * 2.0 - 1.0;
    let flow2 = fbm(uv * 3.0 - vec2<f32>(t * 0.2, t * 0.3)) * 2.0 - 1.0;
    
    // Audio reactive distortion
    uv += vec2<f32>(flow, flow2) * (0.2 + bass * 0.3);
    
    // Multiple layers of noise
    let n1 = fbm(uv * 3.0 + t * 0.5);
    let n2 = fbm(uv * 5.0 - t * 0.3);
    let n3 = fbm(uv * 7.0 + t * 0.7);
    
    // Create psychedelic color patterns
    var col = vec3<f32>(
        0.5 + 0.5 * sin(n1 * 6.28 + t + bass * 3.14),
        0.5 + 0.5 * sin(n2 * 6.28 + t * 1.3 + mid * 3.14),
        0.5 + 0.5 * sin(n3 * 6.28 + t * 1.7 + high * 3.14)
    );
    
    // Add some glow
    let glow = pow(n1 * n2 * n3, 2.0);
    col += vec3<f32>(glow) * (0.5 + bass * 0.5);
    
    // Audio reactive brightness pulsing
    col *= 0.7 + 0.3 * sin(t * 2.0 + (bass + mid + high) * 3.14);
    
    return vec4<f32>(col, 1.0);
}
