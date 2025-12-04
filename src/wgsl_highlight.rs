use eframe::egui::{text::{LayoutJob, TextFormat}, Color32, FontId};

// Minimal WGSL tokenizer -> LayoutJob for syntax highlighting.
// No external crates. Handles line comments, block comments, strings, identifiers/keywords, numbers and punctuation.

fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '.'
}

pub fn layout_job_from_str(src: &str, font_size: f32) -> LayoutJob {
    let mut job = LayoutJob::default();
    let mono = FontId::monospace(font_size);

    let keywords = [
        "fn", "let", "var", "struct", "return", "if", "else", "for", "while",
        "const", "loop", "break", "continue", "switch", "case", "default",
        "true", "false",
    ];

    let types = ["f32", "u32", "i32", "vec2", "vec3", "vec4", "mat2", "mat3", "mat4"];

    let builtins = ["@vertex", "@fragment", "@group", "@binding", "@location", "@builtin"];

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
            job.append(&s, 0.0, TextFormat { font_id: mono.clone(), color: Color32::from_rgb(80, 200, 120), ..Default::default() });
            continue;
        }

        // Block comment: /* ... */
        if c == '/' && i + 1 < chars.len() && chars[i + 1] == '*' {
            let start = i;
            i += 2;
            while i + 1 < chars.len() {
                if chars[i] == '*' && chars[i + 1] == '/' {
                    i += 2;
                    break;
                }
                i += 1;
            }
            // If we reached the end without closing, i == len
            let s: String = chars[start..i].iter().collect();
            job.append(&s, 0.0, TextFormat { font_id: mono.clone(), color: Color32::from_rgb(80, 200, 120), ..Default::default() });
            continue;
        }

        // Strings: double-quoted or single-quoted with escapes
        if c == '"' || c == '\'' {
            let quote = c;
            let start = i;
            i += 1;
            while i < chars.len() {
                if chars[i] == '\\' {
                    // skip escape and next char if present
                    i += 1;
                    if i < chars.len() { i += 1; }
                    continue;
                }
                if chars[i] == quote {
                    i += 1; // include closing quote
                    break;
                }
                i += 1;
            }
            let s: String = chars[start..i].iter().collect();
            job.append(&s, 0.0, TextFormat { font_id: mono.clone(), color: Color32::from_rgb(200, 140, 200), ..Default::default() });
            continue;
        }

        // Whitespace
        if c.is_whitespace() {
            let start = i;
            while i < chars.len() && chars[i].is_whitespace() { i += 1; }
            let s: String = chars[start..i].iter().collect();
            job.append(&s, 0.0, TextFormat { font_id: mono.clone(), color: Color32::WHITE, ..Default::default() });
            continue;
        }

        // Identifier or keyword (allow leading '@')
        if c.is_alphabetic() || c == '_' || c == '@' {
            let start = i;
            i += 1;
            while i < chars.len() && is_ident_char(chars[i]) { i += 1; }
            let s: String = chars[start..i].iter().collect();
            let lower = s.to_lowercase();
            if keywords.contains(&lower.as_str()) {
                job.append(&s, 0.0, TextFormat { font_id: mono.clone(), color: Color32::from_rgb(200, 120, 20), ..Default::default() });
            } else if types.iter().any(|t| lower.starts_with(t)) {
                job.append(&s, 0.0, TextFormat { font_id: mono.clone(), color: Color32::from_rgb(170, 120, 220), ..Default::default() });
            } else if builtins.iter().any(|b| s.starts_with(b)) || s.starts_with('@') {
                job.append(&s, 0.0, TextFormat { font_id: mono.clone(), color: Color32::from_rgb(120, 200, 220), ..Default::default() });
            } else {
                job.append(&s, 0.0, TextFormat { font_id: mono.clone(), color: Color32::WHITE, ..Default::default() });
            }
            continue;
        }

        // Numbers (decimal, with dot, or hex prefix 0x)
        if c.is_numeric() {
            let start = i;
            if c == '0' && i + 1 < chars.len() && (chars[i + 1] == 'x' || chars[i + 1] == 'X') {
                // hex
                i += 2;
                while i < chars.len() && chars[i].is_ascii_hexdigit() { i += 1; }
            } else {
                i += 1;
                while i < chars.len() && (chars[i].is_numeric() || chars[i] == '.' ) { i += 1; }
            }
            let s: String = chars[start..i].iter().collect();
            job.append(&s, 0.0, TextFormat { font_id: mono.clone(), color: Color32::from_rgb(220, 160, 80), ..Default::default() });
            continue;
        }

        // Anything else: punctuation/symbol
        {
            let s = c.to_string();
            job.append(&s, 0.0, TextFormat { font_id: mono.clone(), color: Color32::LIGHT_GRAY, ..Default::default() });
            i += 1;
        }
    }

    job
}
