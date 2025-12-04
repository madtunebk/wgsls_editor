// Basic WGSL autocomplete vocabulary. Lightweight, static list.
pub fn suggestions() -> &'static [&'static str] {
    const WORDS: &[&str] = &[
        // Keywords
        "fn", "let", "var", "const", "override", "struct", "return", "if", "else", "switch",
        "case", "default", "loop", "break", "continue", "while", "for", "discard", "enable",
        // Types
        "bool", "i32", "u32", "f32", "f16", "vec2", "vec3", "vec4",
        "mat2x2", "mat2x3", "mat2x4", "mat3x2", "mat3x3", "mat3x4", "mat4x2", "mat4x3", "mat4x4",
        "array", "ptr", "sampler", "sampler_comparison",
        "texture_1d", "texture_2d", "texture_2d_array", "texture_3d", "texture_cube",
        "texture_storage_2d", "texture_storage_2d_array", "texture_storage_3d",
        // Attributes / decorations
        "@vertex", "@fragment", "@compute", "@group", "@binding", "@location", "@builtin",
        "@interpolate", "@id", "@workgroup_size",
        // Address spaces / access
        "function", "private", "workgroup", "uniform", "storage", "read", "write", "read_write",
        // Builtins / intrinsics
        "abs", "min", "max", "clamp", "mix", "select", "dot", "cross", "normalize", "length",
        "distance", "pow", "exp", "log", "sin", "cos", "tan", "asin", "acos", "atan",
        "floor", "ceil", "round", "fract", "sqrt", "rsqrt",
        "textureSample", "textureLoad", "textureStore", "textureDimensions",
    ];
    WORDS
}

