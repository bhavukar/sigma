use crate::canvas::{render_canvas, ActiveTool, CanvasState, ResizeHandle};
use crate::inspector::render_inspector;
use crate::network::NetworkManager;
use crate::sidebar::render_sidebar;
use crate::toolbar::render_toolbar;
use egui::{Key, Rect as EguiRect, Sense, Vec2 as EguiVec2};
use sigma_core::{Color, Document, History, Node, PeerState, Rect, SyncMessage, Vec2};

pub struct SigmaApp {
    pub doc: Document,
    pub canvas_state: CanvasState,
    pub history: History,
    pub network: NetworkManager,
    pub mouse_pos_world: Vec2,
    pub last_broadcast_pos: Vec2,
}

impl Default for SigmaApp {
    fn default() -> Self {
        let doc = Document::default();
        let room_id = get_initial_room();
        let (user_name, user_color) = get_user_profile();
        let network = NetworkManager::new(&room_id, &user_name, user_color);

        Self {
            doc,
            canvas_state: CanvasState::default(),
            history: History::default(),
            network,
            mouse_pos_world: Vec2::ZERO,
            last_broadcast_pos: Vec2::ZERO,
        }
    }
}

impl eframe::App for SigmaApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        ctx.set_visuals(egui::Visuals::dark());

        // 1. Process Incoming Real-time Network Messages from Cloudflare
        self.process_network_messages();

        // 2. Handle Global Keyboard Shortcuts
        self.handle_shortcuts(&ctx);

        // 3. Top Toolbar
        egui::Panel::top("top_toolbar")
            .exact_size(42.0)
            .show(ui, |ui| {
                render_toolbar(
                    ui,
                    &mut self.doc,
                    &mut self.canvas_state,
                    &mut self.history,
                    &self.network,
                );
            });

        // 4. Left Panel (Layers Tree)
        egui::Panel::left("left_sidebar")
            .exact_size(240.0)
            .resizable(false)
            .show(ui, |ui| {
                render_sidebar(ui, &mut self.doc, &mut self.canvas_state);
            });

        // 5. Right Panel (Design Inspector)
        egui::Panel::right("right_inspector")
            .exact_size(260.0)
            .resizable(false)
            .show(ui, |ui| {
                render_inspector(ui, &mut self.doc, &mut self.canvas_state);
            });

        // 6. Central Panel (The Infinite Design Canvas)
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ui, |ui| {
                let canvas_rect = ui.available_rect_before_wrap();
                let response = ui.allocate_rect(canvas_rect, Sense::click_and_drag());

                self.handle_canvas_input(ui, &response, canvas_rect);
                render_canvas(ui, &mut self.doc, &mut self.canvas_state, canvas_rect);
            });

        // Request regular repaints when peers are active
        if !self.canvas_state.remote_peers.is_empty() {
            ctx.request_repaint();
        }
    }
}

impl SigmaApp {
    fn process_network_messages(&mut self) {
        let messages = self.network.poll_incoming();
        for msg in messages {
            match msg {
                SyncMessage::RoomState { peers, doc } => {
                    for peer in peers {
                        if peer.id != self.network.user_id {
                            self.canvas_state.remote_peers.insert(peer.id.clone(), peer);
                        }
                    }
                    if let Some(remote_doc) = doc {
                        self.doc = remote_doc;
                    }
                }
                SyncMessage::Join {
                    user_id,
                    user_name,
                    color,
                } => {
                    if user_id != self.network.user_id {
                        let peer = PeerState::new(&user_id, &user_name, color);
                        self.canvas_state.remote_peers.insert(user_id, peer);
                    }
                }
                SyncMessage::Cursor { user_id, x, y } => {
                    if user_id != self.network.user_id {
                        if let Some(peer) = self.canvas_state.remote_peers.get_mut(&user_id) {
                            peer.cursor = Vec2::new(x, y);
                        } else {
                            let mut peer = PeerState::new(&user_id, "Designer", Color::FIGMA_BLUE);
                            peer.cursor = Vec2::new(x, y);
                            self.canvas_state.remote_peers.insert(user_id, peer);
                        }
                    }
                }
                SyncMessage::Selection {
                    user_id,
                    selected_ids,
                } => {
                    if user_id != self.network.user_id {
                        if let Some(peer) = self.canvas_state.remote_peers.get_mut(&user_id) {
                            peer.selected_ids = selected_ids;
                        }
                    }
                }
                SyncMessage::DocUpdate { user_id, doc } => {
                    if user_id != self.network.user_id {
                        self.doc = doc;
                    }
                }
                SyncMessage::Leave { user_id } => {
                    self.canvas_state.remote_peers.remove(&user_id);
                }
            }
        }
    }

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

