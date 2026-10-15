use crate::canvas::{render_canvas, ActiveTool, CanvasState};
use crate::inspector::render_inspector;
use crate::sidebar::render_sidebar;
use crate::toolbar::render_toolbar;
use egui::{Key, Rect as EguiRect, Sense, Vec2 as EguiVec2};
use sigma_core::{Color, Document, History, Node, Rect, Vec2};

pub struct SigmaApp {
    pub doc: Document,
    pub canvas_state: CanvasState,
    pub history: History,
    pub mouse_pos_world: Vec2,
}

impl Default for SigmaApp {
    fn default() -> Self {
        let doc = Document::default();
        Self {
            doc,
            canvas_state: CanvasState::default(),
            history: History::default(),
            mouse_pos_world: Vec2::ZERO,
        }
    }
}

impl eframe::App for SigmaApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        ctx.set_visuals(egui::Visuals::dark());

        // Handle Global Keyboard Shortcuts
        self.handle_shortcuts(&ctx);

        // Top Toolbar
        egui::Panel::top("top_toolbar")
            .exact_size(42.0)
            .show(ui, |ui| {
                render_toolbar(ui, &mut self.doc, &mut self.canvas_state, &mut self.history);
            });

        // Left Panel (Layers Tree)
        egui::Panel::left("left_sidebar")
            .exact_size(240.0)
            .resizable(false)
            .show(ui, |ui| {
                render_sidebar(ui, &mut self.doc, &mut self.canvas_state);
            });

        // Right Panel (Design Inspector)
        egui::Panel::right("right_inspector")
            .exact_size(260.0)
            .resizable(false)
            .show(ui, |ui| {
                render_inspector(ui, &mut self.doc, &mut self.canvas_state);
            });

        // Central Panel (The Infinite Design Canvas)
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ui, |ui| {
                let canvas_rect = ui.available_rect_before_wrap();
                let response = ui.allocate_rect(canvas_rect, Sense::click_and_drag());

                self.handle_canvas_input(ui, &response, canvas_rect);
                render_canvas(ui, &mut self.doc, &mut self.canvas_state, canvas_rect);
            });
    }
}

impl SigmaApp {
    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            // Tool Shortcuts
            if i.key_pressed(Key::V) {
                self.canvas_state.active_tool = ActiveTool::Select;
            } else if i.key_pressed(Key::F) {
                self.canvas_state.active_tool = ActiveTool::Frame;
            } else if i.key_pressed(Key::R) {
                self.canvas_state.active_tool = ActiveTool::Rectangle;
            } else if i.key_pressed(Key::O) {
                self.canvas_state.active_tool = ActiveTool::Ellipse;
            } else if i.key_pressed(Key::T) {
                self.canvas_state.active_tool = ActiveTool::Text;
            } else if i.key_pressed(Key::H) {
                self.canvas_state.active_tool = ActiveTool::Hand;
            }

            // Delete selected
            if (i.key_pressed(Key::Delete) || i.key_pressed(Key::Backspace))
                && !self.canvas_state.selected_ids.is_empty()
            {
                self.history.push_state(&self.doc);
                for id in self.canvas_state.selected_ids.clone() {
                    self.doc.remove_node(id);
                }
                self.canvas_state.selected_ids.clear();
            }

            // Duplicate (Cmd+D / Ctrl+D)
            if (i.modifiers.command || i.modifiers.ctrl) && i.key_pressed(Key::D) {
                if let Some(&first_id) = self.canvas_state.selected_ids.first() {
                    self.history.push_state(&self.doc);
                    if let Some(new_id) = self.doc.duplicate_node(first_id) {
                        self.canvas_state.selected_ids = vec![new_id];
                    }
                }
            }

