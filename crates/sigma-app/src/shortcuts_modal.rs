use crate::canvas::CanvasState;
use egui::{Color32, RichText, ScrollArea, Stroke, Ui};

pub fn render_shortcuts_modal(ctx: &egui::Context, state: &mut CanvasState) {
    if !state.show_shortcuts_modal {
        return;
    }

    let mut open = state.show_shortcuts_modal;
    let screen_rect = ctx.input(|i| i.viewport_rect());
    let modal_width = (screen_rect.width() * 0.75).clamp(480.0, 680.0);
    let modal_height = (screen_rect.height() * 0.75).clamp(380.0, 560.0);

    egui::Window::new("keyboard_shortcuts_window")
        .open(&mut open)
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .fixed_size([modal_width, modal_height])
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(
            egui::Frame::new()
                .fill(Color32::from_rgb(18, 19, 24))
                .stroke(Stroke::new(1.0, Color32::from_rgb(44, 46, 58)))
                .corner_radius(egui::CornerRadius::same(14))
                .inner_margin(egui::Margin::same(18))
                .shadow(egui::epaint::Shadow {
                    offset: [0, 16],
                    blur: 36,
                    spread: 0,
                    color: Color32::from_black_alpha(190),
                }),
        )
        .show(ctx, |ui| {
            // Header
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("⌨ KEYBOARD SHORTCUTS")
                        .size(14.0)
                        .strong()
                        .color(Color32::from_rgb(245, 245, 247)),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(RichText::new("✕").size(14.0).color(Color32::from_rgb(160, 160, 175))).clicked() {
                        state.show_shortcuts_modal = false;
                    }
                });
            });

            ui.add_space(8.0);

            // Search Bar
            ui.horizontal(|ui| {
                ui.label(RichText::new("🔍").size(13.0).color(Color32::from_rgb(130, 130, 140)));
                ui.add(
                    egui::TextEdit::singleline(&mut state.shortcuts_search)
                        .hint_text("Search shortcuts (e.g. frame, copy, zoom, group)...")
                        .desired_width(ui.available_width()),
                );
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(6.0);

            let search = state.shortcuts_search.to_lowercase();

            ScrollArea::vertical().show(ui, |ui| {
                let sections = [
                    (
                        "TOOLS & DRAWING",
                        vec![
                            ("Move / Select Tool", "V"),
                            ("Frame / Artboard", "F"),
                            ("Rectangle", "R"),
                            ("Ellipse / Circle", "O"),
                            ("Line", "L"),
                            ("Arrow", "A"),
                            ("Star", "S"),
                            ("Polygon", "P"),
                            ("Text Tool", "T"),
                            ("Hand Tool (Pan)", "H  or  Space + Drag"),
                        ],
                    ),
                    (
                        "EDITING & CLIPBOARD",
                        vec![
                            ("Undo", "⌘ Z"),
                            ("Redo", "⇧ ⌘ Z"),
                            ("Copy", "⌘ C"),
                            ("Paste (at cursor / +20px)", "⌘ V"),
                            ("Cut", "⌘ X"),
                            ("Duplicate Selection", "⌘ D"),
                            ("Delete Selection", "⌫  or  Del"),
                            ("Select All", "⌘ A"),
                            ("Deselect All", "Esc"),
                        ],
                    ),
                    (
                        "ARRANGE & HIERARCHY",
                        vec![
                            ("Bring Forward", "]"),
                            ("Send Backward", "["),
                            ("Bring to Front", "⇧ ]"),
                            ("Send to Back", "⇧ ["),
                            ("Group Selection", "⌘ G"),
                            ("Ungroup Selection", "⇧ ⌘ G"),
                            ("Frame Selection", "⌥ ⌘ G"),
                            ("Lock / Unlock Layer", "⇧ ⌘ L"),
                            ("Hide / Show Layer", "⇧ ⌘ H"),
                            ("Nudge 1px / 10px", "Arrows  /  ⇧ + Arrows"),
                        ],
                    ),
                    (
                        "VIEW & PRESENTATION",
                        vec![
                            ("Zoom In", "⌘ +"),
                            ("Zoom Out", "⌘ -"),
                            ("Zoom 100%", "⌘ 0"),
                            ("Fit All to Screen", "⇧ 1"),
                            ("Toggle Dot Grid", "G"),
                            ("Prototype Presentation Mode", "⌥ P  or  ▶ Present"),
                            ("Exit Presentation Mode", "Esc"),
                            ("Show Shortcuts Cheat Sheet", "?  or  ⌘ /"),
                        ],
                    ),
                ];

                for (cat_name, items) in sections {
                    let matching_items: Vec<_> = items
                        .into_iter()
                        .filter(|(label, sc)| {
                            search.is_empty()
                                || label.to_lowercase().contains(&search)
                                || sc.to_lowercase().contains(&search)
                        })
                        .collect();

                    if matching_items.is_empty() {
                        continue;
                    }

                    ui.add_space(6.0);
                    ui.label(
                        RichText::new(cat_name)
                            .size(11.0)
                            .strong()
                            .color(Color32::from_rgb(10, 132, 255)),
                    );
                    ui.add_space(4.0);

                    for (label, sc) in matching_items {
                        render_shortcut_row(ui, label, sc);
                    }
                }
            });
        });

    if !open {
        state.show_shortcuts_modal = false;
    }
}

fn render_shortcut_row(ui: &mut Ui, label: &str, keys: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).size(12.5).color(Color32::from_rgb(220, 220, 228)));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Render individual keycap badges
            let parts: Vec<&str> = keys.split("  ").collect();
            for part in parts.into_iter().rev() {
                egui::Frame::new()
                    .fill(Color32::from_rgb(28, 30, 38))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(48, 50, 62)))
                    .corner_radius(egui::CornerRadius::same(5))
                    .inner_margin(egui::Margin::symmetric(6, 2))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(part)
                                .size(11.0)
                                .monospace()
                                .strong()
                                .color(Color32::from_rgb(240, 240, 245)),
                        );
                    });
            }
        });
    });
    ui.add_space(2.0);
}