            // Arrow keys Nudging (Figma 1px, Shift: 10px)
            let nudge_step = if i.modifiers.shift { 10.0 } else { 1.0 };
            let mut nudge_delta = Vec2::ZERO;
            if i.key_pressed(Key::ArrowLeft) {
                nudge_delta.x -= nudge_step;
            }
            if i.key_pressed(Key::ArrowRight) {
                nudge_delta.x += nudge_step;
            }
            if i.key_pressed(Key::ArrowUp) {
                nudge_delta.y -= nudge_step;
            }
            if i.key_pressed(Key::ArrowDown) {
                nudge_delta.y += nudge_step;
            }

            if nudge_delta != Vec2::ZERO && !self.canvas_state.selected_ids.is_empty() {
                self.history.push_state(&self.doc);
                for &id in &self.canvas_state.selected_ids {
                    if let Some(node) = self.doc.find_node_mut(id) {
                        node.rect.x += nudge_delta.x;
                        node.rect.y += nudge_delta.y;
                    }
                }
                self.network.send_doc_update(&self.doc);
            }

            // Select All (Cmd/Ctrl + A)
            if (i.modifiers.command || i.modifiers.ctrl) && i.key_pressed(Key::A) {
                self.canvas_state.selected_ids = self.doc.nodes.iter().map(|n| n.id).collect();
                self.network.send_selection(&self.canvas_state.selected_ids);
            }

            // Deselect All (Escape)
            if i.key_pressed(Key::Escape) {
                self.canvas_state.selected_ids.clear();
                self.network.send_selection(&[]);
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
                self.network.send_selection(&[]);
                self.network.send_doc_update(&self.doc);
            }

            // Duplicate (Cmd+D / Ctrl+D)
            if (i.modifiers.command || i.modifiers.ctrl) && i.key_pressed(Key::D) {
                if let Some(&first_id) = self.canvas_state.selected_ids.first() {
                    self.history.push_state(&self.doc);
                    if let Some(new_id) = self.doc.duplicate_node(first_id) {
                        self.canvas_state.selected_ids = vec![new_id];
                        self.network.send_selection(&self.canvas_state.selected_ids);
                        self.network.send_doc_update(&self.doc);
                    }
                }
            }

