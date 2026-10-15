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
            (ActiveTool::Rectangle, "▭", "R"),
            (ActiveTool::Ellipse, "○", "O"),
            (ActiveTool::Line, "—", "L"),
            (ActiveTool::Arrow, "→", "A"),
            (ActiveTool::Star, "★", "S"),
            (ActiveTool::Polygon, "△", "P"),
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

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        // Frame Presets Quick Add
        egui::ComboBox::from_id_salt("frame_presets_combo")
            .selected_text("📐 Presets")
            .show_ui(ui, |ui| {
                let center = sigma_core::Vec2::new(-state.pan.x + 400.0, -state.pan.y + 350.0);

                ui.label(RichText::new("PHONES").strong().size(10.0).color(Color32::from_rgb(140, 140, 150)));
                let phone_presets = [
                    ("iPhone 16 Pro Max", 440.0, 956.0, 48.0),
                    ("iPhone 16 Pro", 402.0, 874.0, 48.0),
                    ("iPhone 16 / 15", 393.0, 852.0, 48.0),
                    ("iPhone SE", 320.0, 568.0, 20.0),
                    ("Pixel 9 Pro", 412.0, 924.0, 40.0),
                    ("Android Large", 412.0, 915.0, 24.0),
                ];
                for (name, w, h, r) in phone_presets {
                    if ui.button(format!("📱 {} ({}×{})", name, w as i32, h as i32)).clicked() {
                        let pos = doc.get_next_frame_position(center, w);
                        let id = doc.add_frame_preset_styled(name, w, h, pos, r);
                        state.selected_ids = vec![id];
                        state.active_tool = ActiveTool::Select;
                        history.push_state(doc);
                        network.send_selection(&state.selected_ids);
                        network.send_doc_update(doc);
                    }
                }

                ui.separator();
                ui.label(RichText::new("DESKTOP & TABLET").strong().size(10.0).color(Color32::from_rgb(140, 140, 150)));
                let desk_presets = [
                    ("MacBook Pro 16\"", 1728.0, 1117.0, 16.0),
                    ("MacBook Pro 14\"", 1512.0, 982.0, 16.0),
                    ("MacBook Air 13\"", 1280.0, 832.0, 16.0),
                    ("Desktop HD 1440", 1440.0, 1024.0, 8.0),
                    ("iPad Pro 13\" M4", 1032.0, 1376.0, 24.0),
                    ("iPad Pro 11\" M4", 834.0, 1210.0, 24.0),
                ];
                for (name, w, h, r) in desk_presets {
                    if ui.button(format!("💻 {} ({}×{})", name, w as i32, h as i32)).clicked() {
                        let pos = doc.get_next_frame_position(center, w);
                        let id = doc.add_frame_preset_styled(name, w, h, pos, r);
                        state.selected_ids = vec![id];
                        state.active_tool = ActiveTool::Select;
                        history.push_state(doc);
                        network.send_selection(&state.selected_ids);
                        network.send_doc_update(doc);
                    }
                }

                ui.separator();
                ui.label(RichText::new("SLIDES & SOCIAL").strong().size(10.0).color(Color32::from_rgb(140, 140, 150)));
                let media_presets = [
                    ("Slide 16:9", 1920.0, 1080.0, 0.0),
                    ("Instagram Post", 1080.0, 1080.0, 0.0),
                    ("Instagram Story", 1080.0, 1920.0, 0.0),
                    ("X / Twitter Post", 1200.0, 675.0, 0.0),
                    ("YouTube Thumbnail", 1280.0, 720.0, 0.0),
                ];
                for (name, w, h, r) in media_presets {
                    if ui.button(format!("🌐 {} ({}×{})", name, w as i32, h as i32)).clicked() {
                        let pos = doc.get_next_frame_position(center, w);
                        let id = doc.add_frame_preset_styled(name, w, h, pos, r);
                        state.selected_ids = vec![id];
                        state.active_tool = ActiveTool::Select;
                        history.push_state(doc);
                        network.send_selection(&state.selected_ids);
                        network.send_doc_update(doc);
                    }
                }
            });

        // UI Components Quick Insert
        egui::ComboBox::from_id_salt("components_library_combo")
            .selected_text("📦 Components")
            .show_ui(ui, |ui| {
                let center = sigma_core::Vec2::new(-state.pan.x + 350.0, -state.pan.y + 300.0);
                if ui.button("+ Primary Button").clicked() {
                    let id = doc.insert_component("button", center);
                    state.selected_ids = vec![id];
                    history.push_state(doc);
                    network.send_doc_update(doc);
                }
                if ui.button("+ Glassmorphic Card").clicked() {
                    let id = doc.insert_component("card", center);
                    state.selected_ids = vec![id];
                    history.push_state(doc);
                    network.send_doc_update(doc);
                }
                if ui.button("+ Text Input Field").clicked() {
                    let id = doc.insert_component("input", center);
                    state.selected_ids = vec![id];
                    history.push_state(doc);
                    network.send_doc_update(doc);
                }
                if ui.button("+ User Avatar").clicked() {
                    let id = doc.insert_component("avatar", center);
                    state.selected_ids = vec![id];
                    history.push_state(doc);
                    network.send_doc_update(doc);
                }
            });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

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

            // Better Auth: Sign In or User Profile Button
            if state.auth_is_signed_in {
                let user_pill = egui::Button::new(
                    RichText::new(format!("👤 {} ({})", state.auth_user_name, state.auth_role))
                        .size(11.0)
                        .strong()
                        .color(Color32::from_rgb(10, 132, 255)),
                );
                if ui
                    .add(user_pill)
                    .on_hover_text("Better Auth: Verified Session. Click to manage profile or sign out.")
                    .clicked()
                {
                    state.show_auth_modal = true;
                }
            } else {
                let signin_btn = egui::Button::new(
                    RichText::new("🔐 Sign In")
                        .size(11.5)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(Color32::from_rgb(10, 132, 255));
                if ui
                    .add(signin_btn)
                    .on_hover_text("Sign In with Better Auth / Email / Google / GitHub")
                    .clicked()
                {
                    state.show_auth_modal = true;
                }
            }

            ui.separator();

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

            // Debugger Toggle CTA
            let dbg_active = state.show_debugger;
            let dbg_btn = egui::Button::new(
                RichText::new(if dbg_active { "🐞 Debug (On)" } else { "🐞 Debug" })
                    .size(11.5)
                    .color(if dbg_active {
                        Color32::from_rgb(255, 190, 0)
                    } else {
                        Color32::from_rgb(175, 175, 190)
                    }),
            );
            if ui
                .add(dbg_btn)
                .on_hover_text("Toggle Engine Diagnostics & Memory HUD")
                .clicked()
            {
                state.show_debugger = !state.show_debugger;
            }

            ui.separator();

            // Multiplayer Peer Avatars (Figma style)
            // 1. Local user avatar bubble
            let local_c = Color32::from_rgb(
                network.user_color.r,
                network.user_color.g,
                network.user_color.b,
            );
            if render_avatar_bubble(ui, &network.user_name, local_c, true) {
                state.show_auth_modal = true;
            }

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

fn render_avatar_bubble(ui: &mut Ui, name: &str, color: Color32, is_self: bool) -> bool {
    let initial = name.chars().next().unwrap_or('?').to_uppercase().to_string();
    let (rect, resp) = ui.allocate_exact_size(EguiVec2::splat(24.0), egui::Sense::click());
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
        format!("You ({}) - Click to manage Better Auth account", name)
    } else {
        name.to_string()
    };
    resp.on_hover_text(hover_text).clicked()
}
