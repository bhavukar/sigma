use crate::canvas::{ActiveTool, CanvasState};
use crate::network::NetworkManager;
use egui::{Color32, Pos2, Rect as EguiRect, Stroke, Ui, Vec2 as EguiVec2};
use sigma_core::{export_node_to_svg, export_to_css, Document, History};

pub fn render_context_menu(
    ctx: &egui::Context,
    doc: &mut Document,
    state: &mut CanvasState,
    history: &mut History,
    network: &NetworkManager,
) {
    let menu_pos = match state.context_menu_pos {
        Some(pos) => pos,
        None => return,
    };

    let screen_rect = ctx.input(|i| i.viewport_rect());
    let menu_width = 240.0;
    // Estimate menu height depending on whether selection exists
    let has_selection = !state.selected_ids.is_empty();
    let estimated_height = if has_selection { 480.0 } else { 240.0 };

    // Clamp position within screen bounds
    let clamped_x = (menu_pos.x).min(screen_rect.max.x - menu_width - 10.0).max(screen_rect.min.x + 10.0);
    let clamped_y = (menu_pos.y).min(screen_rect.max.y - estimated_height - 10.0).max(screen_rect.min.y + 10.0);
    let target_pos = Pos2::new(clamped_x, clamped_y);

    let mut close_menu = false;
    let mut action_taken = false;

    // Check click outside to dismiss
    if ctx.input(|i| i.pointer.primary_clicked() || i.pointer.secondary_clicked()) {
        if let Some(ptr_pos) = ctx.input(|i| i.pointer.hover_pos()) {
            let rough_rect = EguiRect::from_min_size(target_pos, EguiVec2::new(menu_width, estimated_height));
            if !rough_rect.contains(ptr_pos) {
                close_menu = true;
            }
        }
    }

    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        close_menu = true;
    }

    egui::Area::new(egui::Id::new("sigma_context_menu"))
        .fixed_pos(target_pos)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(Color32::from_rgb(20, 20, 24))
                .stroke(Stroke::new(1.0, Color32::from_rgb(44, 46, 56)))
                .corner_radius(egui::CornerRadius::same(10))
                .inner_margin(egui::Margin::symmetric(6, 6))
                .shadow(egui::epaint::Shadow {
                    offset: [0, 8],
                    blur: 24,
                    spread: 0,
                    color: Color32::from_black_alpha(160),
                })
                .show(ui, |ui| {
                    ui.set_width(menu_width);

                    if has_selection {
                        // Section 1: Clipboard & Edit
                        if render_menu_item(ui, "✂", "Cut", "⌘X") {
                            state.clipboard = doc.clone_nodes_for_clipboard(&state.selected_ids);
                            history.push_state(doc);
                            for id in state.selected_ids.clone() {
                                doc.remove_node(id);
                            }
                            state.selected_ids.clear();
                            network.send_selection(&[]);
                            network.send_doc_update(doc);
                            action_taken = true;
                        }

                        if render_menu_item(ui, "📋", "Copy", "⌘C") {
                            state.clipboard = doc.clone_nodes_for_clipboard(&state.selected_ids);
                            action_taken = true;
                        }

                        let can_paste = !state.clipboard.is_empty();
                        if ui.add_enabled_ui(can_paste, |ui| {
                            render_menu_item(ui, "📄", "Paste Here", "⌘V")
                        }).inner && can_paste {
                            let world_pos = state.screen_to_world(menu_pos, screen_rect);
                            history.push_state(doc);
                            let new_ids = doc.paste_nodes(&state.clipboard, Some(world_pos));
                            if !new_ids.is_empty() {
                                state.selected_ids = new_ids;
                                network.send_selection(&state.selected_ids);
                                network.send_doc_update(doc);
                            }
                            action_taken = true;
                        }

                        if render_menu_item(ui, "📑", "Duplicate", "⌘D") {
                            if let Some(&first_id) = state.selected_ids.first() {
                                history.push_state(doc);
                                if let Some(new_id) = doc.duplicate_node(first_id) {
                                    state.selected_ids = vec![new_id];
                                    network.send_selection(&state.selected_ids);
                                    network.send_doc_update(doc);
                                }
                            }
                            action_taken = true;
                        }

                        if render_menu_item(ui, "🗑", "Delete", "⌫") {
                            history.push_state(doc);
                            for id in state.selected_ids.clone() {
                                doc.remove_node(id);
                            }
                            state.selected_ids.clear();
                            network.send_selection(&[]);
                            network.send_doc_update(doc);
                            action_taken = true;
                        }

                        render_divider(ui);

                        // Section 2: Arrange / Z-Order
                        if render_menu_item(ui, "⤒", "Bring to Front", "⇧]") {
                            history.push_state(doc);
                            doc.bring_to_front(&state.selected_ids);
                            network.send_doc_update(doc);
                            action_taken = true;
                        }
                        if render_menu_item(ui, "▲", "Bring Forward", "]") {
                            history.push_state(doc);
                            doc.bring_forward(&state.selected_ids);
                            network.send_doc_update(doc);
                            action_taken = true;
                        }
                        if render_menu_item(ui, "▼", "Send Backward", "[") {
                            history.push_state(doc);
                            doc.send_backward(&state.selected_ids);
                            network.send_doc_update(doc);
                            action_taken = true;
                        }
                        if render_menu_item(ui, "⤓", "Send to Back", "⇧[") {
                            history.push_state(doc);
                            doc.send_to_back(&state.selected_ids);
                            network.send_doc_update(doc);
                            action_taken = true;
                        }

                        render_divider(ui);

                        // Section 3: Grouping & Framing
                        if state.selected_ids.len() > 1 {
                            if render_menu_item(ui, "⊞", "Group Selection", "⌘G") {
                                history.push_state(doc);
                                if let Some(gid) = doc.group_selection(&state.selected_ids) {
                                    state.selected_ids = vec![gid];
                                    network.send_selection(&state.selected_ids);
                                    network.send_doc_update(doc);
                                }
                                action_taken = true;
                            }
                        } else if state.selected_ids.len() == 1 {
                            if render_menu_item(ui, "⊟", "Ungroup", "⇧⌘G") {
                                history.push_state(doc);
                                let unpacked = doc.ungroup_selection(&state.selected_ids);
                                if !unpacked.is_empty() {
                                    state.selected_ids = unpacked;
                                    network.send_selection(&state.selected_ids);
                                    network.send_doc_update(doc);
                                }
                                action_taken = true;
                            }
                        }

                        if render_menu_item(ui, "📐", "Frame Selection", "⌥⌘G") {
                            history.push_state(doc);
                            if let Some(fid) = doc.frame_selection(&state.selected_ids) {
                                state.selected_ids = vec![fid];
                                network.send_selection(&state.selected_ids);
                                network.send_doc_update(doc);
                            }
                            action_taken = true;
                        }

                        if render_menu_item(ui, "⚡", "Add Auto Layout", "⇧A") {
                            history.push_state(doc);
                            let mut handled = false;
                            if state.selected_ids.len() == 1 {
                                let id = state.selected_ids[0];
                                if let Some(node) = doc.find_node_mut(id) {
                                    if node.kind == sigma_core::NodeKind::Frame {
                                        node.auto_layout.enabled = true;
                                        node.apply_auto_layout();
                                        network.send_doc_update(doc);
                                        handled = true;
                                    }
                                }
                            }
                            if !handled {
                                if let Some(fid) = doc.frame_selection(&state.selected_ids) {
                                    if let Some(frame) = doc.find_node_mut(fid) {
                                        frame.name = "Auto Layout".to_string();
                                        frame.auto_layout.enabled = true;
                                        frame.auto_layout.direction = sigma_core::LayoutDirection::Vertical;
                                        frame.auto_layout.gap = 12.0;
                                        frame.auto_layout.padding = sigma_core::Padding::all(16.0);
                                        frame.apply_auto_layout();
                                    }
                                    state.selected_ids = vec![fid];
                                    network.send_selection(&state.selected_ids);
                                    network.send_doc_update(doc);
                                }
                            }
                            action_taken = true;
                        }

                        if render_menu_item(ui, "🔒", "Lock / Unlock", "⇧⌘L") {
                            history.push_state(doc);
                            doc.toggle_lock_selection(&state.selected_ids);
                            network.send_doc_update(doc);
                            action_taken = true;
                        }

                        if render_menu_item(ui, "👁", "Hide / Show", "⇧⌘H") {
                            history.push_state(doc);
                            doc.toggle_vis_selection(&state.selected_ids);
                            network.send_doc_update(doc);
                            action_taken = true;
                        }

                        render_divider(ui);

                        // Section 4: Copy Code / Specs
                        if render_menu_item(ui, "⚡", "Copy as SVG", "") {
                            if let Some(&first_id) = state.selected_ids.first() {
                                if let Some(node) = doc.find_node(first_id) {
                                    let svg = export_node_to_svg(node);
                                    ctx.copy_text(svg);
                                }
                            }
                            action_taken = true;
                        }

                        if render_menu_item(ui, "🎨", "Copy as CSS", "") {
                            if let Some(&first_id) = state.selected_ids.first() {
                                if let Some(node) = doc.find_node(first_id) {
                                    let css = export_to_css(node);
                                    ctx.copy_text(css);
                                }
                            }
                            action_taken = true;
                        }
                    } else {
                        // Canvas empty right-click options
                        let can_paste = !state.clipboard.is_empty();
                        if ui.add_enabled_ui(can_paste, |ui| {
                            render_menu_item(ui, "📄", "Paste Here", "⌘V")
                        }).inner && can_paste {
                            let world_pos = state.screen_to_world(menu_pos, screen_rect);
                            history.push_state(doc);
                            let new_ids = doc.paste_nodes(&state.clipboard, Some(world_pos));
                            if !new_ids.is_empty() {
                                state.selected_ids = new_ids;
                                network.send_selection(&state.selected_ids);
                                network.send_doc_update(doc);
                            }
                            action_taken = true;
                        }

                        render_divider(ui);

                        if render_menu_item(ui, "⊞", "New Frame", "F") {
                            state.active_tool = ActiveTool::Frame;
                            action_taken = true;
                        }
                        if render_menu_item(ui, "▭", "New Rectangle", "R") {
                            state.active_tool = ActiveTool::Rectangle;
                            action_taken = true;
                        }
                        if render_menu_item(ui, "○", "New Ellipse", "O") {
                            state.active_tool = ActiveTool::Ellipse;
                            action_taken = true;
                        }
                        if render_menu_item(ui, "T", "New Text", "T") {
                            state.active_tool = ActiveTool::Text;
                            action_taken = true;
                        }

                        render_divider(ui);

                        if render_menu_item(ui, "⊡", "Fit to Screen", "⇧1") {
                            state.fit_to_screen(doc, screen_rect);
                            action_taken = true;
                        }

                        let grid_lbl = if state.show_grid { "Hide Dot Grid" } else { "Show Dot Grid" };
                        if render_menu_item(ui, "▦", grid_lbl, "") {
                            state.show_grid = !state.show_grid;
                            action_taken = true;
                        }

                        if render_menu_item(ui, "⌨", "Keyboard Shortcuts", "?") {
                            state.show_shortcuts_modal = true;
                            action_taken = true;
                        }

                        render_divider(ui);

                        if render_menu_item(ui, "🗑", "Clear Canvas", "") {
                            history.push_state(doc);
                            doc.nodes.clear();
                            doc.rebuild_spatial_index();
                            state.selected_ids.clear();
                            network.send_selection(&[]);
                            network.send_doc_update(doc);
                            action_taken = true;
                        }
                    }
                });
        });

    if close_menu || action_taken {
        state.context_menu_pos = None;
    }
}

