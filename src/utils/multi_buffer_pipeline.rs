use std::sync::Arc;
use std::time::Instant;

use crate::utils::{validate_shader, ShaderError};
use eframe::epaint;
use eframe::wgpu::{
    AddressMode, BindGroup, BindGroupLayout, Buffer, CommandEncoder, Device, Extent3d,
    FilterMode, Queue, RenderPipeline, Sampler, SamplerDescriptor, Texture, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor,
};

// Re-export ShaderUniforms from pipeline module
pub use crate::utils::pipeline::ShaderUniforms;

/// Buffer types for multi-pass rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BufferKind {
    MainImage,
    BufferA,
    BufferB,
    BufferC,
    BufferD,
}

impl BufferKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            BufferKind::MainImage => "MainImage",
            BufferKind::BufferA => "BufferA",
            BufferKind::BufferB => "BufferB",
            BufferKind::BufferC => "BufferC",
            BufferKind::BufferD => "BufferD",
        }
    }
}

/// Single render pass that renders into an offscreen texture
pub struct BufferPass {
    pub kind: BufferKind,
    pub pipeline: RenderPipeline,
    pub target_texture: Texture,
    pub target_view: TextureView,
}

impl BufferPass {
    fn render(&self, encoder: &mut CommandEncoder, uniform_bind_group: &BindGroup) {
        let mut rpass = encoder.begin_render_pass(&eframe::wgpu::RenderPassDescriptor {
            label: Some(&format!("{}_pass", self.kind.as_str())),
            color_attachments: &[Some(eframe::wgpu::RenderPassColorAttachment {
                view: &self.target_view,
                resolve_target: None,
                ops: eframe::wgpu::Operations {
                    load: eframe::wgpu::LoadOp::Clear(eframe::wgpu::Color::BLACK),
                    store: eframe::wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, uniform_bind_group, &[]);
        rpass.draw(0..6, 0..1);
    }
}

/// Helper: create an offscreen texture for a buffer
fn create_color_target(
    device: &Device,
    size: [u32; 2],
    format: TextureFormat,
    label: &str,
) -> (Texture, TextureView) {
    let texture = device.create_texture(&TextureDescriptor {
        label: Some(label),
        size: Extent3d {
            width: size[0],
            height: size[1],
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });

    let view = texture.create_view(&TextureViewDescriptor::default());
    (texture, view)
}

/// Multi-pass shader pipeline manager
pub struct MultiPassPipelines {
    pub uniform_buffer: Buffer,
    pub uniform_bind_group_layout: BindGroupLayout,
    pub texture_bind_group_layout: BindGroupLayout,
    pub uniform_bind_group: BindGroup,

    // Optional buffer passes (only created if shader code provided)
    pub buffer_a: Option<BufferPass>,
    pub buffer_b: Option<BufferPass>,
    pub buffer_c: Option<BufferPass>,
    pub buffer_d: Option<BufferPass>,

    // Main image pipeline (always present)
    pub main_image_pipeline: RenderPipeline,
    pub main_texture_bind_group: BindGroup,

    pub sampler: Sampler,
    pub start_time: Instant,
}

impl MultiPassPipelines {
    pub fn new(
        device: &Device,
        format: TextureFormat,
        screen_size: [u32; 2],
        sources: &std::collections::HashMap<BufferKind, String>,
    ) -> Result<Self, ShaderError> {
        log::info!(
            "Creating multi-pass shader pipeline (resolution: {}x{})",
            screen_size[0],
            screen_size[1]
        );

        // ===== Uniform buffer =====
        let uniform_size = std::mem::size_of::<ShaderUniforms>() as u64;
        let uniform_buffer = device.create_buffer(&eframe::wgpu::BufferDescriptor {
            label: Some("shader_uniforms"),
            size: uniform_size,
            usage: eframe::wgpu::BufferUsages::COPY_DST | eframe::wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        // ===== Bind group layout: uniforms @group(0) =====
        let uniform_bgl = device.create_bind_group_layout(&eframe::wgpu::BindGroupLayoutDescriptor {
            label: Some("uniform_bgl"),
            entries: &[eframe::wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: eframe::wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: eframe::wgpu::BindingType::Buffer {
                    ty: eframe::wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let uniform_bg = device.create_bind_group(&eframe::wgpu::BindGroupDescriptor {
            label: Some("uniform_bg"),
            layout: &uniform_bgl,
            entries: &[eframe::wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // ===== Bind group layout: textures @group(1) =====
        let texture_bgl = device.create_bind_group_layout(&eframe::wgpu::BindGroupLayoutDescriptor {
            label: Some("texture_bgl"),
            entries: &[
                // Buffer A texture @binding(0)
                eframe::wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: eframe::wgpu::ShaderStages::FRAGMENT,
                    ty: eframe::wgpu::BindingType::Texture {
                        sample_type: eframe::wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: eframe::wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Sampler @binding(1)
                eframe::wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: eframe::wgpu::ShaderStages::FRAGMENT,
                    ty: eframe::wgpu::BindingType::Sampler(
                        eframe::wgpu::SamplerBindingType::Filtering,
                    ),
                    count: None,
                },
            ],
        });

        // ===== Shared sampler =====
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("buffer_sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        // ===== BUFFER A: offscreen pass (optional) =====
        let buffer_a = if let Some(buffer_a_src) = sources.get(&BufferKind::BufferA) {
            if !buffer_a_src.trim().is_empty() {
                log::debug!("Creating BufferA pass");
                validate_shader(buffer_a_src)?;

                let buffer_a_module = device.create_shader_module(eframe::wgpu::ShaderModuleDescriptor {
                    label: Some("buffer_a_shader"),
                    source: eframe::wgpu::ShaderSource::Wgsl(buffer_a_src.clone().into()),
                });

                let (buffer_a_tex, buffer_a_view) =
                    create_color_target(device, screen_size, format, "buffer_a_target");

                let buffer_a_pipeline_layout =
                    device.create_pipeline_layout(&eframe::wgpu::PipelineLayoutDescriptor {
                        label: Some("buffer_a_pipeline_layout"),
                        bind_group_layouts: &[&uniform_bgl],
                        push_constant_ranges: &[],
                    });

                let buffer_a_pipeline =
                    device.create_render_pipeline(&eframe::wgpu::RenderPipelineDescriptor {
                        label: Some("buffer_a_pipeline"),
                        layout: Some(&buffer_a_pipeline_layout),
                        vertex: eframe::wgpu::VertexState {
                            module: &buffer_a_module,
                            entry_point: Some("vs_main"),
                            compilation_options: eframe::wgpu::PipelineCompilationOptions::default(),
                            buffers: &[],
                        },
                        fragment: Some(eframe::wgpu::FragmentState {
                            module: &buffer_a_module,
                            entry_point: Some("fs_main"),
                            compilation_options: eframe::wgpu::PipelineCompilationOptions::default(),
                            targets: &[Some(eframe::wgpu::ColorTargetState {
                                format,
                                blend: Some(eframe::wgpu::BlendState::ALPHA_BLENDING),
                                write_mask: eframe::wgpu::ColorWrites::ALL,
                            })],
                        }),
                        primitive: eframe::wgpu::PrimitiveState::default(),
                        depth_stencil: None,
                        multisample: eframe::wgpu::MultisampleState::default(),
                        multiview: None,
                        cache: None,
                    });

                Some(BufferPass {
                    kind: BufferKind::BufferA,
                    pipeline: buffer_a_pipeline,
                    target_texture: buffer_a_tex,
                    target_view: buffer_a_view,
                })
            } else {
                None
            }
        } else {
            None
        };

        // ===== MAIN IMAGE: reads BufferA texture =====
        let main_src = sources
            .get(&BufferKind::MainImage)
            .ok_or_else(|| ShaderError::CompilationError("Missing MainImage source".into()))?;

        log::debug!("Creating MainImage pipeline");
        validate_shader(main_src)?;

        let main_module = device.create_shader_module(eframe::wgpu::ShaderModuleDescriptor {
            label: Some("main_image_shader"),
            source: eframe::wgpu::ShaderSource::Wgsl(main_src.clone().into()),
        });

        let main_pipeline_layout =
            device.create_pipeline_layout(&eframe::wgpu::PipelineLayoutDescriptor {
                label: Some("main_pipeline_layout"),
                bind_group_layouts: &[&uniform_bgl, &texture_bgl],
                push_constant_ranges: &[],
            });

        let main_pipeline =
            device.create_render_pipeline(&eframe::wgpu::RenderPipelineDescriptor {
                label: Some("main_image_pipeline"),
                layout: Some(&main_pipeline_layout),
                vertex: eframe::wgpu::VertexState {
                    module: &main_module,
                    entry_point: Some("vs_main"),
                    compilation_options: eframe::wgpu::PipelineCompilationOptions::default(),
                    buffers: &[],
                },
                fragment: Some(eframe::wgpu::FragmentState {
                    module: &main_module,
                    entry_point: Some("fs_main"),
                    compilation_options: eframe::wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(eframe::wgpu::ColorTargetState {
                        format,
                        blend: Some(eframe::wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: eframe::wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: eframe::wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: eframe::wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

        // ===== Bind group for MainImage to read BufferA texture =====
        let main_tex_bg = if let Some(ref buffer_a_pass) = buffer_a {
            device.create_bind_group(&eframe::wgpu::BindGroupDescriptor {
                label: Some("main_texture_bg"),
                layout: &texture_bgl,
                entries: &[
                    eframe::wgpu::BindGroupEntry {
                        binding: 0,
                        resource: eframe::wgpu::BindingResource::TextureView(&buffer_a_pass.target_view),
                    },
                    eframe::wgpu::BindGroupEntry {
                        binding: 1,
                        resource: eframe::wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            })
        } else {
            // If no BufferA, create a dummy 1x1 black texture
            let (dummy_tex, dummy_view) = create_color_target(device, [1, 1], format, "dummy_texture");
            device.create_bind_group(&eframe::wgpu::BindGroupDescriptor {
                label: Some("main_texture_bg_dummy"),
                layout: &texture_bgl,
                entries: &[
                    eframe::wgpu::BindGroupEntry {
                        binding: 0,
                        resource: eframe::wgpu::BindingResource::TextureView(&dummy_view),
                    },
                    eframe::wgpu::BindGroupEntry {
                        binding: 1,
                        resource: eframe::wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            })
        };

        log::info!("Multi-pass shader pipeline created successfully");

        Ok(Self {
            uniform_buffer,
            uniform_bind_group_layout: uniform_bgl,
            texture_bind_group_layout: texture_bgl,
            uniform_bind_group: uniform_bg,
            buffer_a,
            buffer_b: None, // TODO: Implement later
            buffer_c: None,
            buffer_d: None,
            main_image_pipeline: main_pipeline,
            main_texture_bind_group: main_tex_bg,
            sampler,
            start_time: Instant::now(),
        })
    }

    /// Record all render passes: buffers first, then main image
    pub fn record_passes(&self, encoder: &mut CommandEncoder, screen_view: &TextureView) {
        // 1) Buffer A → its own texture
        if let Some(ref buffer_a) = self.buffer_a {
            buffer_a.render(encoder, &self.uniform_bind_group);
        }

        // 2) Buffer B, C, D (when implemented)
        // ...

        // 3) MainImage → screen, sampling from BufferA.target_view
        {
            let mut rpass = encoder.begin_render_pass(&eframe::wgpu::RenderPassDescriptor {
                label: Some("main_image_pass"),
                color_attachments: &[Some(eframe::wgpu::RenderPassColorAttachment {
                    view: screen_view,
                    resolve_target: None,
                    ops: eframe::wgpu::Operations {
                        load: eframe::wgpu::LoadOp::Clear(eframe::wgpu::Color::BLACK),
                        store: eframe::wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            rpass.set_pipeline(&self.main_image_pipeline);
            rpass.set_bind_group(0, &self.uniform_bind_group, &[]);
            rpass.set_bind_group(1, &self.main_texture_bind_group, &[]);
            rpass.draw(0..6, 0..1);
        }
    }

    /// Update uniforms before rendering
    pub fn update_uniforms(&self, queue: &Queue, uniforms: &ShaderUniforms) {
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(uniforms));
    }
}

/// Callback for rendering multi-pass shader
pub struct MultiPassCallback {
    pub shader: Arc<MultiPassPipelines>,
    pub bass_energy: Arc<std::sync::Mutex<f32>>,
    pub mid_energy: Arc<std::sync::Mutex<f32>>,
    pub high_energy: Arc<std::sync::Mutex<f32>>,
}

impl eframe::egui_wgpu::CallbackTrait for MultiPassCallback {
    fn prepare(
        &self,
        _device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        screen_descriptor: &eframe::egui_wgpu::ScreenDescriptor,
        encoder: &mut eframe::wgpu::CommandEncoder,
        _resources: &mut eframe::egui_wgpu::CallbackResources,
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

        self.shader.update_uniforms(queue, &uniforms);

        // Render buffer passes to offscreen textures
        if let Some(ref buffer_a) = self.shader.buffer_a {
            buffer_a.render(encoder, &self.shader.uniform_bind_group);
        }

        Vec::new()
    }

    fn paint(
        &self,
        _info: epaint::PaintCallbackInfo,
        render_pass: &mut eframe::wgpu::RenderPass<'static>,
        _resources: &eframe::egui_wgpu::CallbackResources,
    ) {
        // Render main image (which samples from buffer textures)
        render_pass.set_pipeline(&self.shader.main_image_pipeline);
        render_pass.set_bind_group(0, &self.shader.uniform_bind_group, &[]);
        render_pass.set_bind_group(1, &self.shader.main_texture_bind_group, &[]);
        render_pass.draw(0..6, 0..1);

        static FIRST_RENDER: std::sync::Once = std::sync::Once::new();
        FIRST_RENDER.call_once(|| {
            log::debug!("First multi-pass shader render executed");
        });
    }
}
