use crate::canvas::CanvasState;
use crate::network::NetworkManager;
use egui::{Color32, RichText, ScrollArea, Ui};
use sigma_core::{Document, Node, NodeId, NodeKind};

pub fn render_sidebar(
    ui: &mut Ui,
    doc: &mut Document,
    state: &mut CanvasState,
    network: &NetworkManager,
) {
    ui.vertical(|ui| {
        // Clean Minimal Tabs: Layers & Templates (No duplicate app header)
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            let layer_count = doc.nodes.len();
            let is_layers = state.active_sidebar_tab == 0;
            let is_assets = state.active_sidebar_tab == 1;

            let layers_btn = egui::Button::new(
                RichText::new(format!("Layers ({})", layer_count))
                    .size(11.5)
                    .strong()
                    .color(if is_layers {
                        Color32::WHITE
                    } else {
                        Color32::from_rgb(148, 163, 184)
                    }),
            )
            .fill(if is_layers {
                Color32::from_rgb(39, 39, 44)
            } else {
                Color32::TRANSPARENT
            })
            .stroke(egui::Stroke::NONE)
            .corner_radius(egui::CornerRadius::same(5));

            if ui.add(layers_btn).clicked() {
                state.active_sidebar_tab = 0;
            }

            let assets_btn = egui::Button::new(
                RichText::new("Templates")
                    .size(11.5)
                    .strong()
                    .color(if is_assets {
                        Color32::WHITE
                    } else {
                        Color32::from_rgb(148, 163, 184)
                    }),
            )
            .fill(if is_assets {
                Color32::from_rgb(39, 39, 44)
            } else {
                Color32::TRANSPARENT
            })
            .stroke(egui::Stroke::NONE)
            .corner_radius(egui::CornerRadius::same(5));

            if ui.add(assets_btn).clicked() {
                state.active_sidebar_tab = 1;
            }
        });

        ui.add_space(6.0);
        ui.separator();

        if state.active_sidebar_tab == 1 {
            render_ui_kits_tab(ui, doc, state, network);
            return;
        }

        // Layer Action Bar (Reorder, Group, Duplicate, Delete)
        if !state.selected_ids.is_empty() {
            ui.horizontal(|ui| {
                ui.add_space(2.0);
                if ui.small_button("⤒").on_hover_text("Bring to Front (Shift+])").clicked() {
                    doc.bring_to_front(&state.selected_ids);
                    network.send_doc_update(doc);
                }
                if ui.small_button("▲").on_hover_text("Bring Forward (])").clicked() {
                    doc.bring_forward(&state.selected_ids);
                    network.send_doc_update(doc);
                }
                if ui.small_button("▼").on_hover_text("Send Backward ([)").clicked() {
                    doc.send_backward(&state.selected_ids);
                    network.send_doc_update(doc);
                }
                if ui.small_button("⤓").on_hover_text("Send to Back (Shift+[)").clicked() {
                    doc.send_to_back(&state.selected_ids);
                    network.send_doc_update(doc);
                }

                ui.separator();

                if state.selected_ids.len() > 1 {
                    if ui.small_button("⊞").on_hover_text("Group Selection (Cmd+G)").clicked() {
                        if let Some(gid) = doc.group_selection(&state.selected_ids) {
                            state.selected_ids = vec![gid];
                            network.send_selection(&state.selected_ids);
                            network.send_doc_update(doc);
                        }
                    }
                } else if state.selected_ids.len() == 1 {
                    let first_id = state.selected_ids[0];
                    if doc.find_parent_frame_id(first_id).is_some() {
                        if ui.small_button("↖").on_hover_text("Extract out of Frame").clicked() {
                            doc.extract_from_parent(first_id);
                            network.send_doc_update(doc);
                        }
                    }
                    if ui.small_button("⊟").on_hover_text("Ungroup (Cmd+Shift+G)").clicked() {
                        let unpacked = doc.ungroup_selection(&state.selected_ids);
                        if !unpacked.is_empty() {
                            state.selected_ids = unpacked;
                            network.send_selection(&state.selected_ids);
                            network.send_doc_update(doc);
                        }
                    }
                }

                if ui.small_button("📋").on_hover_text("Duplicate (Cmd+D)").clicked() {
                    if let Some(&first_id) = state.selected_ids.first() {
                        if let Some(new_id) = doc.duplicate_node(first_id) {
                            state.selected_ids = vec![new_id];
                            network.send_selection(&state.selected_ids);
                            network.send_doc_update(doc);
                        }
                    }
                }

                if ui.small_button("🗑").on_hover_text("Delete (Backspace/Del)").clicked() {
                    for id in state.selected_ids.clone() {
                        doc.remove_node(id);
                    }
                    state.selected_ids.clear();
                    network.send_selection(&[]);
                    network.send_doc_update(doc);
                }
            });
            ui.separator();
        }

        // Layer Tree View (Scrollable)
        ScrollArea::vertical().show(ui, |ui| {
            let mut action_select = None;
            let mut action_toggle_vis = None;
            let mut action_toggle_lock = None;
            let mut action_rename_commit = None;
            let mut action_move_up = None;
            let mut action_move_down = None;
            let mut action_extract_parent = None;

            // Iterate in reverse render order (top-most layer on top of list)
            for node in doc.nodes.iter().rev() {
                render_node_row(
                    ui,
                    node,
                    state,
                    0,
                    &mut action_select,
                    &mut action_toggle_vis,
                    &mut action_toggle_lock,
                    &mut action_rename_commit,
                    &mut action_move_up,
                    &mut action_move_down,
                    &mut action_extract_parent,
                );
            }

            if let Some(id) = action_select {
                if ui.input(|i| i.modifiers.shift) {
                    if state.selected_ids.contains(&id) {
                        state.selected_ids.retain(|&x| x != id);
                    } else {
                        state.selected_ids.push(id);
                    }
                } else {
                    state.selected_ids = vec![id];
                }
                network.send_selection(&state.selected_ids);
            }

            if let Some(id) = action_move_up {
                doc.move_node_up(id);
                network.send_doc_update(doc);
            }

            if let Some(id) = action_move_down {
                doc.move_node_down(id);
                network.send_doc_update(doc);
            }

            if let Some(id) = action_extract_parent {
                doc.extract_from_parent(id);
                network.send_doc_update(doc);
            }

            if let Some(id) = action_toggle_vis {
                let update_info = if let Some(node) = doc.find_node_mut(id) {
                    node.visible = !node.visible;
                    Some((node.visible, node.id, node.rect))
                } else {
                    None
                };
                if let Some((visible, nid, rect)) = update_info {
                    if visible {
                        doc.spatial_index.insert(nid, rect);
                    } else {
                        doc.spatial_index.remove(nid);
                    }
                    network.send_doc_update(doc);
                }
            }

            if let Some(id) = action_toggle_lock {
                if let Some(node) = doc.find_node_mut(id) {
                    node.locked = !node.locked;
                    network.send_doc_update(doc);
                }
            }

            if let Some((id, new_name)) = action_rename_commit {
                if let Some(node) = doc.find_node_mut(id) {
                    node.name = new_name;
                    network.send_doc_update(doc);
                }
                state.renaming_layer_id = None;
            }
        });
    });
}

