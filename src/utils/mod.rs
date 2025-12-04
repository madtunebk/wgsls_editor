pub mod errors;
pub mod fonts;
pub mod monitors;
pub mod pipeline;
pub mod text;
pub mod theme;
pub mod toast;
pub mod wgsl_syntax;

pub use errors::{format_shader_error, panic_to_string, parse_wgsl_error, ShaderError};
pub use fonts::register_error_fonts;
pub use monitors::detect_primary_monitor_xrandr;
pub use pipeline::{ShaderCallback, ShaderPipeline, ShaderUniforms};
pub use text::{apply_completion, byte_index_from_char_index};
pub use theme::{apply_editor_theme, apply_viewer_theme};
pub use toast::{Toast, ToastManager, ToastType};
