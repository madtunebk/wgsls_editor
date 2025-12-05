// Simple fragment shader example - no boilerplate needed!
// Uniforms and VSOut are auto-injected

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    let uv = in.uv;
    
    // Animated gradient based on UV and time
    let r = sin(uv.x * 3.14159 + uniforms.time) * 0.5 + 0.5;
    let g = sin(uv.y * 3.14159 - uniforms.time) * 0.5 + 0.5;
    let b = sin((uv.x + uv.y) * 3.14159 + uniforms.time * 0.5) * 0.5 + 0.5;
    
    // Pulse with bass
    let pulse = 1.0 + uniforms.audio_bass * 0.3;
    
    return vec4<f32>(r * pulse, g * pulse, b * pulse, 1.0);
}
