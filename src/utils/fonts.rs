use eframe::egui;
use std::fs;
use std::path::Path;

pub fn register_error_fonts(ctx: &egui::Context) {
    let inter = Path::new("src/assets/fonts/static/Inter_18pt-Regular.ttf");
    let roboto_mono = Path::new("src/assets/fonts/static/RobotoMono-Regular.ttf");
    let material = Path::new(
        "src/assets/fonts/Material_Symbols_Rounded/static/MaterialSymbolsRounded-Regular.ttf",
    );

    let mut defs = egui::FontDefinitions::default();
    let mut changed = false;
    if inter.exists() {
        if let Ok(bytes) = fs::read(inter) {
            defs.font_data
                .insert("Inter".to_owned(), egui::FontData::from_owned(bytes).into());
            defs.families
                .entry(egui::FontFamily::Name("Inter".into()))
                .or_default()
                .insert(0, "Inter".to_owned());
            changed = true;
        }
    }
    if roboto_mono.exists() {
        if let Ok(bytes) = fs::read(roboto_mono) {
            defs.font_data.insert(
                "RobotoMono".to_owned(),
                egui::FontData::from_owned(bytes).into(),
            );
            defs.families
                .entry(egui::FontFamily::Name("RobotoMono".into()))
                .or_default()
                .insert(0, "RobotoMono".to_owned());
            changed = true;
        }
    }
    if material.exists() {
        if let Ok(bytes) = fs::read(material) {
            defs.font_data.insert(
                "MaterialIcons".to_owned(),
                egui::FontData::from_owned(bytes).into(),
            );
            defs.families
                .entry(egui::FontFamily::Name("MaterialIcons".into()))
                .or_default()
                .insert(0, "MaterialIcons".to_owned());
            changed = true;
        }
    }
    if changed {
        ctx.set_fonts(defs);
    }
}
