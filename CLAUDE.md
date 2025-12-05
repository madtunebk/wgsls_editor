# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a ShaderToy-like application built with Rust, egui, and WGPU. It provides a live WGSL shader editor with real-time preview, audio-reactive capabilities, and a custom UI. The application features a split-pane interface with a code editor on the left and a live shader preview on the right.

## Build & Run Commands

```bash
# Build (debug)
cargo build

# Build (release)
cargo build --release

# Run with logging
RUST_LOG=info cargo run

# Run with debug backtrace
RUST_BACKTRACE=1 cargo run

# Test
cargo test

# Run specific test
cargo test <module>::<test_name>

# Debug tests
RUST_BACKTRACE=1 cargo test -- --nocapture

# Format code
cargo fmt

# Lint (strict mode)
cargo clippy --all-targets -- -D warnings
```

## Architecture

### Module Structure

The codebase follows a fully modular organization with **zero code duplication**:

- **`src/main.rs`**: Entry point. Sets up the eframe window, detects monitor size, registers fonts, applies theme, and instantiates `TopApp`.
- **`src/screens/editor.rs`**: The main `TopApp` struct implementing `eframe::App`. Uses a unified `HashMap<BufferKind, ShaderBuffer>` to manage all buffers (MainImage + Buffer A-D). No duplicate code across tabs.
- **`src/screens/shader_buffer.rs`**: **NEW** - Unified `ShaderBuffer` struct that represents any buffer type (MainImage or Buffer A-D). Replaces 5 duplicate tab implementations with a single generic structure.
- **`src/ui_components/shader_editor.rs`**: **NEW** - Shared shader editor rendering function used by all buffers. Single implementation of the WGSL code editor with syntax highlighting.
- **`src/utils/`**: Core utilities shared across the app:
  - `multi_buffer_pipeline.rs`: `MultiPassPipelines` for multi-pass rendering. Manages buffer passes, textures, and render pipeline coordination.
  - `shader_validator.rs`: WGSL validation using naga before compilation. Validates uniforms struct, entry points, and syntax.
  - `shader_constants.rs`: **NEW** - Centralized constants for all shader-related code (boilerplate, vertex shader, default shaders, font sizes). Single source of truth.
  - `pipeline.rs`: Legacy `ShaderPipeline` (deprecated) and `ShaderUniforms` struct definition.
  - `audio.rs`: `AudioState` for 3-band audio (bass/mid/high). Includes FFT-based input capture.
  - `audio_file.rs`: Audio playback from files (MP3) using rodio.
  - `audio_analyzer.rs`: Real-time FFT analysis for audio visualization. Processes audio samples and extracts frequency band energies.
  - `errors.rs`: `ShaderError` enum and formatting utilities.
  - `theme.rs`: Dark theme configuration for egui.
  - `fonts.rs`: Custom font registration for shader error display.
  - `monitors.rs`: Monitor detection via xrandr on Linux.
  - `toast.rs`: Toast notification system.
  - `wgsl_syntax.rs`: WGSL syntax highlighting for code editor.
- **`src/ui_components/`**: Reusable UI components:
  - `settings_menu.rs`: Settings overlay
  - `shader_properties.rs`: Shader presets and audio file picker
  - `shader_editor.rs`: **NEW** - Shared WGSL code editor component
- **`src/assets/shards/`**: Preset shader templates (psychedelic.frag, tunnel.frag, raymarch.frag, fractal.frag, etc.).

### Modular Architecture Principles

The codebase has been refactored to follow strict modularity principles:

1. **No Code Duplication**: All duplicate code has been eliminated. The previous 5 identical tab files (MainImageTab, BufferATab-DTab) have been replaced with a single `ShaderBuffer` struct.

2. **Single Source of Truth**: Constants are defined once in `shader_constants.rs`. Shader editor rendering is implemented once in `shader_editor.rs`.

3. **Data-Driven Design**: Instead of separate fields for each tab (`main_image_tab`, `buffer_a_tab`, etc.), `TopApp` uses a single `HashMap<BufferKind, ShaderBuffer>` that can scale to any number of buffers.

4. **Shared Components**: UI components like the shader editor are extracted into reusable functions that all buffers share.

### Shader Compilation Flow

