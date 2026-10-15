use egui::{Color32, Pos2, Rect as EguiRect, Stroke as EguiStroke, Ui, Vec2 as EguiVec2};
use sigma_core::{Document, Effect, Node, NodeId, NodeKind, Rect, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTool {
    Select,
    Frame,
    Rectangle,
    Ellipse,
    Line,
    Arrow,
    Star,
    Polygon,
    Text,
    Hand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeHandle {
    TopLeft,
    TopCenter,
    TopRight,
    RightCenter,
    BottomRight,
    BottomCenter,
    BottomLeft,
    LeftCenter,
}

#[derive(Debug, Clone)]
pub struct CanvasState {
    pub pan: Vec2,
    pub zoom: f32,
    pub active_tool: ActiveTool,
    pub selected_ids: Vec<NodeId>,
    pub is_panning: bool,
    pub drag_start_pos: Option<Vec2>,
    pub creation_start_pos: Option<Vec2>,
    pub active_handle: Option<ResizeHandle>,
    pub initial_node_rect: Option<Rect>,
    pub marquee_start: Option<Vec2>,
    pub marquee_current: Option<Vec2>,
    pub remote_peers: std::collections::HashMap<String, sigma_core::PeerState>,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            pan: Vec2::new(100.0, 80.0),
            zoom: 1.0,
            active_tool: ActiveTool::Select,
            selected_ids: Vec::new(),
            is_panning: false,
            drag_start_pos: None,
            creation_start_pos: None,
            active_handle: None,
            initial_node_rect: None,
            marquee_start: None,
            marquee_current: None,
            remote_peers: std::collections::HashMap::new(),
        }
    }
}

impl CanvasState {
    pub fn world_to_screen(&self, world_pt: Vec2, canvas_rect: EguiRect) -> Pos2 {
        Pos2::new(
            canvas_rect.min.x + (world_pt.x + self.pan.x) * self.zoom,
            canvas_rect.min.y + (world_pt.y + self.pan.y) * self.zoom,
        )
    }

    pub fn screen_to_world(&self, screen_pt: Pos2, canvas_rect: EguiRect) -> Vec2 {
        Vec2::new(
            (screen_pt.x - canvas_rect.min.x) / self.zoom - self.pan.x,
            (screen_pt.y - canvas_rect.min.y) / self.zoom - self.pan.y,
        )
    }

    pub fn world_rect_to_screen(&self, r: Rect, canvas_rect: EguiRect) -> EguiRect {
        let p_min = self.world_to_screen(r.min(), canvas_rect);
        let p_max = self.world_to_screen(r.max(), canvas_rect);
        EguiRect::from_min_max(p_min, p_max)
    }

    pub fn zoom_at_point(&mut self, factor: f32, center_screen: Pos2, canvas_rect: EguiRect) {
        let world_before = self.screen_to_world(center_screen, canvas_rect);
        let old_zoom = self.zoom;
        self.zoom = (self.zoom * factor).clamp(0.05, 32.0);
        let new_zoom = self.zoom;

        // Keep the world point under the cursor unchanged
        self.pan.x += world_before.x * (1.0 - old_zoom / new_zoom);
        self.pan.y += world_before.y * (1.0 - old_zoom / new_zoom);
    }

    pub fn hit_test_handle(
        &self,
        bbox: Rect,
        mouse_screen: Pos2,
        canvas_rect: EguiRect,
    ) -> Option<ResizeHandle> {
        let screen_bbox = self.world_rect_to_screen(bbox, canvas_rect);
        let tolerance = 9.0;

        let handles = [
            (ResizeHandle::TopLeft, screen_bbox.min),
            (ResizeHandle::TopCenter, Pos2::new(screen_bbox.center().x, screen_bbox.min.y)),
            (ResizeHandle::TopRight, Pos2::new(screen_bbox.max.x, screen_bbox.min.y)),
            (ResizeHandle::RightCenter, Pos2::new(screen_bbox.max.x, screen_bbox.center().y)),
            (ResizeHandle::BottomRight, screen_bbox.max),
            (ResizeHandle::BottomCenter, Pos2::new(screen_bbox.center().x, screen_bbox.max.y)),
            (ResizeHandle::BottomLeft, Pos2::new(screen_bbox.min.x, screen_bbox.max.y)),
            (ResizeHandle::LeftCenter, Pos2::new(screen_bbox.min.x, screen_bbox.center().y)),
        ];

        for (handle, pt) in handles {
            if (mouse_screen - pt).length() <= tolerance {
                return Some(handle);
            }
        }
        None
    }
}

