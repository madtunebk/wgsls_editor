use std::collections::BTreeSet;
use std::sync::OnceLock;

#[derive(Clone, Debug)]
pub enum SuggestionKind {
    Keyword,
    Attribute,
    AddressSpace,
    TypeCtor,
    BuiltinFn,
    Swizzle,
    Local,
}

#[derive(Clone, Debug)]
pub struct Suggestion {
    pub label: String,
    pub detail: Option<String>,
    pub doc: Option<String>,
    pub insert_suffix: Option<String>,
    #[allow(dead_code)]
    pub kind: SuggestionKind,
}

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

fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn current_prefix(src: &str, caret_char: usize) -> (String, bool, bool) {
    let chars: Vec<char> = src.chars().collect();
    let mut i = caret_char.min(chars.len());
    // Build prefix backwards
    while i > 0 {
        let c = chars[i - 1];
        if is_ident_char(c) || c == '@' { i -= 1; } else { break; }
    }
    let prefix: String = chars[i..caret_char.min(chars.len())].iter().collect();
    let dot_ctx = i > 0 && chars[i - 1] == '.';
    let at_ctx = prefix.starts_with('@');
    (prefix, dot_ctx, at_ctx)
}

fn local_symbols(src: &str) -> Vec<String> {
    let mut set = BTreeSet::new();
    // very light-weight scan for identifiers
    let mut cur = String::new();
    for c in src.chars() {
        if is_ident_char(c) {
            cur.push(c);
        } else {
            if cur.len() > 1 { set.insert(cur.clone()); }
            cur.clear();
        }
    }
    if cur.len() > 1 { set.insert(cur); }
    set.into_iter().collect()
}

fn swizzle_suggestions() -> &'static [&'static str] {
    &[
        "x","y","z","w","r","g","b","a",
        "xy","xz","yz","xyz","rgba","rgb","xyzw",
    ]
}

fn attr_suggestions() -> &'static [(&'static str, &'static str)] {
    &[
        ("@vertex", "Entry point for vertex stage"),
        ("@fragment", "Entry point for fragment stage"),
        ("@compute", "Entry point for compute stage"),
        ("@group", "Bind group index for a resource"),
        ("@binding", "Binding index within a group"),
        ("@location", "User-defined IO location"),
        ("@builtin", "Use a built-in IO semantic"),
        ("@workgroup_size", "Workgroup size for compute"),
        ("@interpolate", "Interpolation mode for fragment inputs"),
        ("@id", "Pipeline constant ID"),
    ]
}

fn address_spaces() -> &'static [(&'static str, &'static str)] {
    &[
        ("function", "Default local function storage"),
        ("private", "Module-scope private storage"),
        ("workgroup", "Shared workgroup memory"),
        ("uniform", "Uniform-constant storage"),
        ("storage", "Storage buffer (read/write)"),
    ]
}

#[derive(Clone, Copy)]
struct BuiltinFn { name: &'static str, signature: &'static str, doc: &'static str }

fn builtin_functions() -> &'static [BuiltinFn] {
    &[
        BuiltinFn { name: "sin", signature: "fn sin(x: f32) -> f32", doc: "Sine of angle (radians)." },
        BuiltinFn { name: "cos", signature: "fn cos(x: f32) -> f32", doc: "Cosine of angle (radians)." },
        BuiltinFn { name: "tan", signature: "fn tan(x: f32) -> f32", doc: "Tangent of angle (radians)." },
        BuiltinFn { name: "asin", signature: "fn asin(x: f32) -> f32", doc: "Arcsine; returns radians." },
        BuiltinFn { name: "acos", signature: "fn acos(x: f32) -> f32", doc: "Arccosine; returns radians." },
        BuiltinFn { name: "atan", signature: "fn atan(y_over_x: f32) -> f32", doc: "Arctangent; returns radians." },
        BuiltinFn { name: "pow", signature: "fn pow(x: f32, y: f32) -> f32", doc: "x raised to the power y." },
        BuiltinFn { name: "fract", signature: "fn fract(x: f32) -> f32", doc: "Fractional part of x." },
        BuiltinFn { name: "floor", signature: "fn floor(x: f32) -> f32", doc: "Largest integer <= x." },
        BuiltinFn { name: "ceil", signature: "fn ceil(x: f32) -> f32", doc: "Smallest integer >= x." },
        BuiltinFn { name: "round", signature: "fn round(x: f32) -> f32", doc: "Nearest integer to x." },
        BuiltinFn { name: "sqrt", signature: "fn sqrt(x: f32) -> f32", doc: "Square root of x." },
        BuiltinFn { name: "rsqrt", signature: "fn rsqrt(x: f32) -> f32", doc: "Reciprocal square root of x." },
        BuiltinFn { name: "min", signature: "fn min(x: f32, y: f32) -> f32", doc: "Minimum of x and y." },
        BuiltinFn { name: "max", signature: "fn max(x: f32, y: f32) -> f32", doc: "Maximum of x and y." },
        BuiltinFn { name: "clamp", signature: "fn clamp(x: f32, a: f32, b: f32) -> f32", doc: "Clamp x into [a,b]." },
        BuiltinFn { name: "mix", signature: "fn mix(x: f32, y: f32, a: f32) -> f32", doc: "Linear interpolate." },
        BuiltinFn { name: "dot", signature: "fn dot(x: vecN<f32>, y: vecN<f32>) -> f32", doc: "Dot product." },
        BuiltinFn { name: "cross", signature: "fn cross(x: vec3<f32>, y: vec3<f32>) -> vec3<f32>", doc: "Cross product." },
        BuiltinFn { name: "normalize", signature: "fn normalize(x: vecN<f32>) -> vecN<f32>", doc: "Normalize vector." },
        BuiltinFn { name: "length", signature: "fn length(x: vecN<f32>) -> f32", doc: "Vector length." },
        BuiltinFn { name: "distance", signature: "fn distance(a: vecN<f32>, b: vecN<f32>) -> f32", doc: "Distance between a and b." },
        BuiltinFn { name: "textureSample", signature: "fn textureSample(t, s, uv) -> vec4<f32>", doc: "Sample a texture with sampler." },
        BuiltinFn { name: "textureLoad", signature: "fn textureLoad(t, coords, level) -> vec4<f32>", doc: "Load a texel by integer coords." },
    ]
}

