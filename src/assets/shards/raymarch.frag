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

// 3D rotation matrix
fn rotate_y(a: f32) -> mat3x3<f32> {
    let s = sin(a);
    let c = cos(a);
    return mat3x3<f32>(
        c, 0.0, s,
        0.0, 1.0, 0.0,
        -s, 0.0, c
    );
}

fn rotate_x(a: f32) -> mat3x3<f32> {
    let s = sin(a);
    let c = cos(a);
    return mat3x3<f32>(
        1.0, 0.0, 0.0,
        0.0, c, -s,
        0.0, s, c
    );
}

// SDF for a box
fn sd_box(p: vec3<f32>, b: vec3<f32>) -> f32 {
    let q = abs(p) - b;
    return length(max(q, vec3<f32>(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0);
}

// Raymarching
fn scene(p: vec3<f32>) -> f32 {
    var pos = p;
    let t = uniforms.time;
    let bass = uniforms.audio_bass;
    
    // Infinite repetition
    pos = vec3<f32>(
        (pos.x + 1.0) % 2.0 - 1.0,
        (pos.y + 1.0) % 2.0 - 1.0,
        pos.z
    );
    
    // Rotate boxes
    pos = rotate_y(t + bass * 2.0) * rotate_x(t * 0.7) * pos;
    
    // Audio reactive size
    let size = vec3<f32>(0.3 + bass * 0.2, 0.3 + uniforms.audio_mid * 0.2, 0.3 + uniforms.audio_high * 0.2);
    
    return sd_box(pos, size);
}

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    var uv = (in.uv * 2.0 - 1.0) * vec2<f32>(uniforms.resolution.x / uniforms.resolution.y, 1.0);
    
    let t = uniforms.time;
    
    // Camera
    var ro = vec3<f32>(0.0, 0.0, -3.0 - sin(t * 0.5) * 2.0);
    let rd = normalize(vec3<f32>(uv, 1.5));
    
    // Raymarch
    var total_dist = 0.0;
    var col = vec3<f32>(0.0);
    
    for (var i = 0; i < 80; i++) {
        let p = ro + rd * total_dist;
        let d = scene(p);
        
        if (d < 0.001 || total_dist > 20.0) {
            break;
        }
        
        total_dist += d;
        
        // Glow effect
        let glow = 0.02 / (d * d);
        let freq_color = vec3<f32>(
            uniforms.audio_bass,
            uniforms.audio_mid,
            uniforms.audio_high
        );
        col += glow * freq_color * 0.01;
    }
    
    // Color based on distance traveled
    let depth_color = 1.0 - total_dist / 20.0;
    col += vec3<f32>(depth_color) * vec3<f32>(0.8, 0.4, 1.0) * 0.1;
    
    // Boost with audio
    col *= 1.0 + (uniforms.audio_bass + uniforms.audio_mid + uniforms.audio_high) * 0.3;
    
    return vec4<f32>(col, 1.0);
}
