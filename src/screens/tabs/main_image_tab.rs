// MainImage buffer tab
use eframe::egui;

#[cfg(feature = "code_editor")]
use crate::utils::wgsl_syntax;

pub struct MainImageTab {
    pub fragment_code: String,
    pub vertex_code: String,
    editor_font_size: f32,
    active_shader: u8, // 0 = Fragment, 1 = Vertex
}

impl MainImageTab {
    pub fn new(vertex: String, fragment: String, font_size: f32) -> Self {
        Self {
            fragment_code: fragment,
            vertex_code: vertex,
            editor_font_size: font_size,
            active_shader: 0,
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui, is_fragment_tab: bool) {
        ui.set_min_height(ui.available_height());

        let code = if is_fragment_tab {
            &mut self.fragment_code
        } else {
            &mut self.vertex_code
        };

        let shader_type = if is_fragment_tab { "frag" } else { "vert" };
        let editor_id = format!("main_image_{}_{}", shader_type, is_fragment_tab as u8);

        #[cfg(feature = "code_editor")]
        {
            egui_code_editor::CodeEditor::default()
                .id_source(&editor_id)
                .with_fontsize(self.editor_font_size)
                .with_theme(egui_code_editor::ColorTheme::GITHUB_DARK)
                .with_syntax(wgsl_syntax::wgsl())
                .with_numlines(true)
                .vscroll(true)
                .auto_shrink(false)
                .show(ui, code);
        }

        #[cfg(not(feature = "code_editor"))]
        {
            ui.add(
                egui::TextEdit::multiline(code)
                    .id(egui::Id::new(&editor_id))
                    .font(egui::TextStyle::Monospace)
                    .code_editor()
                    .desired_width(f32::INFINITY)
                    .desired_rows(30),
            );
        }
    }

    pub fn set_font_size(&mut self, size: f32) {
        self.editor_font_size = size;
    }

    pub fn get_shaders(&self) -> (&str, &str) {
        (&self.vertex_code, &self.fragment_code)
    }

    pub fn set_fragment(&mut self, code: String) {
        self.fragment_code = code;
    }

    pub fn set_vertex(&mut self, code: String) {
        self.vertex_code = code;
    }
}