// Optional JSON extension loaded from data/wgsl_builtins.json at runtime
#[derive(serde::Deserialize)]
#[allow(dead_code)]
struct JsonSuggestion {
    label: String,
    #[serde(default)]
    detail: Option<String>,
    #[serde(default)]
    doc: Option<String>,
    #[serde(default)]
    insert_suffix: Option<String>,
    #[serde(default)]
    kind: Option<String>,
}

fn load_json_builtins() -> Vec<Suggestion> {
    let path = "data/wgsl_builtins.json";
    match std::fs::read_to_string(path) {
        Ok(s) => match serde_json::from_str::<Vec<JsonSuggestion>>(&s) {
            Ok(items) => items
                .into_iter()
                .map(|j| Suggestion { label: j.label, detail: j.detail, doc: j.doc, insert_suffix: j.insert_suffix, kind: SuggestionKind::BuiltinFn })
                .collect(),
            Err(_) => Vec::new(),
        },
        Err(_) => Vec::new(),
    }
}

fn extract_group_alternatives(pat: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut depth = 0usize;
    let mut start = None;
    for (i, ch) in pat.chars().enumerate() {
        if ch == '(' {
            if depth == 0 { start = Some(i + 1); }
            depth += 1;
        } else if ch == ')' {
            if depth > 0 {
                depth -= 1;
                if depth == 0 {
                    if let Some(s) = start {
                        let inner = &pat[s..i];
                        out = inner
                            .split('|')
                            .map(|t| t.trim())
                            .map(|t| t.trim_matches(['\\', 'b', 'B', '^', '$', '{', '}', '[', ']', '?', '*', '+', ':', '!'].as_ref()))
                            .map(|t| t.trim_matches(|c: char| !c.is_alphanumeric() && c != '_' && c != '@'))
                            .filter(|t| !t.is_empty())
                            .map(|t| t.to_string())
                            .collect();
                        break;
                    }
                }
            }
        }
    }
    out
}

fn map_scope_to_kind(scope: &str) -> SuggestionKind {
    let s = scope.to_lowercase();
    if s.contains("attribute") || s.contains("decorator") { SuggestionKind::Attribute }
    else if s.contains("storage") { SuggestionKind::AddressSpace }
    else if s.contains("function") { SuggestionKind::BuiltinFn }
    else if s.contains("type") { SuggestionKind::TypeCtor }
    else if s.contains("keyword") { SuggestionKind::Keyword }
    else { SuggestionKind::Keyword }
}

