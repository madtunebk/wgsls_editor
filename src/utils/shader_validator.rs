//! WGSL Shader Validation
//! 
//! Validates shaders before passing to pipeline to catch errors early
//! and provide helpful error messages in the UI.

use crate::utils::ShaderError;

/// Validates a WGSL shader source code
/// 
/// Performs multiple validation checks:
/// 1. Uniforms struct matches expected structure
/// 2. Required entry points exist (vs_main, fs_main)
/// 3. WGSL syntax validation via naga
/// 4. Shader logic validation
pub fn validate_shader(wgsl_src: &str) -> Result<(), ShaderError> {
    // 1. Check shader is not empty
    if wgsl_src.trim().is_empty() {
        return Err(ShaderError::ValidationError(
            "Shader source is empty".to_string(),
        ));
    }

    // 2. Validate Uniforms struct
    validate_uniforms_struct(wgsl_src)?;

    // 3. Validate required attributes and entry points
    validate_entry_points(wgsl_src)?;

    // 4. Validate WGSL syntax with naga
    validate_wgsl_syntax(wgsl_src)?;

    Ok(())
}

/// Validate that the Uniforms struct matches our expected structure
fn validate_uniforms_struct(wgsl_src: &str) -> Result<(), ShaderError> {
    // Expected fields in exact order (must match ShaderUniforms in pipeline.rs)
    let expected_fields = [
        "time: f32",
        "audio_bass: f32",
        "audio_mid: f32", 
        "audio_high: f32",
        "resolution: vec2<f32>",
        "_pad0: vec2<f32>",
    ];
    
    // Check if shader defines a Uniforms struct
    if !wgsl_src.contains("struct Uniforms") {
        return Err(ShaderError::ValidationError(
            "Shader must define a 'struct Uniforms' matching the pipeline structure.\n\nExpected:\nstruct Uniforms {\n    time: f32,\n    audio_bass: f32,\n    audio_mid: f32,\n    audio_high: f32,\n    resolution: vec2<f32>,\n    _pad0: vec2<f32>,\n}".to_string()
        ));
    }
    
    // Extract struct definition
    if let Some(start) = wgsl_src.find("struct Uniforms") {
        if let Some(struct_content) = wgsl_src[start..].find('{') {
            let start_brace = start + struct_content;
            if let Some(end_brace) = wgsl_src[start_brace..].find('}') {
                let struct_body = &wgsl_src[start_brace + 1..start_brace + end_brace];
                
                // Check each expected field
                for field in &expected_fields {
                    let field_name = field.split(':').next().unwrap().trim();
                    if !struct_body.contains(field) {
                        return Err(ShaderError::ValidationError(format!(
                            "Uniforms struct mismatch!\n\nMissing or incorrect field: {}\n\nExpected struct (32 bytes total):\n\nstruct Uniforms {{\n    time: f32,\n    audio_bass: f32,\n    audio_mid: f32,\n    audio_high: f32,\n    resolution: vec2<f32>,\n    _pad0: vec2<f32>,\n}}\n\nYour struct is missing or has the wrong type for field: '{}'",
                            field, field_name
                        )));
                    }
                }
            }
        }
    }
    
    // Check binding declaration
    if !wgsl_src.contains("@group(0) @binding(0)") || !wgsl_src.contains("var<uniform> uniforms: Uniforms") {
        return Err(ShaderError::ValidationError(
            "Missing uniform binding declaration.\n\nRequired:\n@group(0) @binding(0) var<uniform> uniforms: Uniforms;".to_string()
        ));
    }
    
    Ok(())
}

/// Validate required shader entry points and attributes
fn validate_entry_points(wgsl_src: &str) -> Result<(), ShaderError> {
    // Check for @vertex and @fragment attributes
    if !wgsl_src.contains("@vertex") {
        return Err(ShaderError::ValidationError(
            "Shader missing @vertex attribute".to_string(),
        ));
    }

    if !wgsl_src.contains("@fragment") {
        return Err(ShaderError::ValidationError(
            "Shader missing @fragment attribute".to_string(),
        ));
    }

    // Validate vertex entry point
    if !wgsl_src.contains("fn vs_main") {
        return Err(ShaderError::ValidationError(
            "Shader missing vertex entry point 'fn vs_main'.\n\nRequired:\n@vertex\nfn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput".to_string(),
        ));
    }

    // Validate fragment entry point
    if !wgsl_src.contains("fn fs_main") {
        return Err(ShaderError::ValidationError(
            "Shader missing fragment entry point 'fn fs_main'.\n\nRequired:\n@fragment\nfn fs_main(@location(0) tex_coords: vec2<f32>) -> @location(0) vec4<f32>".to_string(),
        ));
    }

    // Check for VertexOutput struct (commonly needed)
    if !wgsl_src.contains("struct VertexOutput") {
        return Err(ShaderError::ValidationError(
            "Shader missing 'struct VertexOutput'.\n\nRequired:\nstruct VertexOutput {\n    @builtin(position) position: vec4<f32>,\n    @location(0) tex_coords: vec2<f32>,\n}".to_string(),
        ));
    }

    Ok(())
}

/// Validate WGSL syntax using naga parser
fn validate_wgsl_syntax(wgsl_src: &str) -> Result<(), ShaderError> {
    log::debug!("Validating WGSL with naga parser");
    
    // Parse WGSL
    let module = match naga::front::wgsl::parse_str(wgsl_src) {
        Ok(module) => {
            log::debug!("Naga parse successful");
            module
        }
        Err(parse_error) => {
            let error_msg = format!("WGSL Parse Error:\n{}", parse_error.emit_to_string(wgsl_src));
            log::error!("Shader parse failed: {}", error_msg);
            return Err(ShaderError::ValidationError(error_msg));
        }
    };

    // Validate the module
    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );

    if let Err(validation_error) = validator.validate(&module) {
        let error_msg = format!("WGSL Validation Error:\n{}", validation_error.emit_to_string(wgsl_src));
        log::error!("Shader validation failed: {}", error_msg);
        return Err(ShaderError::ValidationError(error_msg));
    }

    log::debug!("Naga validation passed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_empty_shader() {
        let result = validate_shader("");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_missing_uniforms() {
        let shader = r#"
            @vertex
            fn vs_main() -> @builtin(position) vec4<f32> {
                return vec4<f32>(0.0, 0.0, 0.0, 1.0);
            }
            @fragment
            fn fs_main() -> @location(0) vec4<f32> {
                return vec4<f32>(1.0, 0.0, 0.0, 1.0);
            }
        "#;
        let result = validate_shader(shader);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_missing_entry_points() {
        let shader = r#"
            struct Uniforms {
                time: f32,
                audio_bass: f32,
                audio_mid: f32,
                audio_high: f32,
                resolution: vec2<f32>,
                _pad0: vec2<f32>,
            }
            @group(0) @binding(0) var<uniform> uniforms: Uniforms;
        "#;
        let result = validate_shader(shader);
        assert!(result.is_err());
    }
}
