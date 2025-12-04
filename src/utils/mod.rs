pub mod fonts;
pub mod monitors;
pub mod errors;
pub mod text;

pub use errors::panic_to_string;
pub use fonts::register_error_fonts;
pub use monitors::detect_primary_monitor_xrandr;
pub use text::{apply_completion, byte_index_from_char_index};
