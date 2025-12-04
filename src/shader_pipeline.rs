use std::sync::Arc;
use std::time::Instant;

use eframe::epaint;
use eframe::wgpu::{Device, RenderPipeline, Buffer, BindGroup};

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
    pub fn new(device: &Device, format: egui_wgpu::wgpu::TextureFormat, wgsl_src: &str) -> Result<Self, String> {
        log::debug!("[ShaderPipeline] Creating shader module from WGSL source ({} bytes)", wgsl_src.len());
        let shader = device.create_shader_module(egui_wgpu::wgpu::ShaderModuleDescriptor {
            label: Some("dynamic_shader"),
            source: egui_wgpu::wgpu::ShaderSource::Wgsl(wgsl_src.into()),
        });

        log::debug!("[ShaderPipeline] Shader module created successfully");

        let uniform_size = std::mem::size_of::<ShaderUniforms>() as u64;
        log::debug!("[ShaderPipeline] Creating uniform buffer (size: {} bytes)", uniform_size);
        let uniform_buffer = device.create_buffer(&egui_wgpu::wgpu::BufferDescriptor {
            label: Some("shader_uniforms"),
            size: uniform_size,
            usage: egui_wgpu::wgpu::BufferUsages::COPY_DST | egui_wgpu::wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        let uniform_bgl = device.create_bind_group_layout(&egui_wgpu::wgpu::BindGroupLayoutDescriptor {
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

        log::info!("[ShaderPipeline] Pipeline created successfully with format: {:?}", format);

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

        let uniforms = ShaderUniforms {
            time: elapsed,
            audio_bass: 0.0,
            audio_mid: 0.0,
            audio_high: 0.0,
            resolution,
            _pad0: [0.0, 0.0],
        };

        queue.write_buffer(&self.shader.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
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
        render_pass.draw(0..6, 0..1);  // Draw 6 vertices (2 triangles)

        // Log first render only
        static FIRST_RENDER: std::sync::Once = std::sync::Once::new();
        FIRST_RENDER.call_once(|| {
            log::info!("[ShaderCallback] First render executed - drawing 6 vertices");
        });
    }
}