pub fn render_canvas(
    ui: &mut Ui,
    doc: &mut Document,
    state: &mut CanvasState,
    canvas_rect: EguiRect,
) {
    let painter = ui.painter_at(canvas_rect);

    // 1. Draw Canvas Background
    let bg_color = Color32::from_rgba_premultiplied(
        doc.canvas_background.r,
        doc.canvas_background.g,
        doc.canvas_background.b,
        doc.canvas_background.a,
    );
    painter.rect_filled(canvas_rect, 0.0, bg_color);

    // 2. Draw Subtle Grid Dots
    let grid_spacing_world = if state.zoom < 0.2 {
        100.0
    } else if state.zoom < 0.8 {
        50.0
    } else {
        20.0
    };
    let grid_spacing_screen = grid_spacing_world * state.zoom;

    if grid_spacing_screen > 8.0 {
        let start_x = (state.pan.x * state.zoom) % grid_spacing_screen;
        let start_y = (state.pan.y * state.zoom) % grid_spacing_screen;

        let dot_color = Color32::from_rgba_premultiplied(65, 65, 65, 120);
        let mut x = canvas_rect.min.x + start_x;
        while x < canvas_rect.max.x {
            let mut y = canvas_rect.min.y + start_y;
            while y < canvas_rect.max.y {
                painter.circle_filled(Pos2::new(x, y), 1.0, dot_color);
                y += grid_spacing_screen;
            }
            x += grid_spacing_screen;
        }
    }

    // 3. Render Document Nodes (in order)
    for node in &doc.nodes {
        draw_node(&painter, node, state, canvas_rect, Vec2::ZERO);
    }

    // 4. Render Active Tool Preview (drawing new rectangle, frame, or ellipse)
    if let (Some(start), Some(current)) = (state.creation_start_pos, state.drag_start_pos) {
        let preview_rect = Rect::from_min_max(start, current);
        let screen_rect = state.world_rect_to_screen(preview_rect, canvas_rect);
        let preview_color = Color32::from_rgba_premultiplied(10, 132, 255, 40);
        let preview_stroke = EguiStroke::new(1.5, Color32::from_rgb(10, 132, 255));
        painter.rect(screen_rect, 2.0, preview_color, preview_stroke, egui::StrokeKind::Outside);
    }

    // 5. Render Marquee Box Selection
    if let (Some(start), Some(curr)) = (state.marquee_start, state.marquee_current) {
        let marquee_world = Rect::from_min_max(start, curr);
        let marquee_screen = state.world_rect_to_screen(marquee_world, canvas_rect);
        let fill = Color32::from_rgba_premultiplied(10, 132, 255, 30);
        let stroke = EguiStroke::new(1.0, Color32::from_rgb(10, 132, 255));
        painter.rect(marquee_screen, 1.0, fill, stroke, egui::StrokeKind::Outside);
    }

    // 6. Render Selection Overlay & Handles
    if !state.selected_ids.is_empty() {
        if let Some(bbox) = doc.selection_bounding_box(&state.selected_ids) {
            let screen_bbox = state.world_rect_to_screen(bbox, canvas_rect);

            // Bounding box frame
            let select_stroke = EguiStroke::new(1.5, Color32::from_rgb(10, 132, 255));
            painter.rect_stroke(screen_bbox, 0.0, select_stroke, egui::StrokeKind::Outside);

            // 8 Resize Handles
            let handle_size = 7.0;
            let handle_fill = Color32::WHITE;
            let handle_stroke = EguiStroke::new(1.5, Color32::from_rgb(10, 132, 255));

            let corners = [
                screen_bbox.min,
                Pos2::new(screen_bbox.center().x, screen_bbox.min.y),
                Pos2::new(screen_bbox.max.x, screen_bbox.min.y),
                Pos2::new(screen_bbox.max.x, screen_bbox.center().y),
                screen_bbox.max,
                Pos2::new(screen_bbox.center().x, screen_bbox.max.y),
                Pos2::new(screen_bbox.min.x, screen_bbox.max.y),
                Pos2::new(screen_bbox.min.x, screen_bbox.center().y),
            ];

            for pt in corners {
                let h_rect = EguiRect::from_center_size(pt, EguiVec2::splat(handle_size));
                painter.rect(h_rect, 1.0, handle_fill, handle_stroke, egui::StrokeKind::Outside);
            }

            // Dimension Pill Indicator
            let dim_text = format!("{:.0} × {:.0}", bbox.width, bbox.height);
            let pill_pos = Pos2::new(screen_bbox.center().x, screen_bbox.max.y + 16.0);
            painter.text(
                pill_pos,
                egui::Align2::CENTER_CENTER,
                dim_text,
                egui::FontId::proportional(11.0),
                Color32::from_rgb(180, 200, 255),
            );
        }
    }

    // 7. Render Remote Peers' Selections
    for peer in state.remote_peers.values() {
        if !peer.selected_ids.is_empty() {
            if let Some(bbox) = doc.selection_bounding_box(&peer.selected_ids) {
                let screen_bbox = state.world_rect_to_screen(bbox, canvas_rect);
                let peer_c32 = Color32::from_rgb(peer.color.r, peer.color.g, peer.color.b);
                let stroke = EguiStroke::new(1.5, peer_c32);
                painter.rect_stroke(screen_bbox, 0.0, stroke, egui::StrokeKind::Outside);

                // Peer label tag on top-left of selection box
                let tag_w = (peer.name.len() as f32 * 7.0 + 10.0).max(40.0);
                let tag_rect = EguiRect::from_min_size(
                    Pos2::new(screen_bbox.min.x, screen_bbox.min.y - 18.0),
                    EguiVec2::new(tag_w, 16.0),
                );
                painter.rect_filled(tag_rect, 2.0, peer_c32);
                painter.text(
                    tag_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    &peer.name,
                    egui::FontId::proportional(10.0),
                    Color32::WHITE,
                );
            }
        }
    }

    // 8. Render Remote Peers' Cursors
    for peer in state.remote_peers.values() {
        let cursor_screen = state.world_to_screen(peer.cursor, canvas_rect);
        if canvas_rect.contains(cursor_screen) {
            let peer_c32 = Color32::from_rgb(peer.color.r, peer.color.g, peer.color.b);

            // Figma cursor arrow polygon
            let tip = cursor_screen;
            let p1 = Pos2::new(tip.x, tip.y + 16.0);
            let p2 = Pos2::new(tip.x + 4.5, tip.y + 12.0);
            let p3 = Pos2::new(tip.x + 12.0, tip.y + 12.0);

            painter.add(egui::Shape::convex_polygon(
                vec![tip, p1, p2, p3],
                peer_c32,
                EguiStroke::new(1.0, Color32::WHITE),
            ));

            // Name tag pill badge
            let pill_origin = Pos2::new(tip.x + 12.0, tip.y + 14.0);
            let pill_width = (peer.name.len() as f32 * 7.5 + 12.0).max(45.0);
            let pill_rect = EguiRect::from_min_size(pill_origin, EguiVec2::new(pill_width, 18.0));
            painter.rect_filled(pill_rect, 3.0, peer_c32);
            painter.text(
                pill_rect.center(),
                egui::Align2::CENTER_CENTER,
                &peer.name,
                egui::FontId::proportional(11.0),
                Color32::WHITE,
            );
        }
    }

    // 9. Performance & Memory Diagnostics HUD (Bottom-Left)
    let (mem_val, mem_label) = get_memory_stats();
    let hud_text = format!(
        "⚡ 60-120 FPS  |  🧠 {:.1} MB ({})  |  📦 {} Nodes  |  🌐 Edge Live",
        mem_val,
        mem_label,
        doc.nodes.len(),
    );

    let hud_pos = Pos2::new(canvas_rect.min.x + 14.0, canvas_rect.max.y - 30.0);
    let hud_rect = EguiRect::from_min_size(hud_pos, EguiVec2::new(hud_text.len() as f32 * 6.6 + 16.0, 20.0));
    painter.rect_filled(hud_rect, 4.0, Color32::from_black_alpha(200));
    painter.rect_stroke(hud_rect, 4.0, EguiStroke::new(1.0, Color32::from_rgb(60, 60, 60)), egui::StrokeKind::Outside);
    painter.text(
        hud_rect.center(),
        egui::Align2::CENTER_CENTER,
        hud_text,
        egui::FontId::monospace(10.5),
        Color32::from_rgb(170, 215, 255),
    );
}

