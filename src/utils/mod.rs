pub mod audio;
pub mod audio_analyzer;
pub mod audio_file;
pub mod errors;
pub mod fonts;
pub mod monitors;
pub mod pipeline;
pub mod text;
pub mod theme;
pub mod toast;
pub mod wgsl_syntax;

pub use errors::{format_shader_error, ShaderError};
pub use fonts::register_error_fonts;
pub use monitors::detect_primary_monitor_xrandr;
pub use pipeline::{ShaderCallback, ShaderPipeline};
pub use theme::apply_editor_theme;
pub use toast::ToastManager;
