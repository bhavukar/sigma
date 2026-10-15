use crate::canvas::{ActiveTool, CanvasState};
use crate::network::NetworkManager;
use egui::{Color32, RichText, Stroke, Ui, Vec2 as EguiVec2};
use sigma_core::{Document, History};

pub fn render_toolbar(
    ui: &mut Ui,
    doc: &mut Document,
    state: &mut CanvasState,
    history: &mut History,
    network: &NetworkManager,
) {
    ui.horizontal(|ui| {
        ui.add_space(10.0);

        // 1. Clean Typographic Logo: "Sigma"
        crate::logo::render_brand_lockup(ui, 0.0);

        ui.add_space(10.0);
        ui.label(RichText::new("/").size(13.0).color(Color32::from_rgb(75, 75, 85)));
        ui.add_space(8.0);

        // 2. Document Name (Clean & Minimal)
        ui.label(
            RichText::new(&doc.name)
                .size(12.5)
                .color(Color32::from_rgb(180, 180, 190)),
        );

        ui.add_space(10.0);

        // Subtle Undo / Redo
        let can_undo = history.can_undo();
        let undo_resp = ui
            .add_enabled(
                can_undo,
                egui::Button::new(RichText::new("↶").size(12.0).color(if can_undo {
                    Color32::from_rgb(200, 200, 210)
                } else {
                    Color32::from_rgb(90, 90, 100)
                })),
            )
            .on_hover_text("Undo (⌘Z)");
        if undo_resp.clicked() {
            if let Some(prev) = history.undo(doc) {
                *doc = prev;
                network.send_doc_update(doc);
            }
        }

        let can_redo = history.can_redo();
        let redo_resp = ui
            .add_enabled(
                can_redo,
                egui::Button::new(RichText::new("↷").size(12.0).color(if can_redo {
                    Color32::from_rgb(200, 200, 210)
                } else {
                    Color32::from_rgb(90, 90, 100)
                })),
            )
            .on_hover_text("Redo (⇧⌘Z)");
        if redo_resp.clicked() {
            if let Some(next) = history.redo(doc) {
                *doc = next;
                network.send_doc_update(doc);
            }
        }

        // 3. Right side layout for Actions (Share, Present, Avatar, Shortcuts)
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(12.0);

            // Shortcuts modal button (?)
            let sc_btn = egui::Button::new(
                RichText::new("?")
                    .size(12.0)
                    .strong()
                    .color(Color32::from_rgb(160, 160, 175)),
            )
            .fill(Color32::from_rgb(26, 26, 30))
            .stroke(Stroke::new(1.0, Color32::from_rgb(44, 44, 52)))
            .corner_radius(egui::CornerRadius::same(5));
            if ui.add(sc_btn).on_hover_text("Keyboard Shortcuts (?)").clicked() {
                state.show_shortcuts_modal = true;
            }

            ui.add_space(6.0);

            // Present Button
            let pres_active = state.is_presenting;
            let pres_btn = egui::Button::new(
                RichText::new(if pres_active { "⏹ Stop" } else { "▶ Present" })
                    .size(11.5)
                    .strong()
                    .color(Color32::WHITE),
            )
            .fill(if pres_active {
                Color32::from_rgb(220, 38, 38)
            } else {
                Color32::from_rgb(16, 185, 129)
            })
            .corner_radius(egui::CornerRadius::same(5));

            if ui
                .add(pres_btn)
                .on_hover_text("Preview Prototype (Esc to exit)")
                .clicked()
            {
                state.is_presenting = !state.is_presenting;
            }

            ui.add_space(6.0);

            // Share Button
            let share_url = format!(
                "https://sigma-sync.bhavuk-arora03.workers.dev/?room={}",
                network.room_id
            );
            let share_btn = egui::Button::new(
                RichText::new("Share")
                    .size(11.5)
                    .color(Color32::from_rgb(230, 230, 238)),
            )
            .fill(Color32::from_rgb(30, 30, 36))
            .stroke(Stroke::new(1.0, Color32::from_rgb(50, 50, 60)))
            .corner_radius(egui::CornerRadius::same(5));

            if ui
                .add(share_btn)
                .on_hover_text(&format!("Copy share link: {}", share_url))
                .clicked()
            {
                ui.ctx().copy_text(share_url);
            }

            ui.add_space(6.0);

            // User Profile / Sign In
            if state.auth_is_signed_in {
                let user_pill = egui::Button::new(
                    RichText::new(&state.auth_user_name)
                        .size(11.5)
                        .color(Color32::from_rgb(56, 189, 248)),
                )
                .fill(Color32::from_rgb(24, 24, 28))
                .stroke(Stroke::new(1.0, Color32::from_rgb(44, 44, 52)))
                .corner_radius(egui::CornerRadius::same(5));
                if ui.add(user_pill).on_hover_text("Account Settings").clicked() {
                    state.show_auth_modal = true;
                }
            } else {
                let signin_btn = egui::Button::new(
                    RichText::new("Sign In")
                        .size(11.5)
                        .color(Color32::from_rgb(200, 200, 210)),
                )
                .fill(Color32::TRANSPARENT)
                .stroke(Stroke::new(1.0, Color32::from_rgb(46, 46, 54)))
                .corner_radius(egui::CornerRadius::same(5));
                if ui.add(signin_btn).on_hover_text("Sign In").clicked() {
                    state.show_auth_modal = true;
                }
            }

            // Remote peer avatars (if any)
            for peer in state.remote_peers.values() {
                let peer_c = Color32::from_rgb(peer.color.r, peer.color.g, peer.color.b);
                render_avatar_bubble(ui, &peer.name, peer_c, false);
            }

            // Center: Minimal Segmented Tools Dock
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                let avail = ui.available_width();
                let dock_width = 240.0;
                let left_pad = ((avail - dock_width) * 0.5).max(16.0);
                ui.add_space(left_pad);

                let tools = [
                    (ActiveTool::Select, "↖", "Move (V)"),
                    (ActiveTool::Frame, "⊞", "Frame (F)"),
                    (ActiveTool::Rectangle, "▭", "Rectangle (R)"),
                    (ActiveTool::Ellipse, "○", "Ellipse (O)"),
                    (ActiveTool::Line, "—", "Line (L)"),
                    (ActiveTool::Text, "T", "Text (T)"),
                    (ActiveTool::Hand, "✋", "Hand (H)"),
                ];

                egui::Frame::new()
                    .fill(Color32::from_rgb(22, 22, 26))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(40, 40, 48)))
                    .corner_radius(egui::CornerRadius::same(6))
                    .inner_margin(egui::Margin::symmetric(4, 2))
                    .show(ui, |ui| {
                        for (tool, icon, tip) in tools {
                            let is_active = state.active_tool == tool;
                            let btn = egui::Button::new(
                                RichText::new(icon)
                                    .size(13.0)
                                    .strong()
                                    .color(if is_active {
                                        Color32::from_rgb(56, 189, 248)
                                    } else {
                                        Color32::from_rgb(180, 180, 192)
                                    }),
                            )
                            .fill(if is_active {
                                Color32::from_rgb(36, 36, 44)
                            } else {
                                Color32::TRANSPARENT
                            })
                            .stroke(egui::Stroke::NONE)
                            .corner_radius(egui::CornerRadius::same(4));

                            if ui.add(btn).on_hover_text(tip).clicked() {
                                state.active_tool = tool;
                            }
                        }
                    });
            });
        });
    });
}

fn render_avatar_bubble(ui: &mut Ui, name: &str, color: Color32, is_self: bool) -> bool {
    let initial = name.chars().next().unwrap_or('?').to_uppercase().to_string();
    let (rect, resp) = ui.allocate_exact_size(EguiVec2::splat(22.0), egui::Sense::click());
    let painter = ui.painter_at(rect);
    let center = rect.center();

    painter.circle(center, 11.0, color, Stroke::new(1.5, Color32::from_rgb(35, 35, 40)));
    painter.text(
        center,
        egui::Align2::CENTER_CENTER,
        initial,
        egui::FontId::proportional(10.5),
        Color32::WHITE,
    );

    let hover_text = if is_self {
        format!("You ({}) - Click to manage account", name)
    } else {
        name.to_string()
    };
    resp.on_hover_text(hover_text).clicked()
}