1. User edits shader code in the editor (MainImage/Buffer A-D tabs).
2. On "Apply Pipeline" or Ctrl+Enter, `shader_needs_update` flag is set.
3. In the next `update()` frame, all buffer shaders are collected and auto-injection is applied:
   - Boilerplate (Uniforms struct, VSOut struct, texture bindings) is prepended
   - Standard vertex shader is prepended if not provided
   - Each shader is validated with **naga** first
4. If naga validation passes for all shaders, WGPU creates a `MultiPassPipelines` with all buffer passes.
5. On success: pipeline is stored, toast shows success. On error: native egui error window displays formatted error.
6. The shader preview renders using `MultiPassCallback`, passing uniforms (time, audio bands, resolution) and rendering buffers in order: BufferA → BufferB → BufferC → BufferD → MainImage.

### Key Data Flow

- **Multi-Pass Pipeline**: Stored in `Arc<Mutex<Option<Arc<MultiPassPipelines>>>>` for thread-safe access. Contains all buffer passes and textures.
- **Buffer System**: Each buffer (A-D) renders to an offscreen texture. MainImage can sample from all buffer textures via texture bindings (group 1).
- **Audio State**: Three separate `Arc<Mutex<f32>>` values for bass, mid, and high frequency bands. Updated by audio thread (via `AudioAnalyzer`), read by shader callback.
- **Error Handling**: `Arc<Mutex<Option<ShaderError>>>` stores last error. Formatted and displayed in native egui window.
- **Uniforms**: `ShaderUniforms` struct (time, audio_bass, audio_mid, audio_high, resolution, _pad0) passed to shader each frame via uniform buffer.

### Auto-Injection System

The application uses an **auto-injection** system to reduce boilerplate. When a user writes a shader, code is automatically prepended based on the buffer type:

**Boilerplate (injected into ALL buffers):**
```wgsl
struct Uniforms {
    time: f32,
    audio_bass: f32,
    audio_mid: f32,
    audio_high: f32,
    resolution: vec2<f32>,
    _pad0: vec2<f32>,
}
@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VSOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
}
```

**Multi-Pass Texture Bindings (ONLY injected into MainImage):**
```wgsl
@group(1) @binding(0) var buffer_a_texture: texture_2d<f32>;
@group(1) @binding(1) var buffer_a_sampler: sampler;
@group(1) @binding(2) var buffer_b_texture: texture_2d<f32>;
@group(1) @binding(3) var buffer_b_sampler: sampler;
@group(1) @binding(4) var buffer_c_texture: texture_2d<f32>;
@group(1) @binding(5) var buffer_c_sampler: sampler;
@group(1) @binding(6) var buffer_d_texture: texture_2d<f32>;
@group(1) @binding(7) var buffer_d_sampler: sampler;
```

**IMPORTANT**:
- Buffer A-D shaders do NOT get texture bindings (they render independently)
- Only MainImage can sample from buffers A-D
- Each buffer has its own sampler (buffer_a_sampler, buffer_b_sampler, etc.)

**Standard Vertex Shader (STANDARD_VERTEX in editor.rs:39-49):**
```wgsl
@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VSOut {
    // Full-screen triangle implementation
}
```

Users only need to write the `@fragment fn fs_main()` function. The editor automatically combines boilerplate + vertex shader + user fragment shader before compilation.

### WGSL Shader Requirements

After auto-injection, shaders MUST include:
- A `@vertex` function `vs_main` (auto-injected if not provided)
- A `@fragment` function `fs_main` that returns a color
- Uniforms struct matching the exact layout (auto-injected)
- Entry points: `vs_main` for vertex, `fs_main` for fragment

Example minimal user shader:
```wgsl
@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    return vec4(in.uv.x, in.uv.y, sin(uniforms.time), 1.0);
}
```

### Multi-Pass Rendering

The application supports 5 render passes:
- **Buffer A, B, C, D**: Offscreen textures (intermediate passes)
- **MainImage**: Final output to screen

Render order: BufferA → BufferB → BufferC → BufferD → MainImage

MainImage can sample from any buffer texture:
```wgsl
@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    let buffer_a = textureSample(buffer_a_texture, texture_sampler, in.uv);
    let buffer_b = textureSample(buffer_b_texture, texture_sampler, in.uv + vec2(0.01));
    return mix(buffer_a, buffer_b, 0.5);
}
```

