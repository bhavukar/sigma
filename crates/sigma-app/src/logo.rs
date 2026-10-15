use egui::{Color32, RichText, Ui};

/// Renders the clean typographic brand logo: "Sigma"
pub fn render_brand_lockup(ui: &mut Ui, _size: f32) -> egui::Response {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Sigma")
                .size(15.0)
                .strong()
                .color(Color32::from_rgb(245, 245, 247)),
        );
    })
    .response
}
