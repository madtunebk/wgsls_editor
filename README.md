# WebShard Editor

A modern WGSL shader editor built with Rust and egui, featuring real-time shader compilation, syntax highlighting, and audio-reactive capabilities.

![Rust](https://img.shields.io/badge/rust-2021-orange.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

## Features

- 🎨 **Real-time WGSL Shader Editor** - Write and preview WGSL shaders with instant feedback
- 🌈 **Syntax Highlighting** - Full WGSL syntax highlighting with color-coded tokens
- 🔊 **Audio Reactive** - FFT-based audio analysis for shader uniforms (bass, mid, high frequencies)
- ⚡ **WGPU Backend** - Hardware-accelerated rendering using WebGPU
- 🎯 **Dual Editor** - Separate vertex and fragment shader editing
- 🛠️ **Error Handling** - Detailed shader compilation error reporting with line numbers
- 🎭 **Custom Themes** - Dark theme optimized for shader development
- 📝 **Toast Notifications** - User-friendly status messages

## Prerequisites

- Rust 2021 edition or later
- WGPU-compatible graphics driver
- Linux (primary development platform, should work on other platforms)

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
│   │   ├── editor.rs        # Main shader editor UI
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
│   │   ├── pipeline.rs      # WGSL shader pipeline
│   │   ├── text.rs          # Text utilities
│   │   ├── theme.rs         # UI theming
│   │   ├── toast.rs         # Toast notifications
│   │   ├── wgsl_syntax.rs   # WGSL syntax highlighting
│   │   └── mod.rs
│   ├── funcs/
│   │   └── mod.rs
│   └── assets/
│       ├── fonts/           # Material Symbols & Inter fonts
│       └── shards/          # Default shader templates
│           ├── test.frag
│           └── test.vert
├── data/
│   └── wgsl_builtins.json   # WGSL language definitions
└── Cargo.toml
```

## Usage

### Shader Editing

1. **Switch Tabs**: Toggle between Fragment and Vertex shader editors
2. **Live Preview**: Shaders compile and render in real-time
3. **Error Messages**: Compilation errors appear with line numbers and descriptions

### Audio Features

- Load audio files to drive shader uniforms
- Access FFT data: `u_bass`, `u_mid`, `u_high` in your shaders
- Real-time frequency analysis visualization

### Keyboard Shortcuts

- `Ctrl+S` - Save shader (if implemented)
- `Ctrl+,` - Open settings
- Tab switching for editor panels

## Shader Uniforms

The following uniforms are automatically provided to your shaders:

```wgsl
@group(0) @binding(0) var<uniform> u_time: f32;      // Time in seconds
@group(0) @binding(1) var<uniform> u_resolution: vec2<f32>; // Screen resolution
@group(0) @binding(2) var<uniform> u_mouse: vec2<f32>;      // Mouse position
@group(0) @binding(3) var<uniform> u_bass: f32;      // Bass frequency energy
@group(0) @binding(4) var<uniform> u_mid: f32;       // Mid frequency energy
@group(0) @binding(5) var<uniform> u_high: f32;      // High frequency energy
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
- WGSL shader language by the WebGPU working group
- Inspired by [ShaderToy](https://www.shadertoy.com/)

## Troubleshooting

### Graphics Driver Issues

If you encounter WGPU initialization errors:
```bash
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