Buffer passes can also sample from other buffers if needed (texture bindings are available to all passes).

## Common Patterns

### Adding New Shader Uniforms

1. Update `ShaderUniforms` struct in `src/utils/pipeline.rs`.
2. Ensure struct is `#[repr(C)]`, `Pod`, and `Zeroable` (for bytemuck).
3. Update `SHADER_BOILERPLATE` in `src/screens/editor.rs` to match new struct.
4. Update `validate_uniforms_struct()` in `src/utils/shader_validator.rs` to validate new fields.
5. Update `MultiPassCallback::prepare()` in `src/utils/multi_buffer_pipeline.rs` to populate new uniforms.
6. Update default shaders (`src/assets/shards/`) to demonstrate new uniforms.

### Handling Shader Errors

Errors are captured in `ShaderError` enum (see `src/utils/errors.rs`). Use `format_shader_error()` to format for display. Naga validation errors include line numbers and context. Displayed in native egui window (`show_error_window` in `TopApp`).

### Audio Integration

Audio state is shared via three separate `Arc<Mutex<f32>>` values (bass_energy, mid_energy, high_energy). Two modes:
1. **Input capture** (FFT from mic): `start_input_fft()` in `audio.rs`.
2. **File playback** (MP3): `start_file_audio()` in `audio_file.rs` (currently used).

Both modes use `AudioAnalyzer` for FFT processing. The analyzer extracts frequency band energies:
- Bass: 20-250 Hz
- Mid: 250-4000 Hz
- High: 4000-20000 Hz

Tuning constants in `audio_analyzer.rs:6-20` control normalization and smoothing. Shader receives 3 bands as uniforms, updated each frame.

### UI Theming

Dark theme applied in `apply_editor_theme()` every frame to prevent drift. Custom colors in `src/utils/theme.rs`. Editor font size adjustable via settings or keyboard shortcuts (Ctrl+Plus/Minus/0).

## Keyboard Shortcuts

- **Ctrl/Cmd + Enter**: Apply shader (compile pipeline)
- **Ctrl/Cmd + Plus**: Increase font size
- **Ctrl/Cmd + Minus**: Decrease font size
- **Ctrl/Cmd + 0**: Reset font size to default
- **Ctrl/Cmd + ,**: Open settings menu

## Debugging

- Enable logging: `RUST_LOG=debug cargo run` or `RUST_LOG=info cargo run`.
- Check shader compilation: Look for `[MultiPassPipelines]` logs.
- Audio debugging: Enable "Debug Audio" in settings overlay to manually control band levels.
- WGPU validation: Errors logged to stderr. Check for naga parse/validation errors first.
- Naga validation: Shader validation happens in `validate_shader()` before WGPU compilation. Check `shader_validator.rs` for validation logic.

## Code Conventions (from AGENTS.md)

- Edition: Rust 2021, rustfmt defaults (4-space indent).
- Imports: std, external crates, local modules. No glob imports.
- Naming: `snake_case` for functions/variables, `CamelCase` for types, `SCREAMING_SNAKE_CASE` for constants.
- Error handling: Return `Result<T, E>`, propagate with `?`. Avoid `unwrap`/`expect` except in tests or early init.
- Mutability: Prefer immutable bindings.
- Tests: Keep unit tests small, avoid GPU/windowing (mock or extract logic).

## Important Implementation Notes

- The application expects preset shaders in `src/assets/shards/` (e.g., default.frag, psychedelic.frag).
- Audio file path can be set via settings UI (file picker) or defaults to `src/assets/test.mp3`.
- Window sizing: Defaults to 75% of primary monitor size (detected via xrandr), centered. Falls back to 1440x810 if monitor detection fails.
- Shader compilation happens on the main thread during `update()`, not async. Keep shader compilation fast.
- The `MultiPassCallback` trait renders the shader via WGPU callback, bypassing egui's normal rendering.
- All buffer shaders are compiled together as a single pipeline. If any buffer fails validation, the entire pipeline compilation fails.
- The auto-injection system prepends boilerplate before validation. Users never see or write the boilerplate code.
- Font size is synchronized across all buffer tabs. Changing font size in one tab updates all tabs.