fn load_textmate_tokens_once() -> &'static Vec<Suggestion> {
    static TM: OnceLock<Vec<Suggestion>> = OnceLock::new();
    TM.get_or_init(|| {
        let path = "data/wgsl.tmLanguage.json";
        let s = match std::fs::read_to_string(path) { Ok(s) => s, Err(_) => return Vec::new() };
        let v: serde_json::Value = match serde_json::from_str(&s) { Ok(v) => v, Err(_) => return Vec::new() };
        let mut result: Vec<Suggestion> = Vec::new();
        let mut visit = |node: &serde_json::Value| {
            if let Some(obj) = node.as_object() {
                let scope = obj.get("name").and_then(|n| n.as_str()).unwrap_or("");
                let kind = map_scope_to_kind(scope);
                if let Some(mat) = obj.get("match").and_then(|m| m.as_str()) {
                    for tok in extract_group_alternatives(mat) {
                        let mut insert_suffix = None;
                        if matches!(kind, SuggestionKind::BuiltinFn | SuggestionKind::TypeCtor) { insert_suffix = Some("()".to_string()); }
                        result.push(Suggestion { label: tok, detail: Some(scope.to_string()), doc: None, insert_suffix, kind: kind.clone() });
                    }
                }
            }
        };
        if let Some(patterns) = v.get("patterns").and_then(|p| p.as_array()) {
            for p in patterns { visit(p); }
        }
        if let Some(repo) = v.get("repository").and_then(|r| r.as_object()) {
            for (_k, val) in repo.iter() {
                visit(val);
                if let Some(patts) = val.get("patterns").and_then(|p| p.as_array()) {
                    for p in patts { visit(p); }
                }
            }
        }
        let mut seen = BTreeSet::new();
        result.retain(|s| seen.insert(s.label.clone()));
        result
    })
}

pub fn suggestions_for_context(src: &str, caret_char: usize) -> Vec<Suggestion> {
    let (prefix, dot_ctx, at_ctx) = current_prefix(src, caret_char);
    let mut out: Vec<Suggestion> = Vec::new();

    if dot_ctx {
        for s in swizzle_suggestions() {
            if prefix.is_empty() || s.starts_with(prefix.as_str()) {
                out.push(Suggestion { label: (*s).to_string(), detail: Some("swizzle".into()), doc: None, insert_suffix: None, kind: SuggestionKind::Swizzle });
            }
        }
        return out;
    }

    if at_ctx {
        for (s, doc) in attr_suggestions() {
            if s.starts_with(prefix.as_str()) {
                out.push(Suggestion { label: (*s).to_string(), detail: Some("attribute".into()), doc: Some(doc.to_string()), insert_suffix: None, kind: SuggestionKind::Attribute });
            }
        }
        return out;
    }

    // Simple left-context lookbehind for address-space suggestions like var<workgroup>
    let left = src[..caret_char.min(src.len())].chars().rev().take(32).collect::<String>();
    let left_lower = left.to_lowercase();
    let is_after_var_angle = left_lower.contains("<rav") || left_lower.starts_with('<'); // naive detection for "var<"
    let is_after_ptr_angle = left_lower.contains("<rtp"); // naive detection for "ptr<"
    if is_after_var_angle || is_after_ptr_angle {
        for (as_name, doc) in address_spaces() {
            if prefix.is_empty() || as_name.starts_with(prefix.as_str()) {
                out.push(Suggestion { label: (*as_name).to_string(), detail: Some("address space".into()), doc: Some(doc.to_string()), insert_suffix: None, kind: SuggestionKind::AddressSpace });
            }
        }
    }

    // base vocab
    for w in suggestions().iter() {
        if prefix.is_empty() || w.starts_with(prefix.as_str()) {
            out.push(Suggestion { label: (*w).to_string(), detail: None, doc: None, insert_suffix: None, kind: SuggestionKind::Keyword });
        }
    }
    // builtin functions with signatures
    for b in builtin_functions() {
        if prefix.is_empty() || b.name.starts_with(prefix.as_str()) {
            out.push(Suggestion { label: b.name.to_string(), detail: Some(b.signature.to_string()), doc: Some(b.doc.to_string()), insert_suffix: Some("()".to_string()), kind: SuggestionKind::BuiltinFn });
        }
    }
    // constructors (commonly used types)
    for ctor in [
        "vec2", "vec3", "vec4",
        "mat2x2", "mat2x3", "mat2x4", "mat3x2", "mat3x3", "mat3x4", "mat4x2", "mat4x3", "mat4x4",
    ] {
        if prefix.is_empty() || ctor.starts_with(prefix.as_str()) {
            out.push(Suggestion { label: ctor.to_string(), detail: Some("constructor".into()), doc: None, insert_suffix: Some("()".to_string()), kind: SuggestionKind::TypeCtor });
        }
    }
    // JSON-extended builtins
    for s in load_json_builtins() {
        if prefix.is_empty() || s.label.starts_with(prefix.as_str()) {
            out.push(s);
        }
    }
    // TextMate tokens (if wgsl.tmLanguage.json is present)
    for s in load_textmate_tokens_once().iter() {
        if prefix.is_empty() || s.label.starts_with(prefix.as_str()) {
            out.push(s.clone());
        }
    }
    // locals
    for sym in local_symbols(src) {
        if prefix.is_empty() || sym.starts_with(prefix.as_str()) {
            out.push(Suggestion { label: sym, detail: Some("local".into()), doc: None, insert_suffix: None, kind: SuggestionKind::Local });
        }
    }
    // de-dup while preserving insertion order
    let mut seen = BTreeSet::new();
    out.retain(|s| seen.insert(s.label.clone()));
    out.truncate(128);
    out
}
