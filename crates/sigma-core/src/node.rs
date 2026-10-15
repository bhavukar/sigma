use crate::color::Color;
use crate::layout::AutoLayout;
use crate::math::{Rect, Vec2};
use crate::style::{CornerRadius, Effect, Fill, Stroke};
use serde::{Deserialize, Serialize};

pub type NodeId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeKind {
    Frame,
    Rectangle,
    Ellipse,
    Text,
    Line,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextProperties {
    pub content: String,
    pub font_size: f32,
    pub line_height: f32,
    pub letter_spacing: f32,
}

impl Default for TextProperties {
    fn default() -> Self {
        Self {
            content: "Text".to_string(),
            font_size: 16.0,
            line_height: 20.0,
            letter_spacing: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub name: String,
    pub kind: NodeKind,
    pub rect: Rect,
    pub rotation: f32,
    pub visible: bool,
    pub locked: bool,
    pub opacity: f32,
    pub fills: Vec<Fill>,
    pub strokes: Vec<Stroke>,
    pub effects: Vec<Effect>,
    pub corner_radius: CornerRadius,
    pub auto_layout: AutoLayout,
    pub children: Vec<Node>,
    pub text_props: Option<TextProperties>,
}

impl Node {
    pub fn new_rectangle(id: NodeId, name: &str, rect: Rect, fill_color: Color) -> Self {
        Self {
            id,
            name: name.to_string(),
            kind: NodeKind::Rectangle,
            rect,
            rotation: 0.0,
            visible: true,
            locked: false,
            opacity: 1.0,
            fills: vec![Fill::Solid {
                color: fill_color,
                opacity: 1.0,
            }],
            strokes: vec![],
            effects: vec![],
            corner_radius: CornerRadius::ZERO,
            auto_layout: AutoLayout::default(),
            children: vec![],
            text_props: None,
        }
    }

    pub fn new_frame(id: NodeId, name: &str, rect: Rect) -> Self {
        Self {
            id,
            name: name.to_string(),
            kind: NodeKind::Frame,
            rect,
            rotation: 0.0,
            visible: true,
            locked: false,
            opacity: 1.0,
            fills: vec![Fill::Solid {
                color: Color::WHITE,
                opacity: 1.0,
            }],
            strokes: vec![Stroke {
                color: Color::rgba(220, 220, 220, 255),
                width: 1.0,
                align: crate::style::StrokeAlign::Inside,
                opacity: 1.0,
            }],
            effects: vec![],
            corner_radius: CornerRadius::uniform(8.0),
            auto_layout: AutoLayout::default(),
            children: vec![],
            text_props: None,
        }
    }

    pub fn new_ellipse(id: NodeId, name: &str, rect: Rect, fill_color: Color) -> Self {
        Self {
            id,
            name: name.to_string(),
            kind: NodeKind::Ellipse,
            rect,
            rotation: 0.0,
            visible: true,
            locked: false,
            opacity: 1.0,
            fills: vec![Fill::Solid {
                color: fill_color,
                opacity: 1.0,
            }],
            strokes: vec![],
            effects: vec![],
            corner_radius: CornerRadius::ZERO,
            auto_layout: AutoLayout::default(),
            children: vec![],
            text_props: None,
        }
    }

    pub fn new_text(id: NodeId, name: &str, origin: Vec2, content: &str, font_size: f32, color: Color) -> Self {
        let est_width = content.len() as f32 * font_size * 0.6;
        let est_height = font_size * 1.3;
        Self {
            id,
            name: name.to_string(),
            kind: NodeKind::Text,
            rect: Rect {
                x: origin.x,
                y: origin.y,
                width: est_width.max(30.0),
                height: est_height.max(20.0),
            },
            rotation: 0.0,
            visible: true,
            locked: false,
            opacity: 1.0,
            fills: vec![Fill::Solid {
                color,
                opacity: 1.0,
            }],
            strokes: vec![],
            effects: vec![],
            corner_radius: CornerRadius::ZERO,
            auto_layout: AutoLayout::default(),
            children: vec![],
            text_props: Some(TextProperties {
                content: content.to_string(),
                font_size,
                line_height: font_size * 1.25,
                letter_spacing: 0.0,
            }),
        }
    }

    pub fn hit_test(&self, point: Vec2) -> bool {
        if !self.visible {
            return false;
        }
        self.rect.contains_point(point)
    }

    pub fn apply_auto_layout(&mut self) {
        if self.kind == NodeKind::Frame && self.auto_layout.enabled && !self.children.is_empty() {
            let child_sizes: Vec<Vec2> = self.children.iter().map(|c| c.rect.size()).collect();
            let (positions, _hug_size) = self
                .auto_layout
                .compute_children_positions(self.rect.size(), &child_sizes);

            for (child, pos) in self.children.iter_mut().zip(positions) {
                child.rect.x = pos.x;
                child.rect.y = pos.y;
            }
        }
    }
}
