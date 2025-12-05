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

The codebase follows a modular organization:

- **`src/main.rs`**: Entry point. Sets up the eframe window, detects monitor size, registers fonts, applies theme, and instantiates `TopApp`.
- **`src/screens/editor.rs`**: The main `TopApp` struct implementing `eframe::App`. Handles the entire UI: editor panel (left), shader preview (right), settings overlay, audio overlay, error windows, and toast notifications.
- **`src/utils/`**: Core utilities shared across the app:
  - `pipeline.rs`: `ShaderPipeline` creation and WGPU render pipeline management. Validates WGSL using naga before compilation.
  - `audio.rs`: `AudioState` for 3-band audio (bass/mid/high). Includes FFT-based input capture.
  - `audio_file.rs`: Audio playback from files (MP3) using rodio.
  - `audio_analyzer.rs`: Real-time FFT analysis for audio visualization. Processes audio samples and extracts frequency band energies.
  - `errors.rs`: `ShaderError` enum and formatting utilities.
  - `theme.rs`: Dark theme configuration for egui.
  - `fonts.rs`: Custom font registration for shader error display.
  - `monitors.rs`: Monitor detection via xrandr on Linux.
  - `toast.rs`: Toast notification system.
  - `wgsl_syntax.rs`: WGSL syntax highlighting for code editor.
- **`src/ui_components/`**: Reusable UI components like settings menus.
- **`src/funcs/`**: Placeholder for future function modules (currently redirects to utils).

### Shader Compilation Flow

1. User edits shader code in the editor (Fragment/Vertex tabs).
2. On "Apply" or Ctrl+Enter, `shader_needs_update` flag is set.
3. In the next `update()` frame, the combined vertex + fragment WGSL is validated with **naga** first.
4. If naga validation passes, WGPU creates a `ShaderPipeline`.
5. On success: pipeline is stored, toast shows success. On error: native egui error window displays formatted error.
6. The shader preview renders using `ShaderCallback`, passing uniforms (time, audio bands, resolution).

### Key Data Flow

- **Shader Pipeline**: Stored in `Arc<Mutex<Option<Arc<ShaderPipeline>>>>` for thread-safe access.
- **Audio State**: Three separate `Arc<Mutex<f32>>` values for bass, mid, and high frequency bands. Updated by audio thread (via `AudioAnalyzer`), read by shader callback.
- **Error Handling**: `Arc<Mutex<Option<ShaderError>>>` stores last error. Formatted and displayed in native egui window.
- **Uniforms**: `ShaderUniforms` struct (time, audio_bass, audio_mid, audio_high, resolution) passed to shader each frame via uniform buffer.

### WGSL Shader Requirements

Shaders MUST include:
- A `@vertex` function (typically `vs_main`) that outputs clip position and UVs.
- A `@fragment` function (typically `fs_main`) that returns a color.
- Bind group 0, binding 0: uniform buffer with `Uniforms` struct (see `test.frag` for example).
- Entry points: `vs_main` for vertex, `fs_main` for fragment (hardcoded in pipeline.rs:146, 152).

Example uniforms:
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
```

### Optional Feature: Code Editor

The `code_editor` feature enables `egui_code_editor` with syntax highlighting. Disabled by default to reduce dependencies. Enable with:
```bash
cargo run --features code_editor
```

Without the feature, falls back to plain `egui::TextEdit`.

## Common Patterns

### Adding New Shader Uniforms

1. Update `ShaderUniforms` struct in `src/utils/pipeline.rs`.
2. Ensure struct is `#[repr(C)]`, `Pod`, and `Zeroable`.
3. Update `ShaderCallback::prepare()` to populate new uniforms.
4. Update default shaders (`src/assets/shards/`) to use new uniforms.

### Handling Shader Errors

Errors are captured in `ShaderError` enum. Use `format_shader_error()` to format for display. Naga validation errors include line numbers and context. Displayed in native egui window (`show_error_window`).

### Audio Integration

Audio state is shared via three separate `Arc<Mutex<f32>>` values (bass_energy, mid_energy, high_energy). Two modes:
1. **Input capture** (FFT from mic): `start_input_fft()` in `audio.rs`.
2. **File playback** (MP3): `start_file_audio()` in `audio_file.rs` (currently used).

Both modes use `AudioAnalyzer` for FFT processing. The analyzer extracts frequency band energies:
- Bass: 20-250 Hz
- Mid: 250-4000 Hz
- High: 4000-20000 Hz

Tuning constants in `audio_analyzer.rs` control normalization and smoothing. Shader receives 3 bands as uniforms, updated each frame.

### UI Theming

Dark theme applied in `apply_editor_theme()` every frame to prevent drift. Custom colors in `src/utils/theme.rs`. Editor font size adjustable via settings or keyboard shortcuts (Ctrl+Plus/Minus/0).

## Keyboard Shortcuts

- **Ctrl/Cmd + Enter**: Apply shader
- **Ctrl/Cmd + Plus**: Increase font size
- **Ctrl/Cmd + Minus**: Decrease font size
- **Ctrl/Cmd + 0**: Reset font size to 16

## Debugging

- Enable logging: `RUST_LOG=debug cargo run` or `RUST_LOG=info cargo run`.
- Check shader compilation: Look for `[ShaderPipeline]` logs.
- Audio debugging: Enable "Debug Audio" in settings overlay to manually control band levels.
- WGPU validation: Errors logged to stderr. Check for naga parse/validation errors first.

## Code Conventions (from AGENTS.md)

- Edition: Rust 2021, rustfmt defaults (4-space indent).
- Imports: std, external crates, local modules. No glob imports.
- Naming: `snake_case` for functions/variables, `CamelCase` for types, `SCREAMING_SNAKE_CASE` for constants.
- Error handling: Return `Result<T, E>`, propagate with `?`. Avoid `unwrap`/`expect` except in tests or early init.
- Mutability: Prefer immutable bindings.
- Tests: Keep unit tests small, avoid GPU/windowing (mock or extract logic).

## Important Implementation Notes

- The application expects shaders in `src/assets/shards/` (test.frag, test.vert).
- Audio file hardcoded to `src/assets/test.mp3` in `TopApp::new()`.
- Window sizing: Defaults to 75% of primary monitor size, centered. Falls back to `DESIGN_W/H * UI_SCALE` if monitor detection fails.
- Shader compilation happens on the main thread during `update()`, not async. Keep shader compilation fast.
- The `ShaderCallback` trait renders the shader via WGPU callback, bypassing egui's normal rendering.
