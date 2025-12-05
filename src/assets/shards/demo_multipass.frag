// Demo: Multi-Pass Rendering with BufferA
// BufferA creates animated patterns, MainImage samples and displays them

// UNIFORMS STRUCTURE (REQUIRED)
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

// Texture binding for BufferA (MainImage only)
@group(1) @binding(0)
var bufferA: texture_2d<f32>;

@group(1) @binding(1)
var bufferSampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

// ========== VERTEX SHADER (SHARED) ==========
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(1.0, 1.0),
    );

    var tex_coords = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(1.0, 0.0),
    );

    var output: VertexOutput;
    output.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    output.tex_coords = tex_coords[vertex_index];
    return output;
}

// ========== BUFFER A FRAGMENT ==========
// Create animated spiral pattern with audio reactivity
@fragment
fn fs_buffer_a(@location(0) coords: vec2<f32>) -> @location(0) vec4<f32> {
    let uv = coords * 2.0 - 1.0; // -1 to 1
    let time = uniforms.time;
    
    // Polar coordinates
    let dist = length(uv);
    let angle = atan2(uv.y, uv.x);
    
    // Animated spiral with audio
    let spiral = sin(dist * 10.0 - time * 2.0 + angle * 3.0 + uniforms.audio_bass * 5.0);
    let rings = sin(dist * 15.0 + time + uniforms.audio_mid * 3.0);
    
    // Color based on position and audio
    let r = 0.5 + 0.5 * spiral * uniforms.audio_high;
    let g = 0.5 + 0.5 * rings;
    let b = 0.5 + 0.5 * sin(angle + time);
    
    return vec4<f32>(r, g, b, 1.0);
}

// ========== MAIN IMAGE FRAGMENT ==========
// Sample from BufferA and add effects
@fragment
fn fs_main(@location(0) coords: vec2<f32>) -> @location(0) vec4<f32> {
    let uv = coords;
    
    // Sample from BufferA texture
    let buffer_color = textureSample(bufferA, bufferSampler, uv);
    
    // Add vignette
    let center = vec2<f32>(0.5, 0.5);
    let dist = length(uv - center);
    let vignette = 1.0 - smoothstep(0.3, 0.9, dist);
    
    // Combine
    let final_color = buffer_color.rgb * vignette;
    
    return vec4<f32>(final_color, 1.0);
}