fn render_node_row(
    ui: &mut Ui,
    node: &Node,
    state: &mut CanvasState,
    depth: usize,
    action_select: &mut Option<NodeId>,
    action_toggle_vis: &mut Option<NodeId>,
    action_toggle_lock: &mut Option<NodeId>,
    action_rename_commit: &mut Option<(NodeId, String)>,
    action_move_up: &mut Option<NodeId>,
    action_move_down: &mut Option<NodeId>,
    action_extract_parent: &mut Option<NodeId>,
) {
    let is_selected = state.selected_ids.contains(&node.id);
    let has_children = !node.children.is_empty();
    let is_expanded = state.expanded_layer_ids.contains(&node.id);

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
        // Indentation for nested children
        if depth > 0 {
            ui.add_space((depth as f32) * 12.0);
        }

        // Expand / Collapse disclosure triangle for frames with children
        if has_children {
            let arrow = if is_expanded { "▼" } else { "▶" };
            if ui.small_button(arrow).clicked() {
                if is_expanded {
                    state.expanded_layer_ids.remove(&node.id);
                } else {
                    state.expanded_layer_ids.insert(node.id);
                }
            }
        } else {
            ui.add_space(14.0);
        }

        // Layer Kind Icon
        ui.label(
            RichText::new(kind_icon)
                .size(11.0)
                .color(if is_selected {
                    Color32::from_rgb(10, 132, 255)
                } else {
                    Color32::from_rgb(150, 150, 160)
                }),
        );

        // Layer Name / Inline Renaming Box
        if state.renaming_layer_id == Some(node.id) {
            let edit_resp = ui.add(
                egui::TextEdit::singleline(&mut state.renaming_text)
                    .desired_width(90.0),
            );
            if edit_resp.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                *action_rename_commit = Some((node.id, state.renaming_text.clone()));
            }
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                state.renaming_layer_id = None;
            }
        } else {
            let text_color = if is_selected {
                Color32::from_rgb(10, 132, 255)
            } else if !node.visible {
                Color32::from_rgb(110, 110, 120)
            } else {
                Color32::from_rgb(220, 220, 225)
            };

            let label_resp = ui.selectable_label(
                is_selected,
                RichText::new(&node.name).color(text_color).size(12.0),
            );

            if label_resp.clicked() {
                *action_select = Some(node.id);
            }
            if label_resp.double_clicked() {
                state.renaming_layer_id = Some(node.id);
                state.renaming_text = node.name.clone();
            }
        }

        // Right-aligned status buttons: Up/Down Reorder, Extract, Lock, Visibility
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Visibility toggle
            let vis_icon = if node.visible { "👁" } else { "⊘" };
            let vis_color = if node.visible {
                Color32::from_rgb(140, 140, 150)
            } else {
                Color32::from_rgb(255, 90, 90)
            };
            if ui.add(egui::Button::new(RichText::new(vis_icon).size(10.0).color(vis_color)).frame(false)).clicked() {
                *action_toggle_vis = Some(node.id);
            }

            // Lock toggle
            let lock_icon = if node.locked { "🔒" } else { "🔓" };
            let lock_color = if node.locked {
                Color32::from_rgb(255, 180, 0)
            } else {
                Color32::from_rgb(80, 80, 90)
            };
            if ui.add(egui::Button::new(RichText::new(lock_icon).size(10.0).color(lock_color)).frame(false)).clicked() {
                *action_toggle_lock = Some(node.id);
            }

            // Move Down (backward)
            if ui.add(egui::Button::new(RichText::new("▼").size(9.0).color(Color32::from_rgb(120, 120, 130))).frame(false))
                .on_hover_text("Move Down in layer order ([)")
                .clicked()
            {
                *action_move_down = Some(node.id);
            }

            // Move Up (forward)
            if ui.add(egui::Button::new(RichText::new("▲").size(9.0).color(Color32::from_rgb(120, 120, 130))).frame(false))
                .on_hover_text("Move Up in layer order (])")
                .clicked()
            {
                *action_move_up = Some(node.id);
            }

            // If nested inside a frame, show extract button
            if depth > 0 {
                if ui.add(egui::Button::new(RichText::new("↖").size(9.0).color(Color32::from_rgb(10, 132, 255))).frame(false))
                    .on_hover_text("Extract out of Frame to Root")
                    .clicked()
                {
                    *action_extract_parent = Some(node.id);
                }
            }
        });
    });

    // If frame is expanded, recursively render children!
    if has_children && is_expanded {
        for child in node.children.iter().rev() {
            render_node_row(
                ui,
                child,
                state,
                depth + 1,
                action_select,
                action_toggle_vis,
                action_toggle_lock,
                action_rename_commit,
                action_move_up,
                action_move_down,
                action_extract_parent,
            );
        }
    }
}

