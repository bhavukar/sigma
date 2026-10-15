use crate::canvas::{ActiveTool, CanvasState};
use egui::{Color32, RichText, Ui};
use sigma_core::{export_to_svg, Document, History};

pub fn render_toolbar(
    ui: &mut Ui,
    doc: &mut Document,
    state: &mut CanvasState,
    history: &mut History,
) {
    ui.horizontal(|ui| {
        ui.add_space(8.0);

        // Tool buttons
        let tools = [
            (ActiveTool::Select, "↖ Move", "V"),
            (ActiveTool::Frame, "⊞ Frame", "F"),
            (ActiveTool::Rectangle, "▭ Rectangle", "R"),
            (ActiveTool::Ellipse, "○ Ellipse", "O"),
            (ActiveTool::Text, "T Text", "T"),
            (ActiveTool::Hand, "✋ Hand", "H"),
        ];

        for (tool, label, shortcut) in tools {
            let is_active = state.active_tool == tool;
            let btn_text = format!("{} ({})", label, shortcut);
            let btn = ui.selectable_label(is_active, btn_text);
            if btn.clicked() {
                state.active_tool = tool;
            }
        }

        ui.add_space(16.0);
        ui.separator();
        ui.add_space(16.0);

        // Undo / Redo
        let can_undo = history.can_undo();
        let undo_resp = ui.add_enabled(can_undo, egui::Button::new("↶ Undo"));
        if undo_resp.clicked() {
            if let Some(prev) = history.undo(doc) {
                *doc = prev;
            }
        }

        let can_redo = history.can_redo();
        let redo_resp = ui.add_enabled(can_redo, egui::Button::new("↷ Redo"));
        if redo_resp.clicked() {
            if let Some(next) = history.redo(doc) {
                *doc = next;
            }
        }

        ui.add_space(16.0);
        ui.separator();

        // Right side: Zoom & Export
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(8.0);

            // Export SVG
            if ui.button("⚡ Export SVG").clicked() {
                let svg_content = export_to_svg(doc);
                ui.ctx().copy_text(svg_content);
            }

            ui.separator();

            // Zoom In / Out
            if ui.button("+").on_hover_text("Zoom in").clicked() {
                state.zoom = (state.zoom * 1.2).min(32.0);
            }

            let zoom_str = format!("{:.0}%", state.zoom * 100.0);
            if ui.button(RichText::new(zoom_str).strong()).on_hover_text("Reset to 100%").clicked() {
                state.zoom = 1.0;
            }

            if ui.button("-").on_hover_text("Zoom out").clicked() {
                state.zoom = (state.zoom / 1.2).max(0.05);
            }

            ui.separator();

            // Document Name
            ui.label(
                RichText::new(&doc.name)
                    .color(Color32::from_rgb(200, 200, 200))
                    .size(13.0),
            );
        });
    });
}
