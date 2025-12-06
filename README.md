# WebShard Editor

A modern WGSL shader editor built with Rust and egui, featuring real-time multi-pass shader compilation, syntax highlighting, and audio-reactive capabilities.

![Rust](https://img.shields.io/badge/rust-2021-orange.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

## Features

- 🎨 **Real-time WGSL Shader Editor** - Write and preview WGSL shaders with instant feedback
- 🌈 **Syntax Highlighting** - Full WGSL syntax highlighting with color-coded tokens
- 🔊 **Audio Reactive** - FFT-based audio analysis for shader uniforms (bass, mid, high frequencies)
- 🎬 **Multi-Pass Rendering** - MainImage + 4 buffers (A-D) with texture sampling between passes
- ⚡ **WGPU Backend** - Hardware-accelerated rendering using WebGPU
- 🎯 **Auto-Injection** - Automatic uniform and vertex shader injection (no boilerplate needed)
- 🛠️ **Validation** - Real-time shader validation with detailed error reporting
- ⌨️ **Keyboard Shortcuts** - Efficient workflow with Ctrl+Enter apply, Ctrl+Plus/Minus font size
- 🎭 **Custom Themes** - Dark theme optimized for shader development
- 📝 **Preset Shaders** - Built-in examples: psychedelic, tunnel, raymarch, fractal
- 💾 **State Persistence** - Auto-saves editor state between sessions
- 📦 **Import/Export** - Save and share shaders as JSON with base64 encoding support
- 🔔 **Smart Notifications** - Toast notifications with auto-dismiss and error persistence

## Prerequisites

- Rust 2021 edition or later
- WGPU-compatible graphics driver
- Linux (primary development platform, should work on other platforms)

## Quick Start

WebShard Editor uses **auto-injection** - you only write the fragment shader logic!

### Minimal Shader Example

```wgsl
// Just write your fragment shader - uniforms and vertex shader are auto-injected!
@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let t = uniforms.time;
    
    // Animated gradient
    let col = vec3(
        0.5 + 0.5 * sin(uv.x * 10.0 + t),
        0.5 + 0.5 * cos(uv.y * 10.0 + t),
        0.5 + 0.5 * sin((uv.x + uv.y) * 5.0 + t)
    );
    
    return vec4(col, 1.0);
}
```

### Auto-Injected Code

The editor automatically provides:

**1. Uniforms Structure:**
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

**2. Vertex Shader:**
```wgsl
struct VSOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VSOut {
    // Full-screen triangle vertex shader (auto-injected)
}
```

**3. Multi-Pass Textures:**
```wgsl
@group(1) @binding(0) var buffer_a_texture: texture_2d<f32>;
@group(1) @binding(1) var buffer_b_texture: texture_2d<f32>;
@group(1) @binding(2) var buffer_c_texture: texture_2d<f32>;
@group(1) @binding(3) var buffer_d_texture: texture_2d<f32>;
@group(1) @binding(4) var texture_sampler: sampler;
```

### Multi-Pass Example

**Buffer A** (generates pattern):
```wgsl
@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    let spiral = atan2(in.uv.y - 0.5, in.uv.x - 0.5) + uniforms.time;
    return vec4(sin(spiral * 5.0), 0.0, 0.0, 1.0);
}
```

**MainImage** (samples Buffer A):
```wgsl
@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    // Sample from Buffer A
    let buffer_a_color = textureSample(buffer_a_texture, texture_sampler, in.uv);
    
    // Apply effects
    return vec4(buffer_a_color.rgb * 2.0, 1.0);
}
```

## Building & Running

### Debug Build
```bash
cargo build
cargo run
```

### Release Build (Recommended)
```bash
cargo build --release
cargo run --release
```

### With Logging
```bash
RUST_LOG=info cargo run --release
```

## Development

### Running Tests
```bash
cargo test
```

### Running Specific Tests
```bash
cargo test <module>::<test_name>
# or pattern matching
cargo test <pattern>
```

### Debugging Tests
```bash
RUST_BACKTRACE=1 cargo test -- --nocapture
```

### Code Formatting
```bash
cargo fmt
```

### Linting
```bash
cargo clippy --all-targets -- -D warnings
```

## Project Structure

```
.
├── src/
│   ├── main.rs              # Application entry point
│   ├── screens/
│   │   ├── editor.rs        # Main shader editor UI and state
│   │   ├── tabs/            # Individual buffer tab modules
│   │   │   ├── main_image_tab.rs
│   │   │   ├── buffer_a_tab.rs
│   │   │   ├── buffer_b_tab.rs
│   │   │   ├── buffer_c_tab.rs
│   │   │   └── buffer_d_tab.rs
│   │   └── mod.rs
│   ├── ui_components/
│   │   ├── settings_menu.rs # Settings panel
│   │   └── mod.rs
│   ├── utils/
│   │   ├── audio.rs         # Audio playback
│   │   ├── audio_analyzer.rs # FFT audio analysis
│   │   ├── audio_file.rs    # Audio file loading
│   │   ├── errors.rs        # Error formatting
│   │   ├── fonts.rs         # Font registration
│   │   ├── monitors.rs      # Monitor detection
│   │   ├── multi_buffer_pipeline.rs # Multi-pass rendering pipeline
│   │   ├── shader_json.rs   # Shader import/export (JSON + base64)
│   │   ├── shader_constants.rs # WGSL constants and boilerplate
│   │   ├── notification.rs  # Smart notification system
│   │   ├── panic_handler.rs # Global panic handler
│   │   ├── text.rs          # Text utilities
│   │   ├── theme.rs         # UI theming
│   │   ├── toast.rs         # Toast notifications
│   │   ├── wgsl_syntax.rs   # WGSL syntax highlighting
│   │   └── mod.rs
│   └── assets/
│       ├── fonts/           # Material Symbols & fonts
│       └── shards/          # Preset shader templates
│           ├── psychedelic.frag
│           ├── tunnel.frag
│           ├── raymarch.frag
│           └── fractal.frag
├── data/
│   └── wgsl_builtins.json   # WGSL language definitions
└── Cargo.toml
```

## Usage

### Buffer Tabs

1. **MainImage** - Final output (required)
2. **Buffer A-D** - Intermediate render targets for multi-pass effects

Each buffer has its own fragment shader. Switch between tabs to edit different passes.

### Workflow

1. **Write Shader**: Only write `@fragment fn fs_main()` - boilerplate is auto-injected
2. **Apply Changes**: Press `Ctrl+Enter` or click "Apply Pipeline"
3. **Load Presets**: Click "Shader Properties" to load example shaders
4. **Multi-Pass**: Use Buffer A-D for feedback, blur, or complex effects
5. **Audio Reactive**: Access `uniforms.audio_bass/mid/high` for audio-driven visuals

### Preset Shaders

- **Default** - Simple gradient animation
- **Psychedelic** - Flowing noise patterns with audio reactivity  
- **Tunnel** - Audio-reactive spiral tunnel
- **Raymarch** - 3D raymarching scene with rotating boxes
- **Fractal** - Julia set fractal explorer

### Keyboard Shortcuts

- `Ctrl+Enter` - Apply shader changes (compile pipeline)
- `Ctrl+E` - Export shader to JSON file
- `Ctrl+S` - Save current shader state
- `Ctrl++` / `Ctrl+-` - Increase/decrease editor font size
- `Ctrl+0` - Reset font size to default
- `Ctrl+,` - Open settings menu

### Import/Export

**Export Shader:**
1. Click "Shader Properties" → "Export Shard" or press `Ctrl+E`
2. Saves all buffers (MainImage + A-D) as JSON
3. Supports base64 encoding for safe text transport

**JSON Format:**
```json
{
  "version": "1.0",
  "encoding": "base64",
  "fragment": "base64_encoded_main_image_shader",
  "buffer_a": "base64_encoded_buffer_a_shader",
  "buffer_b": "base64_encoded_buffer_b_shader"
}
```

**Import Shader:**
- Load JSON files with automatic base64 decoding
- Supports both plain text and encoded formats
- Validates shader structure before import

## Shader Uniforms

Access these in your fragment shader via the auto-injected `uniforms` struct:

```wgsl
@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    let t = uniforms.time;                    // Time in seconds
    let bass = uniforms.audio_bass;           // Bass energy (0.0-1.0)
    let mid = uniforms.audio_mid;             // Mid energy (0.0-1.0)
    let high = uniforms.audio_high;           // High energy (0.0-1.0)
    let res = uniforms.resolution;            // Screen resolution
    let uv = in.uv;                           // UV coordinates (0.0-1.0)
    
    // Your shader code here
    return vec4(uv.x, uv.y, sin(t), 1.0);
}
```

## Multi-Pass Textures

Sample from other buffers in MainImage:

```wgsl
@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    // Sample Buffer A at current UV
    let buffer_a = textureSample(buffer_a_texture, texture_sampler, in.uv);
    
    // Sample Buffer B with offset
    let buffer_b = textureSample(buffer_b_texture, texture_sampler, in.uv + vec2(0.01));
    
    return mix(buffer_a, buffer_b, 0.5);
}
```oup(0) @binding(5) var<uniform> u_high: f32;      // High frequency energy
```

## Dependencies

- **eframe** (0.33) - egui framework with WGPU backend
- **egui-wgpu** (0.33) - WGPU integration
- **egui_code_editor** (0.2.20) - Code editing widget
- **naga** (27) - WGSL shader validation
- **cpal** (0.15) - Cross-platform audio I/O
- **rustfft** (6.2) - FFT implementation
- **rodio** (0.17) - Audio playback
- **serde/serde_json** - Serialization
- **base64** (0.22) - Base64 encoding for shader export
- **dirs** (5.0) - Platform-specific directories
- **regex** (1.10) - Pattern matching

## Contributing

Contributions are welcome! Please:

1. Follow Rust 2021 edition best practices
2. Run `cargo fmt` before committing
3. Ensure `cargo clippy` passes with no warnings
4. Add tests for new features
5. Update documentation as needed

## License

MIT License - see LICENSE file for details

## Acknowledgments

- Built with [egui](https://github.com/emilk/egui)
## Roadmap

- [x] Multi-pass shader support (MainImage + 4 buffers)
- [x] Shader presets library (psychedelic, tunnel, raymarch, fractal)
- [x] Auto-injection of uniforms and vertex shaders
- [x] Import/Export shaders as JSON with base64 encoding
- [x] Smart notification system with auto-dismiss
- [x] Global panic handler for graceful error recovery
- [ ] Import functionality (JSON → editor)
- [ ] Export shader as image/video
- [ ] Texture/image inputs
- [ ] Mouse input uniforms
- [ ] WebGL export
- [ ] Plugin system
- [ ] Cloud shader sharing
# Check WGPU backend
RUST_LOG=wgpu=debug cargo run
```

### Audio Issues

If audio features don't work:
```bash
# List audio devices
RUST_LOG=cpal=debug cargo run
```

### Build Errors

```bash
# Clean build
cargo clean
cargo build --release
```

## Roadmap

- [ ] Shader presets library
- [ ] Export shader as image/video
- [ ] Multi-pass shader support
- [ ] Texture/image inputs
- [ ] WebGL export
- [ ] Plugin system
- [ ] Cloud shader sharing

---

**Made with ❤️ using Rust and egui**