fn render_ui_kits_tab(
    ui: &mut Ui,
    doc: &mut Document,
    state: &mut CanvasState,
    network: &NetworkManager,
) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(8.0);
        ui.label(
            RichText::new("PRODUCTION UI KITS")
                .size(11.0)
                .strong()
                .color(Color32::from_rgb(140, 140, 150)),
        );
        ui.label(
            RichText::new("1-click insert pre-built, responsive vector templates & components.")
                .size(10.0)
                .color(Color32::from_rgb(110, 110, 120)),
        );
        ui.add_space(10.0);

        let center = sigma_core::Vec2::new(-state.pan.x + 400.0, -state.pan.y + 350.0);

        let kits = [
            (
                "📱 iOS 18 Fintech App",
                "iPhone 16 Pro (393×852) with Dynamic Island, portfolio cards & transactions.",
                0,
            ),
            (
                "💻 SaaS Landing Hero",
                "Desktop 1440 (1200×720) with navbar, headlines, CTA dock & stats preview.",
                1,
            ),
            (
                "💳 Glassmorphic Pricing Card",
                "Dark glass card (360×540) with feature checklist, popular badge & CTA.",
                2,
            ),
            (
                "🔐 Modern Auth Modal",
                "Sign In dialog (400×480) with Google, GitHub OAuth & email fields.",
                3,
            ),
            (
                "🔘 UI Button System",
                "Complete button component kit with primary, secondary, outline, pill states.",
                4,
            ),
            (
                "🏷️ Status Badges & Chips",
                "Status tags, pill badges (Active, In Progress, Enterprise, Featured).",
                5,
            ),
        ];

        for (title, desc, kind) in kits {
            egui::Frame::new()
                .fill(Color32::from_rgb(26, 26, 30))
                .stroke(egui::Stroke::new(1.0, Color32::from_rgb(45, 45, 52)))
                .corner_radius(egui::CornerRadius::same(8))
                .inner_margin(egui::Margin::same(10))
                .show(ui, |ui| {
                    ui.label(RichText::new(title).size(12.0).strong().color(Color32::WHITE));
                    ui.add_space(2.0);
                    ui.label(RichText::new(desc).size(10.0).color(Color32::from_rgb(140, 140, 150)));
                    ui.add_space(6.0);

                    if ui.button(RichText::new("+ Insert to Canvas").size(11.0).color(Color32::from_rgb(10, 132, 255))).clicked() {
                        let id = match kind {
                            0 => {
                                let pos = doc.get_next_frame_position(center, 393.0);
                                doc.insert_fintech_mobile(pos)
                            }
                            1 => {
                                let pos = doc.get_next_frame_position(center, 1200.0);
                                doc.insert_saas_hero(pos)
                            }
                            2 => {
                                let pos = doc.get_next_frame_position(center, 360.0);
                                doc.insert_pricing_card(pos)
                            }
                            3 => {
                                let pos = doc.get_next_frame_position(center, 400.0);
                                doc.insert_auth_card(pos)
                            }
                            4 => {
                                let pos = doc.get_next_frame_position(center, 440.0);
                                doc.insert_button_kit(pos)
                            }
                            5 => {
                                let pos = doc.get_next_frame_position(center, 440.0);
                                doc.insert_badge_kit(pos)
                            }
                            _ => doc.alloc_id(),
                        };
                        state.selected_ids = vec![id];
                        state.active_tool = crate::canvas::ActiveTool::Select;
                        network.send_selection(&state.selected_ids);
                        network.send_doc_update(doc);
                    }
                });
            ui.add_space(6.0);
        }
    });
}

