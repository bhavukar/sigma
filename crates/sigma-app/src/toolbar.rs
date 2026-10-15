use crate::canvas::{ActiveTool, CanvasState};
use crate::network::NetworkManager;
use egui::{Color32, RichText, Stroke, Ui, Vec2 as EguiVec2};
use sigma_core::{export_to_svg, Document, History};

pub fn render_toolbar(
    ui: &mut Ui,
    doc: &mut Document,
    state: &mut CanvasState,
    history: &mut History,
    network: &NetworkManager,
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
                network.send_doc_update(doc);
            }
        }

        let can_redo = history.can_redo();
        let redo_resp = ui.add_enabled(can_redo, egui::Button::new("↷ Redo"));
        if redo_resp.clicked() {
            if let Some(next) = history.redo(doc) {
                *doc = next;
                network.send_doc_update(doc);
            }
        }

        ui.add_space(16.0);
        ui.separator();

        // Right side: Multiplayer presence, Zoom & Export
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(8.0);

            // Share Room Button
            let share_url = format!(
                "https://sigma-sync.bhavuk-arora03.workers.dev/?room={}",
                network.room_id
            );
            if ui
                .button(RichText::new("🔗 Share").color(Color32::from_rgb(10, 132, 255)))
                .on_hover_text(&format!("Copy share link: {}", share_url))
                .clicked()
            {
                ui.ctx().copy_text(share_url);
            }

            // Export SVG
            if ui.button("⚡ Export SVG").clicked() {
                let svg_content = export_to_svg(doc);
                ui.ctx().copy_text(svg_content);
            }

            ui.separator();

            // Multiplayer Peer Avatars (Figma style)
            // 1. Local user avatar bubble
            let local_c = Color32::from_rgb(
                network.user_color.r,
                network.user_color.g,
                network.user_color.b,
            );
            render_avatar_bubble(ui, &network.user_name, local_c, true);

            // 2. Remote peer avatar bubbles
            for peer in state.remote_peers.values() {
                let peer_c = Color32::from_rgb(peer.color.r, peer.color.g, peer.color.b);
                render_avatar_bubble(ui, &peer.name, peer_c, false);
            }

            // Live status indicator
            let total_users = 1 + state.remote_peers.len();
            let live_label = if total_users > 1 {
                format!("🟢 {} Designers Live", total_users)
            } else {
                "🟢 Edge Ready".to_string()
            };
            ui.label(
                RichText::new(live_label)
                    .size(11.0)
                    .color(Color32::from_rgb(16, 185, 129)),
            );

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

            // Room / Document Name
            let title_text = format!("{} ({})", doc.name, network.room_id);
            ui.label(
                RichText::new(title_text)
                    .color(Color32::from_rgb(200, 200, 200))
                    .size(12.0),
            );
        });
    });
}

fn render_avatar_bubble(ui: &mut Ui, name: &str, color: Color32, is_self: bool) {
    let initial = name.chars().next().unwrap_or('?').to_uppercase().to_string();
    let (rect, resp) = ui.allocate_exact_size(EguiVec2::splat(24.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let center = rect.center();

    painter.circle(center, 12.0, color, Stroke::new(1.5, Color32::from_rgb(40, 40, 40)));
    painter.text(
        center,
        egui::Align2::CENTER_CENTER,
        initial,
        egui::FontId::proportional(11.0),
        Color32::WHITE,
    );

    let hover_text = if is_self {
        format!("You ({})", name)
    } else {
        name.to_string()
    };
    resp.on_hover_text(hover_text);
}
