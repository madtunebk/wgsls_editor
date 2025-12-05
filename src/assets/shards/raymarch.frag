// 3D raymarching demo with audio reactivity

// 3D rotation matrices
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

// Scene SDF
fn map(p: vec3<f32>, t: f32, bass: f32) -> f32 {
    var p_var = p;
    
    // Rotate the scene
    p_var = rotate_y(t * 0.5) * p_var;
    p_var = rotate_x(t * 0.3) * p_var;
    
    // Audio reactive box size
    let box_size = vec3<f32>(0.5 + bass * 0.3);
    let d1 = sd_box(p_var, box_size);
    
    // Add a repeating grid of boxes
    let grid_p = p_var - vec3<f32>(
        round(p_var.x / 2.0) * 2.0,
        round(p_var.y / 2.0) * 2.0,
        round(p_var.z / 2.0) * 2.0
    );
    let d2 = sd_box(grid_p, vec3<f32>(0.2));
    
    return min(d1, d2);
}

// Calculate normal
fn calc_normal(p: vec3<f32>, t: f32, bass: f32) -> vec3<f32> {
    let e = vec2<f32>(0.001, 0.0);
    return normalize(vec3<f32>(
        map(p + e.xyy, t, bass) - map(p - e.xyy, t, bass),
        map(p + e.yxy, t, bass) - map(p - e.yxy, t, bass),
        map(p + e.yyx, t, bass) - map(p - e.yyx, t, bass)
    ));
}

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    var uv = (in.uv * 2.0 - 1.0) * vec2<f32>(uniforms.resolution.x / uniforms.resolution.y, 1.0);
    
    let t = uniforms.time;
    let bass = uniforms.audio_bass;
    let mid = uniforms.audio_mid;
    let high = uniforms.audio_high;
    
    // Camera setup
    let camera_pos = vec3<f32>(0.0, 0.0, -3.0 - bass);
    let ray_dir = normalize(vec3<f32>(uv, 1.0));
    
    // Raymarch
    var ray_pos = camera_pos;
    var total_dist = 0.0;
    var hit = false;
    
    for (var i = 0; i < 64; i++) {
        let dist = map(ray_pos, t, bass);
        if dist < 0.001 {
            hit = true;
            break;
        }
        ray_pos += ray_dir * dist;
        total_dist += dist;
        if total_dist > 20.0 {
            break;
        }
    }
    
    var col = vec3<f32>(0.0);
    
    if hit {
        let normal = calc_normal(ray_pos, t, bass);
        let light_dir = normalize(vec3<f32>(1.0, 1.0, -1.0));
        let diffuse = max(dot(normal, light_dir), 0.0);
        
        // Audio reactive colors
        col = vec3<f32>(
            0.5 + 0.5 * sin(total_dist + t + bass * 3.14),
            0.5 + 0.5 * sin(total_dist + t * 1.3 + mid * 3.14),
            0.5 + 0.5 * sin(total_dist + t * 1.7 + high * 3.14)
        ) * diffuse;
        
        // Add some ambient
        col += vec3<f32>(0.1, 0.1, 0.15);
    } else {
        // Background gradient
        col = vec3<f32>(0.0, 0.0, 0.1) * (1.0 - length(uv) * 0.5);
    }
    
    return vec4<f32>(col, 1.0);
}
