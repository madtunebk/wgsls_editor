use eframe::egui_wgpu::wgpu::{Device, Queue, TextureFormat};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::screens::shader_buffer::ShaderBuffer;
use crate::utils::{
    catch_panic_mut, format_panic_message, format_shader_error, validate_shader, BufferKind,
    MultiPassPipelines, ShaderError, DEFAULT_BUFFER_RESOLUTION, DEFAULT_FRAGMENT,
    DEFAULT_VERTEX, SHADER_BOILERPLATE, STANDARD_VERTEX, TEXTURE_BINDINGS,
};

/// Handles shader compilation and pipeline creation
pub struct ShaderCompiler {
    /// Compiled shader pipeline (shared with rendering)
    pipeline: Arc<Mutex<Option<Arc<MultiPassPipelines>>>>,

    /// Last compilation error
    last_error: Arc<Mutex<Option<ShaderError>>>,

    /// Flag to trigger recompilation
    needs_update: Arc<AtomicBool>,
}

impl ShaderCompiler {
    /// Create a new shader compiler
    pub fn new() -> Self {
        Self {
            pipeline: Arc::new(Mutex::new(None)),
            last_error: Arc::new(Mutex::new(None)),
            needs_update: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Get shared reference to the pipeline
    pub fn pipeline(&self) -> Arc<Mutex<Option<Arc<MultiPassPipelines>>>> {
        self.pipeline.clone()
    }

    /// Trigger shader recompilation
    pub fn trigger_compilation(&self) {
        self.needs_update.store(true, Ordering::Relaxed);
        *self.last_error.lock().unwrap() = None;
    }

    /// Compile shaders if update is pending
    /// Returns Ok(true) if compiled, Ok(false) if no update needed, Err on failure
    pub fn compile_if_needed(
        &self,
        buffers: &HashMap<BufferKind, ShaderBuffer>,
        image_paths: &[Option<String>; 4],
        device: &Device,
        queue: &Queue,
        format: TextureFormat,
    ) -> Result<bool, CompilationResult> {
        if !self.needs_update.load(Ordering::Relaxed) {
            return Ok(false);
        }

        log::debug!("Shader update requested, beginning multi-pass compilation");
        self.needs_update.store(false, Ordering::Relaxed);

        // Gather shader sources
        let sources = match self.gather_sources(buffers) {
            Ok(sources) => sources,
            Err(err) => {
                *self.last_error.lock().unwrap() = Some(err.clone());
                return Err(CompilationResult::ValidationError(err));
            }
        };

        log::debug!("[ShaderCompiler] Compiling multi-pass pipeline with {} buffers", sources.len());

        // Compile pipeline (with panic catching)
        let result = catch_panic_mut(|| {
            MultiPassPipelines::new(device, queue, format, DEFAULT_BUFFER_RESOLUTION, &sources, image_paths)
        });

        match result {
            Ok(Ok(pipeline)) => {
                // Success
                *self.pipeline.lock().unwrap() = Some(Arc::new(pipeline));
                *self.last_error.lock().unwrap() = None;
                log::info!("[ShaderCompiler] Multi-pass shader compiled successfully");
                Ok(true)
            }
            Ok(Err(err)) => {
                // Shader compilation error
                *self.last_error.lock().unwrap() = Some(err.clone());
                log::error!("[ShaderCompiler] Shader compilation failed: {}", format_shader_error(&err));
                Err(CompilationResult::CompilationError(err))
            }
            Err(panic_msg) => {
                // Caught panic
                let formatted = format_panic_message(&panic_msg);
                let error = ShaderError::CompilationError(formatted.clone());
                *self.last_error.lock().unwrap() = Some(error.clone());
                log::error!("[ShaderCompiler] Pipeline creation panicked: {}", panic_msg);
                Err(CompilationResult::Panic(error))
            }
        }
    }

    /// Gather shader sources from buffers and apply boilerplate injection
    fn gather_sources(
        &self,
        buffers: &HashMap<BufferKind, ShaderBuffer>,
    ) -> Result<HashMap<BufferKind, String>, ShaderError> {
        let mut sources = HashMap::with_capacity(5);

        for buffer_kind in [
            BufferKind::MainImage,
            BufferKind::BufferA,
            BufferKind::BufferB,
            BufferKind::BufferC,
            BufferKind::BufferD,
        ] {
            let (vertex, fragment) = buffers
                .get(&buffer_kind)
                .map(|b| b.get_shaders())
                .unwrap_or(("", ""));

            let fragment_trimmed = fragment.trim();

            // Skip empty fragments (except MainImage which needs default)
            let has_code = fragment_trimmed.lines().any(|line| {
                let trimmed_line = line.trim();
                !trimmed_line.is_empty() && !trimmed_line.starts_with("//")
            });

            if fragment_trimmed.is_empty() || !has_code {
                if buffer_kind == BufferKind::MainImage {
                    // MainImage must exist, use default
                    sources.insert(
                        buffer_kind,
                        format!("{}\n{}\n{}", SHADER_BOILERPLATE, STANDARD_VERTEX, DEFAULT_FRAGMENT),
                    );
                }
                continue;
            }

            // Auto-inject boilerplate + standard vertex unless user provides custom vertex
            let vertex_trimmed = vertex.trim();
            let user_vertex =
                if vertex_trimmed.is_empty() || vertex_trimmed == DEFAULT_VERTEX.trim() {
                    STANDARD_VERTEX
                } else {
                    vertex_trimmed
                };

            // Build complete shader with conditional texture bindings
            // MainImage gets texture bindings to sample from buffers A-D
            // Buffer A-D do NOT get texture bindings (they're independent)
            let needs_textures = buffer_kind == BufferKind::MainImage;

            let mut complete_shader = String::with_capacity(
                SHADER_BOILERPLATE.len() + user_vertex.len() + fragment_trimmed.len() + 200,
            );
            complete_shader.push_str(SHADER_BOILERPLATE);

            // Add texture bindings ONLY for MainImage
            if needs_textures {
                complete_shader.push_str(TEXTURE_BINDINGS);
            }

            complete_shader.push('\n');
            complete_shader.push_str(user_vertex);
            complete_shader.push('\n');
            complete_shader.push_str(fragment_trimmed);

            // Validate the complete shader
            if let Err(e) = validate_shader(&complete_shader) {
                return Err(ShaderError::ValidationError(format!(
                    "[{:?}] {}",
                    buffer_kind, e
                )));
            }

            sources.insert(buffer_kind, complete_shader);
        }

        // Ensure MainImage exists
        sources.entry(BufferKind::MainImage).or_insert_with(|| {
            format!(
                "{}\n{}\n{}",
                SHADER_BOILERPLATE, STANDARD_VERTEX, DEFAULT_FRAGMENT
            )
        });

        Ok(sources)
    }
}

/// Result of shader compilation
#[derive(Debug, Clone)]
pub enum CompilationResult {
    /// Validation error (before compilation)
    ValidationError(ShaderError),

    /// Compilation error (during WGPU pipeline creation)
    CompilationError(ShaderError),

    /// Panic during compilation
    Panic(ShaderError),
}

impl CompilationResult {
    /// Get the underlying ShaderError
    pub fn error(&self) -> &ShaderError {
        match self {
            CompilationResult::ValidationError(e) => e,
            CompilationResult::CompilationError(e) => e,
            CompilationResult::Panic(e) => e,
        }
    }
}