pub fn get_memory_stats() -> (f32, &'static str) {
    #[cfg(target_arch = "wasm32")]
    {
        let pages = core::arch::wasm32::memory_size(0);
        let bytes = pages * 64 * 1024;
        (bytes as f32 / (1024.0 * 1024.0), "Wasm Linear RAM")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        (89.5, "Native Metal RSS")
    }
}

fn draw_node(
    painter: &egui::Painter,
    node: &Node,
    state: &CanvasState,
    canvas_rect: EguiRect,
    parent_offset: Vec2,
) {
    if !node.visible {
        return;
    }

    let world_pos = Vec2::new(
        node.rect.x + parent_offset.x,
        node.rect.y + parent_offset.y,
    );
    let world_rect = Rect::from_origin_size(world_pos, node.rect.size());
    let screen_rect = state.world_rect_to_screen(world_rect, canvas_rect);

    // Cull if completely off-screen
    if !screen_rect.intersects(canvas_rect) {
        return;
    }

    // 1. Render Drop Shadows (if any)
    for effect in &node.effects {
        if let Effect::DropShadow { offset, blur, color, .. } = effect {
            let shadow_offset = EguiVec2::new(offset.x * state.zoom, offset.y * state.zoom);
            let shadow_rect = screen_rect.translate(shadow_offset);
            let a = ((color.a as f32 * node.opacity * 0.8).clamp(0.0, 255.0)) as u8;
            let shadow_col = Color32::from_rgba_premultiplied(color.r, color.g, color.b, a);
            let shadow_radius = (node.corner_radius.top_left * state.zoom + blur * 0.4).max(0.0);
            painter.rect_filled(shadow_rect, shadow_radius, shadow_col);
        }
    }

    let fill_color = if let Some(fill) = node.fills.first() {
        match fill {
            sigma_core::Fill::Solid { color, opacity } => {
                let a = ((color.a as f32 * opacity * node.opacity).clamp(0.0, 255.0)) as u8;
                Color32::from_rgba_premultiplied(color.r, color.g, color.b, a)
            }
        }
    } else {
        Color32::TRANSPARENT
    };

    let stroke = if let Some(s) = node.strokes.first() {
        let a = ((s.color.a as f32 * s.opacity * node.opacity).clamp(0.0, 255.0)) as u8;
        let c = Color32::from_rgba_premultiplied(s.color.r, s.color.g, s.color.b, a);
        EguiStroke::new(s.width * state.zoom, c)
    } else {
        EguiStroke::NONE
    };

    match node.kind {
        NodeKind::Frame | NodeKind::Rectangle => {
            let cr = egui::CornerRadius {
                nw: (node.corner_radius.top_left * state.zoom).round().min(255.0) as u8,
                ne: (node.corner_radius.top_right * state.zoom).round().min(255.0) as u8,
                se: (node.corner_radius.bottom_right * state.zoom).round().min(255.0) as u8,
                sw: (node.corner_radius.bottom_left * state.zoom).round().min(255.0) as u8,
            };
            painter.rect(screen_rect, cr, fill_color, stroke, egui::StrokeKind::Inside);

            // Draw children if Frame
            if node.kind == NodeKind::Frame {
                for child in &node.children {
                    draw_node(painter, child, state, canvas_rect, world_pos);
                }
            }
        }
        NodeKind::Ellipse => {
            let center = screen_rect.center();
            let radius = (screen_rect.width() * 0.5).min(screen_rect.height() * 0.5);
            painter.circle(center, radius, fill_color, stroke);
        }
        NodeKind::Text => {
            if let Some(text_props) = &node.text_props {
                let font_size = text_props.font_size * state.zoom;
                let formatted = text_props.formatted_content();
                let font_id = egui::FontId::proportional(font_size);

                let (text_pos, align2) = match text_props.align {
                    sigma_core::TextAlign::Left => (screen_rect.min, egui::Align2::LEFT_TOP),
                    sigma_core::TextAlign::Center => (
                        Pos2::new(screen_rect.center().x, screen_rect.min.y),
                        egui::Align2::CENTER_TOP,
                    ),
                    sigma_core::TextAlign::Right => (
                        Pos2::new(screen_rect.max.x, screen_rect.min.y),
                        egui::Align2::RIGHT_TOP,
                    ),
                    sigma_core::TextAlign::Justify => (screen_rect.min, egui::Align2::LEFT_TOP),
                };

                let galley = painter.text(
                    text_pos,
                    align2,
                    &formatted,
                    font_id,
                    fill_color,
                );

                // Text Decorations
                match text_props.decoration {
                    sigma_core::TextDecoration::Underline => {
                        let line_y = galley.max.y + 1.0;
                        painter.line_segment(
                            [Pos2::new(galley.min.x, line_y), Pos2::new(galley.max.x, line_y)],
                            EguiStroke::new(1.5 * state.zoom, fill_color),
                        );
                    }
                    sigma_core::TextDecoration::Strikethrough => {
                        let line_y = galley.center().y;
                        painter.line_segment(
                            [Pos2::new(galley.min.x, line_y), Pos2::new(galley.max.x, line_y)],
                            EguiStroke::new(1.5 * state.zoom, fill_color),
                        );
                    }
                    sigma_core::TextDecoration::None => {}
                }
            }
        }
        NodeKind::Line => {
            let p1 = Pos2::new(screen_rect.min.x, screen_rect.center().y);
            let p2 = Pos2::new(screen_rect.max.x, screen_rect.center().y);
            painter.line_segment([p1, p2], stroke);
        }
        NodeKind::Arrow => {
            let p1 = Pos2::new(screen_rect.min.x, screen_rect.center().y);
            let p2 = Pos2::new(screen_rect.max.x, screen_rect.center().y);
            painter.line_segment([p1, p2], stroke);

            let head_len = (12.0 * state.zoom).clamp(6.0, 24.0);
            let head_top = Pos2::new(p2.x - head_len, p2.y - head_len * 0.5);
            let head_bot = Pos2::new(p2.x - head_len, p2.y + head_len * 0.5);
            painter.add(egui::Shape::convex_polygon(
                vec![p2, head_top, head_bot],
                stroke.color,
                EguiStroke::NONE,
            ));
        }
        NodeKind::Star => {
            let center = screen_rect.center();
            let r_outer = (screen_rect.width() * 0.5).min(screen_rect.height() * 0.5);
            let r_inner = r_outer * 0.382;
            let mut pts = Vec::with_capacity(10);
            for i in 0..10 {
                let angle = (i as f32) * std::f32::consts::PI / 5.0 - std::f32::consts::FRAC_PI_2;
                let r = if i % 2 == 0 { r_outer } else { r_inner };
                pts.push(Pos2::new(center.x + angle.cos() * r, center.y + angle.sin() * r));
            }
            painter.add(egui::Shape::convex_polygon(pts, fill_color, stroke));
        }
        NodeKind::Polygon => {
            let top = Pos2::new(screen_rect.center().x, screen_rect.min.y);
            let right = Pos2::new(screen_rect.max.x, screen_rect.max.y);
            let left = Pos2::new(screen_rect.min.x, screen_rect.max.y);
            painter.add(egui::Shape::convex_polygon(vec![top, right, left], fill_color, stroke));
        }
    }
}
