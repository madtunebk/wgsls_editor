use eframe::egui::{text::{LayoutJob, TextFormat}, Color32, FontId};

// Enhanced WGSL syntax highlighter. No external crates.
// Highlights: keywords, types, attributes, intrinsics, address spaces, builtins, numbers, strings, comments, punctuation.

fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '.'
}

fn peek_non_ws(chars: &[char], mut i: usize) -> Option<char> {
    while i < chars.len() {
        let c = chars[i];
        if !c.is_whitespace() { return Some(c); }
        i += 1;
    }
    None
}

pub fn layout_job_from_str(src: &str, font_size: f32) -> LayoutJob {
    let mut job = LayoutJob::default();
    let mono = FontId::monospace(font_size);

    // Language keywords
    const KEYWORDS: &[&str] = &[
        "fn","let","var","const","override","struct","return","if","else","switch","case","default",
        "loop","break","continue","while","for","discard","enable","requires",
    ];

    // Common scalar/vector/matrix and container types
    const TYPES: &[&str] = &[
        "bool","i32","u32","f32","f16","vec2","vec3","vec4",
        "mat2x2","mat2x3","mat2x4","mat3x2","mat3x3","mat3x4","mat4x2","mat4x3","mat4x4",
        "array","ptr","sampler","sampler_comparison",
        "texture_1d","texture_2d","texture_2d_array","texture_3d","texture_cube",
        "texture_storage_2d","texture_storage_2d_array","texture_storage_3d",
    ];

    // Attributes and common builtin decorations
    const ATTRS: &[&str] = &[
        "@vertex","@fragment","@compute","@group","@binding","@location","@builtin","@interpolate","@id","@workgroup_size",
    ];

    // Address spaces and access modes
    const ADDR_OR_ACCESS: &[&str] = &[
        "function","private","workgroup","uniform","storage","read","write","read_write",
    ];

    // A small set of intrinsics and texture helpers
    const INTRINSICS: &[&str] = &[
        "abs","min","max","clamp","mix","select","dot","cross","normalize","length","distance","pow","exp","log",
        "sin","cos","tan","asin","acos","atan","floor","ceil","round","fract","sqrt","rsqrt",
        "textureSample","textureLoad","textureStore","textureDimensions",
    ];

    // Colors
    let col_kw = Color32::from_rgb(220, 140, 60);       // keywords
    let col_ty = Color32::from_rgb(170, 120, 230);      // types
    let col_attr = Color32::from_rgb(110, 200, 230);    // attributes
    let col_intr = Color32::from_rgb(120, 180, 255);    // intrinsics / fn calls
    let col_access = Color32::from_rgb(200, 170, 90);   // address/access
    let col_num = Color32::from_rgb(230, 170, 90);      // numbers
    let col_str = Color32::from_rgb(200, 140, 200);     // strings
    let col_cmt = Color32::from_rgb(90, 180, 120);      // comments
    let col_punc = Color32::from_gray(180);             // punctuation

    let tf_default = TextFormat { font_id: mono.clone(), color: Color32::WHITE, ..Default::default() };
    let tf_kw = TextFormat { font_id: mono.clone(), color: col_kw, ..Default::default() };
    let tf_ty = TextFormat { font_id: mono.clone(), color: col_ty, ..Default::default() };
    let tf_attr = TextFormat { font_id: mono.clone(), color: col_attr, ..Default::default() };
    let tf_intr = TextFormat { font_id: mono.clone(), color: col_intr, ..Default::default() };
    let tf_access = TextFormat { font_id: mono.clone(), color: col_access, ..Default::default() };
    let tf_num = TextFormat { font_id: mono.clone(), color: col_num, ..Default::default() };
    let tf_str = TextFormat { font_id: mono.clone(), color: col_str, ..Default::default() };
    let tf_cmt = TextFormat { font_id: mono.clone(), color: col_cmt, italics: true, ..Default::default() };
    let tf_punc = TextFormat { font_id: mono.clone(), color: col_punc, ..Default::default() };

    let chars: Vec<char> = src.chars().collect();
    let mut i = 0usize;
    while i < chars.len() {
        let c = chars[i];

        // Line comment: // ...\n
        if c == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
            let start = i;
            i += 2;
            while i < chars.len() && chars[i] != '\n' { i += 1; }
            let s: String = chars[start..i].iter().collect();
            job.append(&s, 0.0, tf_cmt.clone());
            continue;
        }

        // Block comment: /* ... */ (non-nesting)
        if c == '/' && i + 1 < chars.len() && chars[i + 1] == '*' {
            let start = i;
            i += 2;
            while i + 1 < chars.len() {
                if chars[i] == '*' && chars[i + 1] == '/' { i += 2; break; }
                i += 1;
            }
            let s: String = chars[start..i].iter().collect();
            job.append(&s, 0.0, tf_cmt.clone());
            continue;
        }

        // Strings
        if c == '"' || c == '\'' {
            let quote = c;
            let start = i;
            i += 1;
            while i < chars.len() {
                if chars[i] == '\\' { i += 1; if i < chars.len() { i += 1; } continue; }
                if chars[i] == quote { i += 1; break; }
                i += 1;
            }
            let s: String = chars[start..i].iter().collect();
            job.append(&s, 0.0, tf_str.clone());
            continue;
        }

        // Whitespace
        if c.is_whitespace() {
            let start = i;
            while i < chars.len() && chars[i].is_whitespace() { i += 1; }
            let s: String = chars[start..i].iter().collect();
            job.append(&s, 0.0, tf_default.clone());
            continue;
        }

        // Identifier or attribute (allow leading '@')
        if c.is_alphabetic() || c == '_' || c == '@' {
            let start = i;
            i += 1;
            while i < chars.len() && is_ident_char(chars[i]) { i += 1; }
            let s: String = chars[start..i].iter().collect();
            let lower = s.to_lowercase();

            if s.starts_with('@') && ATTRS.iter().any(|a| s.starts_with(a)) {
                job.append(&s, 0.0, tf_attr.clone());
                continue;
            }

            if KEYWORDS.contains(&lower.as_str()) {
                job.append(&s, 0.0, tf_kw.clone());
                continue;
            }

            if TYPES.iter().any(|t| lower.starts_with(t)) {
                job.append(&s, 0.0, tf_ty.clone());
                continue;
            }

            if ADDR_OR_ACCESS.contains(&lower.as_str()) {
                job.append(&s, 0.0, tf_access.clone());
                continue;
            }

            // Intrinsics or function calls: identify if followed by '(' and known intrinsic name
            let next = peek_non_ws(&chars, i);
            if next == Some('(') && INTRINSICS.contains(&s.as_str()) {
                job.append(&s, 0.0, tf_intr.clone());
                continue;
            }

            job.append(&s, 0.0, tf_default.clone());
            continue;
        }

        // Numbers: decimal, float with dot/exponent, hex (0x) with underscores; optional literal suffix u/i/f
        if c.is_ascii_digit() || (c == '.' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit()) {
            let start = i;
            if c == '0' && i + 1 < chars.len() && (chars[i + 1] == 'x' || chars[i + 1] == 'X') {
                i += 2;
                while i < chars.len() && (chars[i].is_ascii_hexdigit() || chars[i] == '_') { i += 1; }
            } else {
                // integer/float
                i += 1;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '_' || chars[i] == '.') { i += 1; }
                // exponent part
                if i < chars.len() && (chars[i] == 'e' || chars[i] == 'E') {
                    i += 1;
                    if i < chars.len() && (chars[i] == '+' || chars[i] == '-') { i += 1; }
                    while i < chars.len() && chars[i].is_ascii_digit() { i += 1; }
                }
            }
            // optional literal suffix u/i/f
            if i < chars.len() && (chars[i] == 'u' || chars[i] == 'i' || chars[i] == 'f') { i += 1; }
            let s: String = chars[start..i].iter().collect();
            job.append(&s, 0.0, tf_num.clone());
            continue;
        }

        // Punctuation / symbols
        {
            let s = c.to_string();
            job.append(&s, 0.0, tf_punc.clone());
            i += 1;
        }
    }

    job
}
