use crate::canvas::CanvasState;
use egui::{Color32, RichText, ScrollArea, Ui};
use sigma_core::{Document, NodeKind};

pub fn render_sidebar(ui: &mut Ui, doc: &mut Document, state: &mut CanvasState) {
    ui.vertical(|ui| {
        // App header
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.add_space(10.0);
            ui.label(
                RichText::new("Σ")
                    .size(20.0)
                    .strong()
                    .color(Color32::from_rgb(10, 132, 255)),
            );
            ui.label(
                RichText::new("Sigma")
                    .size(16.0)
                    .strong()
                    .color(Color32::WHITE),
            );
            ui.label(
                RichText::new("PRO")
                    .size(10.0)
                    .color(Color32::from_rgb(255, 180, 0)),
            );
        });

        ui.add_space(8.0);
        ui.separator();

        // Pages Section
        ui.collapsing(RichText::new("Pages").strong().size(12.0), |ui| {
            ui.horizontal(|ui| {
                ui.label("• Page 1 (Design)");
            });
        });

        ui.separator();

        // Layers Section
        ui.horizontal(|ui| {
            ui.label(RichText::new("LAYERS").strong().size(11.0).color(Color32::from_rgb(140, 140, 150)));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if !state.selected_ids.is_empty() {
                    if ui.small_button("🗑").on_hover_text("Delete selected (Del)").clicked() {
                        for id in state.selected_ids.clone() {
                            doc.remove_node(id);
                        }
                        state.selected_ids.clear();
                    }
                    if state.selected_ids.len() > 1 {
                        if ui.small_button("⊞ Group").on_hover_text("Group selection (Cmd+G)").clicked() {
                            if let Some(group_id) = doc.group_selection(&state.selected_ids) {
                                state.selected_ids = vec![group_id];
                            }
                        }
                    }
                    if state.selected_ids.len() == 1 {
                        if ui.small_button("⊟ Ungroup").on_hover_text("Ungroup (Cmd+Shift+G)").clicked() {
                            let unpacked = doc.ungroup_selection(&state.selected_ids);
                            if !unpacked.is_empty() {
                                state.selected_ids = unpacked;
                            }
                        }
                    }
                }
            });
        });

        ui.add_space(4.0);

        ScrollArea::vertical().show(ui, |ui| {
            let mut to_select = None;
            let mut to_toggle_vis = None;
            let mut to_toggle_lock = None;
            let mut to_move_up = None;
            let mut to_move_down = None;

            for (idx, node) in doc.nodes.iter().enumerate().rev() {
                let is_selected = state.selected_ids.contains(&node.id);
                let kind_icon = match node.kind {
                    NodeKind::Frame => "⊞",
                    NodeKind::Rectangle => "▭",
                    NodeKind::Ellipse => "○",
                    NodeKind::Text => "T",
                    NodeKind::Line => "—",
                    NodeKind::Arrow => "→",
                    NodeKind::Star => "★",
                    NodeKind::Polygon => "△",
                };

                ui.horizontal(|ui| {
                    let text_color = if is_selected {
                        Color32::from_rgb(10, 132, 255)
                    } else if !node.visible {
                        Color32::from_rgb(100, 100, 100)
                    } else {
                        Color32::from_rgb(230, 230, 230)
                    };

                    let label_text = format!("{}  {}", kind_icon, node.name);
                    let resp = ui.selectable_label(
                        is_selected,
                        RichText::new(label_text).color(text_color).size(12.0),
                    );

                    if resp.clicked() {
                        to_select = Some(node.id);
                    }

                    // Visibility toggle
                    let vis_icon = if node.visible { "👁" } else { "⊘" };
                    if ui.small_button(vis_icon).clicked() {
                        to_toggle_vis = Some(node.id);
                    }

                    // Lock toggle
                    let lock_icon = if node.locked { "🔒" } else { "🔓" };
                    if ui.small_button(lock_icon).clicked() {
                        to_toggle_lock = Some(node.id);
                    }

                    // Layer Reordering
                    if ui.small_button("▲").clicked() {
                        to_move_up = Some(idx);
                    }
                    if ui.small_button("▼").clicked() {
                        to_move_down = Some(idx);
                    }
                });
            }

            if let Some(id) = to_select {
                state.selected_ids = vec![id];
            }
            if let Some(id) = to_toggle_vis {
                if let Some(node) = doc.find_node_mut(id) {
                    node.visible = !node.visible;
                }
            }
            if let Some(id) = to_toggle_lock {
                if let Some(node) = doc.find_node_mut(id) {
                    node.locked = !node.locked;
                }
            }
            if let Some(idx) = to_move_up {
                if idx + 1 < doc.nodes.len() {
                    doc.nodes.swap(idx, idx + 1);
                }
            }
            if let Some(idx) = to_move_down {
                if idx > 0 {
                    doc.nodes.swap(idx, idx - 1);
                }
            }
        });
    });
}
