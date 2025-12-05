use std::sync::Arc;
use std::time::Instant;

use crate::utils::ShaderError;
use eframe::epaint;
use eframe::wgpu::{BindGroup, Buffer, Device, RenderPipeline};

// Shader uniforms structure
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShaderUniforms {
    pub time: f32,
    pub audio_bass: f32,
    pub audio_mid: f32,
    pub audio_high: f32,
    pub resolution: [f32; 2],
    pub _pad0: [f32; 2],
}

// Shader pipeline wrapper
pub struct ShaderPipeline {
    pub pipeline: RenderPipeline,
    pub uniform_buffer: Buffer,
    pub bind_group: BindGroup,
    pub start_time: Instant,
}

impl ShaderPipeline {
    /// Validate that the Uniforms struct in the shader matches our expected structure
    fn validate_uniforms_struct(wgsl_src: &str) -> Result<(), String> {
        // Expected fields in exact order
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
            return Err("Shader must define a 'struct Uniforms' matching the pipeline structure.\n\nExpected:\nstruct Uniforms {\n    time: f32,\n    audio_bass: f32,\n    audio_mid: f32,\n    audio_high: f32,\n    resolution: vec2<f32>,\n    _pad0: vec2<f32>,\n}".to_string());
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
                            return Err(format!(
                                "Uniforms struct mismatch!\n\nMissing or incorrect field: {}\n\nExpected struct (32 bytes total):\nstruct Uniforms {{\n    time: f32,\n    audio_bass: f32,\n    audio_mid: f32,\n    audio_high: f32,\n    resolution: vec2<f32>,\n    _pad0: vec2<f32>,\n}}\n\nYour struct is missing or has wrong type for: {}",
                                field, field_name
                            ));
                        }
                    }
                }
            }
        }
        
        Ok(())
    }

    pub fn new(
        device: &Device,
        format: egui_wgpu::wgpu::TextureFormat,
        wgsl_src: &str,
    ) -> Result<Self, ShaderError> {
        log::debug!("Creating shader pipeline ({} bytes)", wgsl_src.len());

        // Validate shader source is not empty
        if wgsl_src.trim().is_empty() {
            return Err(ShaderError::ValidationError(
                "Shader source is empty".to_string(),
            ));
        }

        // Validate Uniforms struct BEFORE any other validation
        if let Err(err_msg) = Self::validate_uniforms_struct(wgsl_src) {
            return Err(ShaderError::ValidationError(err_msg));
        }

        // Basic WGSL validation - check for common syntax requirements
        let has_vertex = wgsl_src.contains("@vertex");
        let has_fragment = wgsl_src.contains("@fragment");

        if !has_vertex {
            return Err(ShaderError::ValidationError(
                "Shader missing @vertex function".to_string(),
            ));
        }

        if !has_fragment {
            return Err(ShaderError::ValidationError(
                "Shader missing @fragment function".to_string(),
            ));
        }

        // Validate WGSL syntax using naga BEFORE passing to wgpu
        log::debug!("Validating WGSL with naga parser");
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

        log::debug!("Naga validation passed, creating WGPU shader module");

        let shader = device.create_shader_module(egui_wgpu::wgpu::ShaderModuleDescriptor {
            label: Some("dynamic_shader"),
            source: egui_wgpu::wgpu::ShaderSource::Wgsl(wgsl_src.into()),
        });

        let uniform_size = std::mem::size_of::<ShaderUniforms>() as u64;
        log::trace!("Creating uniform buffer ({} bytes)", uniform_size);
        let uniform_buffer = device.create_buffer(&egui_wgpu::wgpu::BufferDescriptor {
            label: Some("shader_uniforms"),
            size: uniform_size,
            usage: egui_wgpu::wgpu::BufferUsages::COPY_DST | egui_wgpu::wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        let uniform_bgl =
            device.create_bind_group_layout(&egui_wgpu::wgpu::BindGroupLayoutDescriptor {
                label: Some("shader_bgl"),
                entries: &[egui_wgpu::wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: egui_wgpu::wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: egui_wgpu::wgpu::BindingType::Buffer {
                        ty: egui_wgpu::wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let bind_group = device.create_bind_group(&egui_wgpu::wgpu::BindGroupDescriptor {
            label: Some("shader_bg"),
            layout: &uniform_bgl,
            entries: &[egui_wgpu::wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout =
            device.create_pipeline_layout(&egui_wgpu::wgpu::PipelineLayoutDescriptor {
                label: Some("shader_pipeline_layout"),
                bind_group_layouts: &[&uniform_bgl],
                push_constant_ranges: &[],
            });

        let pipeline = device.create_render_pipeline(&egui_wgpu::wgpu::RenderPipelineDescriptor {
            label: Some("shader_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: egui_wgpu::wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: egui_wgpu::wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: Some(egui_wgpu::wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: egui_wgpu::wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(egui_wgpu::wgpu::ColorTargetState {
                    format,
                    blend: Some(egui_wgpu::wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: egui_wgpu::wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: egui_wgpu::wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: egui_wgpu::wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        log::info!("Shader pipeline created successfully (format: {:?})", format);

        Ok(Self {
            pipeline,
            uniform_buffer,
            bind_group,
            start_time: Instant::now(),
        })
    }
}

// Callback for rendering shader
pub struct ShaderCallback {
    pub shader: Arc<ShaderPipeline>,
    pub bass_energy: Arc<std::sync::Mutex<f32>>,
    pub mid_energy: Arc<std::sync::Mutex<f32>>,
    pub high_energy: Arc<std::sync::Mutex<f32>>,
}

impl egui_wgpu::CallbackTrait for ShaderCallback {
    fn prepare(
        &self,
        _device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _encoder: &mut eframe::wgpu::CommandEncoder,
        _resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<eframe::wgpu::CommandBuffer> {
        let elapsed = self.shader.start_time.elapsed().as_secs_f32();

        let resolution = [
            screen_descriptor.size_in_pixels[0] as f32,
            screen_descriptor.size_in_pixels[1] as f32,
        ];

        let bass = *self.bass_energy.lock().unwrap();
        let mid = *self.mid_energy.lock().unwrap();
        let high = *self.high_energy.lock().unwrap();

        let uniforms = ShaderUniforms {
            time: elapsed,
            audio_bass: bass,
            audio_mid: mid,
            audio_high: high,
            resolution,
            _pad0: [0.0, 0.0],
        };

        queue.write_buffer(
            &self.shader.uniform_buffer,
            0,
            bytemuck::bytes_of(&uniforms),
        );
        Vec::new()
    }

    fn paint(
        &self,
        _info: epaint::PaintCallbackInfo,
        render_pass: &mut eframe::wgpu::RenderPass<'static>,
        _resources: &egui_wgpu::CallbackResources,
    ) {
        render_pass.set_pipeline(&self.shader.pipeline);
        render_pass.set_bind_group(0, &self.shader.bind_group, &[]);
        render_pass.draw(0..6, 0..1); // Draw 6 vertices (2 triangles)

        // Log first render only
        static FIRST_RENDER: std::sync::Once = std::sync::Once::new();
        FIRST_RENDER.call_once(|| {
            log::debug!("First shader render executed");
        });
    }
}