            // Undo / Redo
            if (i.modifiers.command || i.modifiers.ctrl) && i.key_pressed(Key::Z) {
                if i.modifiers.shift {
                    if let Some(next) = self.history.redo(&self.doc) {
                        self.doc = next;
                        self.network.send_doc_update(&self.doc);
                    }
                } else {
                    if let Some(prev) = self.history.undo(&self.doc) {
                        self.doc = prev;
                        self.network.send_doc_update(&self.doc);
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

        // Broadcast cursor position when mouse moves across canvas
        if (world_mouse - self.last_broadcast_pos).length() > 2.0 {
            self.network.send_cursor(world_mouse.x, world_mouse.y);
            self.last_broadcast_pos = world_mouse;
        }

        // 1. Mouse Wheel Zoom & Pan
        let scroll_delta = ui.input(|i| i.smooth_scroll_delta);
        let zoom_delta = ui.input(|i| i.zoom_delta());

        if zoom_delta != 1.0 {
            self.canvas_state.zoom_at_point(zoom_delta, mouse_pos, canvas_rect);
        } else if scroll_delta != EguiVec2::ZERO {
            if ui.input(|i| i.modifiers.ctrl || i.modifiers.command) {
                let factor = 1.0 + scroll_delta.y * 0.005;
                self.canvas_state.zoom_at_point(factor, mouse_pos, canvas_rect);
            } else {
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

        // 3. Selection Handle Hit-Testing & Cursor Updating
        let hovered_handle = if !self.canvas_state.selected_ids.is_empty() {
            if let Some(bbox) = self.doc.selection_bounding_box(&self.canvas_state.selected_ids) {
                self.canvas_state.hit_test_handle(bbox, mouse_pos, canvas_rect)
            } else {
                None
            }
        } else {
            None
        };

        if let Some(handle) = self.canvas_state.active_handle.or(hovered_handle) {
            let cursor_icon = match handle {
                ResizeHandle::TopLeft | ResizeHandle::BottomRight => egui::CursorIcon::ResizeNwSe,
                ResizeHandle::TopRight | ResizeHandle::BottomLeft => egui::CursorIcon::ResizeNeSw,
                ResizeHandle::TopCenter | ResizeHandle::BottomCenter => egui::CursorIcon::ResizeNorth,
                ResizeHandle::LeftCenter | ResizeHandle::RightCenter => egui::CursorIcon::ResizeEast,
            };
            ui.output_mut(|o| o.cursor_icon = cursor_icon);
        }

        // 4. Tool Click / Drag Operations
        match self.canvas_state.active_tool {
            ActiveTool::Select => {
                if response.drag_started() {
                    if let Some(handle) = hovered_handle {
                        // Start interactive 8-handle resizing!
                        self.canvas_state.active_handle = Some(handle);
                        if let Some(first_id) = self.canvas_state.selected_ids.first() {
                            if let Some(node) = self.doc.find_node(*first_id) {
                                self.canvas_state.initial_node_rect = Some(node.rect);
                            }
                        }
                        self.canvas_state.drag_start_pos = Some(world_mouse);
                        self.history.push_state(&self.doc);
                    } else if let Some(hit_id) = self.doc.hit_test(world_mouse) {
                        // Move element
                        if !self.canvas_state.selected_ids.contains(&hit_id) {
                            if ui.input(|i| i.modifiers.shift) {
                                self.canvas_state.selected_ids.push(hit_id);
                            } else {
                                self.canvas_state.selected_ids = vec![hit_id];
                            }
                            self.network.send_selection(&self.canvas_state.selected_ids);
                        }
                        self.history.push_state(&self.doc);
                        self.canvas_state.drag_start_pos = Some(world_mouse);
                    } else {
                        // Empty canvas -> start Marquee box selection
                        self.canvas_state.marquee_start = Some(world_mouse);
                        self.canvas_state.marquee_current = Some(world_mouse);
                        if !ui.input(|i| i.modifiers.shift) {
                            self.canvas_state.selected_ids.clear();
                            self.network.send_selection(&[]);
                        }
                    }
                } else if response.dragged() {
                    if let (Some(handle), Some(start), Some(initial_rect)) = (
                        self.canvas_state.active_handle,
                        self.canvas_state.drag_start_pos,
                        self.canvas_state.initial_node_rect,
                    ) {
                        // Interactive resizing calculations
                        let delta = world_mouse - start;
                        for &id in &self.canvas_state.selected_ids {
                            if let Some(node) = self.doc.find_node_mut(id) {
                                let mut new_rect = initial_rect;
                                match handle {
                                    ResizeHandle::BottomRight => {
                                        new_rect.width = (initial_rect.width + delta.x).max(10.0);
                                        new_rect.height = (initial_rect.height + delta.y).max(10.0);
                                    }
                                    ResizeHandle::BottomCenter => {
                                        new_rect.height = (initial_rect.height + delta.y).max(10.0);
                                    }
                                    ResizeHandle::RightCenter => {
                                        new_rect.width = (initial_rect.width + delta.x).max(10.0);
                                    }
                                    ResizeHandle::TopLeft => {
                                        let w = (initial_rect.width - delta.x).max(10.0);
                                        let h = (initial_rect.height - delta.y).max(10.0);
                                        new_rect.x = initial_rect.x + (initial_rect.width - w);
                                        new_rect.y = initial_rect.y + (initial_rect.height - h);
                                        new_rect.width = w;
                                        new_rect.height = h;
                                    }
                                    ResizeHandle::TopCenter => {
                                        let h = (initial_rect.height - delta.y).max(10.0);
                                        new_rect.y = initial_rect.y + (initial_rect.height - h);
                                        new_rect.height = h;
                                    }
                                    ResizeHandle::TopRight => {
                                        let h = (initial_rect.height - delta.y).max(10.0);
                                        new_rect.y = initial_rect.y + (initial_rect.height - h);
                                        new_rect.height = h;
                                        new_rect.width = (initial_rect.width + delta.x).max(10.0);
                                    }
                                    ResizeHandle::BottomLeft => {
                                        let w = (initial_rect.width - delta.x).max(10.0);
                                        new_rect.x = initial_rect.x + (initial_rect.width - w);
                                        new_rect.width = w;
                                        new_rect.height = (initial_rect.height + delta.y).max(10.0);
                                    }
                                    ResizeHandle::LeftCenter => {
                                        let w = (initial_rect.width - delta.x).max(10.0);
                                        new_rect.x = initial_rect.x + (initial_rect.width - w);
                                        new_rect.width = w;
                                    }
                                }
                                node.rect = new_rect;
                            }
                        }
                        self.network.send_doc_update(&self.doc);
                    } else if let Some(start) = self.canvas_state.drag_start_pos {
                        // Move elements
                        let delta = world_mouse - start;
                        for &id in &self.canvas_state.selected_ids {
                            if let Some(node) = self.doc.find_node_mut(id) {
                                node.rect.x += delta.x;
                                node.rect.y += delta.y;
                            }
                        }
                        self.canvas_state.drag_start_pos = Some(world_mouse);
                        self.network.send_doc_update(&self.doc);
                    } else if self.canvas_state.marquee_start.is_some() {
                        self.canvas_state.marquee_current = Some(world_mouse);
                    }
                } else if response.drag_stopped() {
                    if self.canvas_state.active_handle.is_some() {
                        self.canvas_state.active_handle = None;
                        self.canvas_state.initial_node_rect = None;
                        self.canvas_state.drag_start_pos = None;
                        self.network.send_doc_update(&self.doc);
                    } else if let (Some(m_start), Some(m_curr)) = (
                        self.canvas_state.marquee_start,
                        self.canvas_state.marquee_current,
                    ) {
                        let m_box = Rect::from_min_max(m_start, m_curr);
                        let mut newly_selected = Vec::new();
                        for node in &self.doc.nodes {
                            if node.visible && !node.locked && node.rect.intersects(&m_box) {
                                newly_selected.push(node.id);
                            }
                        }
                        if !newly_selected.is_empty() {
                            self.canvas_state.selected_ids = newly_selected;
                            self.network.send_selection(&self.canvas_state.selected_ids);
                        }
                        self.canvas_state.marquee_start = None;
                        self.canvas_state.marquee_current = None;
                    } else {
                        self.canvas_state.drag_start_pos = None;
                        self.network.send_doc_update(&self.doc);
                    }
                } else if response.clicked() {
                    if let Some(hit_id) = self.doc.hit_test(world_mouse) {
                        if ui.input(|i| i.modifiers.shift) {
                            if self.canvas_state.selected_ids.contains(&hit_id) {
                                self.canvas_state.selected_ids.retain(|&id| id != hit_id);
                            } else {
                                self.canvas_state.selected_ids.push(hit_id);
                            }
                        } else {
                            self.canvas_state.selected_ids = vec![hit_id];
                        }
                    } else {
                        self.canvas_state.selected_ids.clear();
                    }
                    self.network.send_selection(&self.canvas_state.selected_ids);
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
                            self.network.send_selection(&self.canvas_state.selected_ids);
                            self.network.send_doc_update(&self.doc);
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
                            self.network.send_selection(&self.canvas_state.selected_ids);
                            self.network.send_doc_update(&self.doc);
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
                            self.network.send_selection(&self.canvas_state.selected_ids);
                            self.network.send_doc_update(&self.doc);
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
                        "Type your text here...",
                        20.0,
                        Color::WHITE,
                    );
                    self.doc.nodes.push(new_text);
                    self.canvas_state.selected_ids = vec![id];
                    self.canvas_state.active_tool = ActiveTool::Select;
                    self.network.send_selection(&self.canvas_state.selected_ids);
                    self.network.send_doc_update(&self.doc);
                }
            }
            ActiveTool::Hand => {}
        }
    }
}

fn get_initial_room() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(win) = web_sys::window() {
            if let Ok(loc) = win.location().search() {
                if let Some(q) = loc.strip_prefix('?') {
                    for pair in q.split('&') {
                        if let Some((k, v)) = pair.split_once('=') {
                            if k == "room" && !v.is_empty() {
                                return v.to_string();
                            }
                        }
                    }
                }
            }
        }
    }
    "design-room".to_string()
}

fn get_user_profile() -> (String, Color) {
    let rand = js_sys_random_val();
    let idx = (rand * 6.0) as usize;
    let color = sigma_core::get_figma_palette(idx);
    let num = (rand * 900.0) as u32 + 100;
    (format!("Designer {}", num), color)
}

fn js_sys_random_val() -> f64 {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Math::random()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos();
        (nanos % 10000) as f64 / 10000.0
    }
}
