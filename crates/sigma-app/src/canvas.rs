use egui::{Color32, Pos2, Rect as EguiRect, Stroke as EguiStroke, Ui, Vec2 as EguiVec2};
use sigma_core::{Document, Node, NodeId, NodeKind, Rect, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTool {
    Select,
    Frame,
    Rectangle,
    Ellipse,
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

        let dot_color = Color32::from_rgba_premultiplied(60, 60, 60, 100);
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

    // 5. Render Selection Overlay & Handles
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
            let corner_radius = node.corner_radius.top_left * state.zoom;
            painter.rect(screen_rect, corner_radius, fill_color, stroke, egui::StrokeKind::Inside);

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
                painter.text(
                    screen_rect.min,
                    egui::Align2::LEFT_TOP,
                    &text_props.content,
                    egui::FontId::proportional(font_size),
                    fill_color,
                );
            }
        }
        NodeKind::Line => {
            painter.line_segment([screen_rect.min, screen_rect.max], stroke);
        }
    }
}