            // Undo / Redo
            if (i.modifiers.command || i.modifiers.ctrl) && i.key_pressed(Key::Z) {
                if i.modifiers.shift {
                    if let Some(next) = self.history.redo(&self.doc) {
                        self.doc = next;
                    }
                } else {
                    if let Some(prev) = self.history.undo(&self.doc) {
                        self.doc = prev;
                    }
                }
            }
        });
    }

    fn handle_canvas_input(
        &mut self,
        ui: &mut egui::Ui,
        response: &egui::Response,
        canvas_rect: EguiRect,
    ) {
        let is_space_down = ui.input(|i| i.key_down(Key::Space));
        let mouse_pos = ui.input(|i| i.pointer.hover_pos()).unwrap_or(canvas_rect.center());
        let world_mouse = self.canvas_state.screen_to_world(mouse_pos, canvas_rect);
        self.mouse_pos_world = world_mouse;

        // 1. Mouse Wheel Zoom & Pan
        let scroll_delta = ui.input(|i| i.smooth_scroll_delta);
        let zoom_delta = ui.input(|i| i.zoom_delta());

        if zoom_delta != 1.0 {
            self.canvas_state.zoom_at_point(zoom_delta, mouse_pos, canvas_rect);
        } else if scroll_delta != EguiVec2::ZERO {
            if ui.input(|i| i.modifiers.ctrl || i.modifiers.command) {
                // Ctrl + Scroll = Zoom
                let factor = 1.0 + scroll_delta.y * 0.005;
                self.canvas_state.zoom_at_point(factor, mouse_pos, canvas_rect);
            } else {
                // Two-finger trackpad scroll = Pan
                self.canvas_state.pan.x += scroll_delta.x / self.canvas_state.zoom;
                self.canvas_state.pan.y += scroll_delta.y / self.canvas_state.zoom;
            }
        }

        // 2. Pan Canvas with Hand tool, Spacebar, or Middle click
        if is_space_down
            || self.canvas_state.active_tool == ActiveTool::Hand
            || ui.input(|i| i.pointer.middle_down())
        {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grab);
            if response.dragged() {
                ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grabbing);
                let delta = response.drag_delta();
                self.canvas_state.pan.x += delta.x / self.canvas_state.zoom;
                self.canvas_state.pan.y += delta.y / self.canvas_state.zoom;
            }
            return;
        }

        // 3. Tool Click / Drag Operations
        match self.canvas_state.active_tool {
            ActiveTool::Select => {
                if response.drag_started() {
                    // Check if clicked an element
                    if let Some(hit_id) = self.doc.hit_test(world_mouse) {
                        if !self.canvas_state.selected_ids.contains(&hit_id) {
                            self.canvas_state.selected_ids = vec![hit_id];
                        }
                        self.history.push_state(&self.doc);
                        self.canvas_state.drag_start_pos = Some(world_mouse);
                    } else {
                        // Clicked empty canvas -> clear selection
                        self.canvas_state.selected_ids.clear();
                    }
                } else if response.dragged() {
                    if let Some(start) = self.canvas_state.drag_start_pos {
                        let delta = world_mouse - start;
                        for &id in &self.canvas_state.selected_ids {
                            if let Some(node) = self.doc.find_node_mut(id) {
                                node.rect.x += delta.x;
                                node.rect.y += delta.y;
                            }
                        }
                        self.canvas_state.drag_start_pos = Some(world_mouse);
                    }
                } else if response.drag_stopped() {
                    self.canvas_state.drag_start_pos = None;
                } else if response.clicked() {
                    if let Some(hit_id) = self.doc.hit_test(world_mouse) {
                        self.canvas_state.selected_ids = vec![hit_id];
                    } else {
                        self.canvas_state.selected_ids.clear();
                    }
                }
            }
            ActiveTool::Rectangle => {
                ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Crosshair);
                if response.drag_started() {
                    self.canvas_state.creation_start_pos = Some(world_mouse);
                    self.canvas_state.drag_start_pos = Some(world_mouse);
                } else if response.dragged() {
                    self.canvas_state.drag_start_pos = Some(world_mouse);
                } else if response.drag_stopped() {
                    if let Some(start) = self.canvas_state.creation_start_pos {
                        let rect = Rect::from_min_max(start, world_mouse);
                        if rect.width > 5.0 && rect.height > 5.0 {
                            self.history.push_state(&self.doc);
                            let id = self.doc.alloc_id();
                            let new_node = Node::new_rectangle(
                                id,
                                &format!("Rectangle {}", id),
                                rect,
                                Color::rgba(10, 132, 255, 255),
                            );
                            self.doc.nodes.push(new_node);
                            self.canvas_state.selected_ids = vec![id];
                        }
                    }
                    self.canvas_state.creation_start_pos = None;
                    self.canvas_state.drag_start_pos = None;
                    self.canvas_state.active_tool = ActiveTool::Select;
                }
            }
            ActiveTool::Frame => {
                ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Crosshair);
                if response.drag_started() {
                    self.canvas_state.creation_start_pos = Some(world_mouse);
                    self.canvas_state.drag_start_pos = Some(world_mouse);
                } else if response.dragged() {
                    self.canvas_state.drag_start_pos = Some(world_mouse);
                } else if response.drag_stopped() {
                    if let Some(start) = self.canvas_state.creation_start_pos {
                        let rect = Rect::from_min_max(start, world_mouse);
                        if rect.width > 10.0 && rect.height > 10.0 {
                            self.history.push_state(&self.doc);
                            let id = self.doc.alloc_id();
                            let new_frame = Node::new_frame(id, &format!("Frame {}", id), rect);
                            self.doc.nodes.push(new_frame);
                            self.canvas_state.selected_ids = vec![id];
                        }
                    }
                    self.canvas_state.creation_start_pos = None;
                    self.canvas_state.drag_start_pos = None;
                    self.canvas_state.active_tool = ActiveTool::Select;
                }
            }
            ActiveTool::Ellipse => {
                ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Crosshair);
                if response.drag_started() {
                    self.canvas_state.creation_start_pos = Some(world_mouse);
                    self.canvas_state.drag_start_pos = Some(world_mouse);
                } else if response.dragged() {
                    self.canvas_state.drag_start_pos = Some(world_mouse);
                } else if response.drag_stopped() {
                    if let Some(start) = self.canvas_state.creation_start_pos {
                        let rect = Rect::from_min_max(start, world_mouse);
                        if rect.width > 5.0 && rect.height > 5.0 {
                            self.history.push_state(&self.doc);
                            let id = self.doc.alloc_id();
                            let new_node = Node::new_ellipse(
                                id,
                                &format!("Ellipse {}", id),
                                rect,
                                Color::rgba(255, 69, 58, 255),
                            );
                            self.doc.nodes.push(new_node);
                            self.canvas_state.selected_ids = vec![id];
                        }
                    }
                    self.canvas_state.creation_start_pos = None;
                    self.canvas_state.drag_start_pos = None;
                    self.canvas_state.active_tool = ActiveTool::Select;
                }
            }
            ActiveTool::Text => {
                ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Text);
                if response.clicked() {
                    self.history.push_state(&self.doc);
                    let id = self.doc.alloc_id();
                    let new_text = Node::new_text(
                        id,
                        &format!("Text {}", id),
                        world_mouse,
                        "New Text",
                        16.0,
                        Color::WHITE,
                    );
                    self.doc.nodes.push(new_text);
                    self.canvas_state.selected_ids = vec![id];
                    self.canvas_state.active_tool = ActiveTool::Select;
                }
            }
            ActiveTool::Hand => {}
        }
    }
}
