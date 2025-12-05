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
        
        // Create cells
        let cell = step(0.5, grid_x) * step(0.5, grid_y);
        
        // Audio reactive colors per layer
        let hue = i / 5.0 + t * 0.2 + bass * 0.5;
        let sat = 0.6 + mid * 0.4;
        let bright = cell * (0.3 + high * 0.7);
        
        // HSV to RGB conversion
        let K = vec4<f32>(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
        let p = abs(fract(vec3<f32>(hue) + K.xyz) * 6.0 - K.www);
        let rgb = bright * mix(K.xxx, clamp(p - K.xxx, vec3<f32>(0.0), vec3<f32>(1.0)), sat);
        
        col += rgb / (1.0 + i);
    }
    
    // Vignette
    col *= 1.0 - r * 0.5;
    
    // Glow/bloom
    col = pow(col, vec3<f32>(0.7));
    
    return vec4<f32>(col, 1.0);
}
