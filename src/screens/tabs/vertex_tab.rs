// Vertex shader tab
use eframe::egui;

#[cfg(feature = "code_editor")]
use crate::utils::wgsl_syntax;

pub struct VertexTab {
    pub code: String,
    editor_font_size: f32,
}

impl VertexTab {
    pub fn new(initial_code: String, font_size: f32) -> Self {
        Self {
            code: initial_code,
            editor_font_size: font_size,
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui, buffer_name: &str) {
        ui.set_min_height(ui.available_height());

        #[cfg(feature = "code_editor")]
        {
            let editor_id = format!("vert_editor_{}", buffer_name);
            
            egui_code_editor::CodeEditor::default()
                .id_source(&editor_id)
                .with_fontsize(self.editor_font_size)
                .with_theme(egui_code_editor::ColorTheme::GITHUB_DARK)
                .with_syntax(wgsl_syntax::wgsl())
                .with_numlines(true)
                .vscroll(true)
                .auto_shrink(false)
                .show(ui, &mut self.code);
        }

        #[cfg(not(feature = "code_editor"))]
        {
            let editor_id = egui::Id::new(format!("vert_editor_{}", buffer_name));
            
            ui.add(
                egui::TextEdit::multiline(&mut self.code)
                    .id(editor_id)
                    .font(egui::TextStyle::Monospace)
                    .code_editor()
                    .desired_width(f32::INFINITY)
                    .desired_rows(30),
            );
        }
    }

    pub fn set_code(&mut self, code: String) {
        self.code = code;
    }

    pub fn get_code(&self) -> &str {
        &self.code
    }

    pub fn set_font_size(&mut self, size: f32) {
        self.editor_font_size = size;
    }
}
