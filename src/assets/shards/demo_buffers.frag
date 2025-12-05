// Demo: Multi-Buffer Shader
// This demonstrates Fragment (MainImage), Buffer A, and Vertex coordination

// UNIFORMS STRUCTURE (Required for all shaders)
struct Uniforms {
    time: f32,
    resolution: vec2<f32>,
    mouse: vec2<f32>,
    bass_energy: f32,
    mid_energy: f32,
    high_energy: f32,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

// MAIN_IMAGE FRAGMENT
// This is the final output - displays Buffer A with effects
@fragment
fn fs_main(@location(0) tex_coords: vec2<f32>) -> @location(0) vec4<f32> {
    let uv = tex_coords;
    let time = uniforms.time;
    
    // Sample from Buffer A (would need texture binding in real implementation)
    // For now, create a gradient based on position
    let buffer_a_color = vec3<f32>(
        0.5 + 0.5 * sin(uv.x * 10.0 + time),
        0.5 + 0.5 * sin(uv.y * 10.0 + time * 0.8),
        0.5 + 0.5 * sin((uv.x + uv.y) * 5.0 + time * 1.2)
    );
    
    // Add vignette effect
    let dist = length(uv - vec2<f32>(0.5, 0.5));
    let vignette = 1.0 - smoothstep(0.3, 0.8, dist);
    
    // Combine
    let final_color = buffer_a_color * vignette;
    
    return vec4<f32>(final_color, 1.0);
}

// BUFFER_A FRAGMENT
// This creates animated patterns that feed into MainImage
@fragment
fn fs_buffer_a(@location(0) tex_coords: vec2<f32>) -> @location(0) vec4<f32> {
    let uv = tex_coords * 2.0 - 1.0;
    let time = uniforms.time;
    
    // Rotating spiral pattern
    let angle = atan2(uv.y, uv.x);
    let radius = length(uv);
    
    let spiral = sin(angle * 5.0 + radius * 10.0 - time * 2.0);
    let rings = sin(radius * 20.0 - time * 3.0);
    
    let pattern = spiral * rings;
    
    // Audio reactive (if audio is available)
    let bass = uniforms.bass_energy;
    let mid = uniforms.mid_energy;
    let high = uniforms.high_energy;
    
    let color = vec3<f32>(
        0.5 + 0.5 * pattern + bass * 0.3,
        0.5 + 0.3 * sin(time + pattern) + mid * 0.3,
        0.5 + 0.3 * cos(time * 0.5 + pattern) + high * 0.3
    );
    
    return vec4<f32>(color, 1.0);
}

// SHARED VERTEX SHADER
// This creates the geometry - can be customized per buffer
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let time = uniforms.time;
    
    // Full-screen quad vertices
    let x = f32((vertex_index & 1u) << 1u);
    let y = f32((vertex_index & 2u));
    
    // Optional: Add vertex animation
    // Uncomment to make the quad wobble
    // let wobble = sin(time + x * 3.14159) * 0.1;
    
    out.position = vec4<f32>(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0);
    out.tex_coords = vec2<f32>(x, 1.0 - y);
    
    return out;
}

// USAGE INSTRUCTIONS:
// 1. Copy MAIN_IMAGE section to Fragment tab (default view)
// 2. Copy BUFFER_A section to Buffer A tab -> Fragment view
// 3. Copy SHARED VERTEX section to Vertex tab (works for all buffers)
// 4. Click "Apply Shader" to see the multi-buffer effect
// 5. Try loading audio for reactive visuals!