fn render_menu_item(ui: &mut Ui, icon: &str, label: &str, shortcut: &str) -> bool {
    let mut clicked = false;
    let height = 28.0;
    let (rect, resp) = ui.allocate_exact_size(EguiVec2::new(ui.available_width(), height), egui::Sense::click());

    let is_hovered = resp.hovered();
    let bg_color = if is_hovered {
        Color32::from_rgb(10, 132, 255) // Active Blue highlight
    } else {
        Color32::TRANSPARENT
    };

    let painter = ui.painter_at(rect);
    if is_hovered {
        painter.rect_filled(rect, 6.0, bg_color);
    }

    // Left Icon & Label
    let text_color = if is_hovered { Color32::WHITE } else { Color32::from_rgb(225, 225, 230) };
    let icon_color = if is_hovered { Color32::WHITE } else { Color32::from_rgb(160, 160, 175) };

    painter.text(
        Pos2::new(rect.min.x + 8.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        icon,
        egui::FontId::proportional(12.0),
        icon_color,
    );

    painter.text(
        Pos2::new(rect.min.x + 28.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(12.0),
        text_color,
    );

    // Right shortcut tag
    if !shortcut.is_empty() {
        let sc_color = if is_hovered { Color32::from_rgba_premultiplied(255, 255, 255, 200) } else { Color32::from_rgb(120, 120, 135) };
        painter.text(
            Pos2::new(rect.max.x - 8.0, rect.center().y),
            egui::Align2::RIGHT_CENTER,
            shortcut,
            egui::FontId::proportional(11.0),
            sc_color,
        );
    }

    if resp.clicked() {
        clicked = true;
    }

    clicked
}

fn render_divider(ui: &mut Ui) {
    ui.add_space(3.0);
    let (rect, _) = ui.allocate_exact_size(EguiVec2::new(ui.available_width(), 1.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.line_segment(
        [Pos2::new(rect.min.x + 4.0, rect.center().y), Pos2::new(rect.max.x - 4.0, rect.center().y)],
        Stroke::new(1.0, Color32::from_rgb(38, 40, 48)),
    );
    ui.add_space(3.0);
}
