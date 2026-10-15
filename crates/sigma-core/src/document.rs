use crate::color::Color;
use crate::math::{Rect, Vec2};
use crate::node::{Node, NodeId, NodeKind};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub name: String,
    pub nodes: Vec<Node>,
    pub next_id: NodeId,
    pub canvas_background: Color,
}

impl Default for Document {
    fn default() -> Self {
        let mut doc = Self {
            name: "Untitled Sigma".to_string(),
            nodes: vec![],
            next_id: 1,
            canvas_background: Color::rgba(30, 30, 30, 255),
        };
        doc.load_default_template();
        doc
    }
}

impl Document {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            nodes: vec![],
            next_id: 1,
            canvas_background: Color::rgba(30, 30, 30, 255),
        }
    }

    pub fn alloc_id(&mut self) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn add_node(&mut self, mut node: Node) -> NodeId {
        let id = self.alloc_id();
        node.id = id;
        self.nodes.push(node);
        id
    }

    pub fn remove_node(&mut self, id: NodeId) -> Option<Node> {
        if let Some(pos) = self.nodes.iter().position(|n| n.id == id) {
            return Some(self.nodes.remove(pos));
        }
        for node in &mut self.nodes {
            if let Some(pos) = node.children.iter().position(|c| c.id == id) {
                return Some(node.children.remove(pos));
            }
        }
        None
    }

    pub fn find_node(&self, id: NodeId) -> Option<&Node> {
        for node in &self.nodes {
            if node.id == id {
                return Some(node);
            }
            for child in &node.children {
                if child.id == id {
                    return Some(child);
                }
            }
        }
        None
    }

    pub fn find_node_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        for node in &mut self.nodes {
            if node.id == id {
                return Some(node);
            }
            for child in &mut node.children {
                if child.id == id {
                    return Some(child);
                }
            }
        }
        None
    }

    pub fn hit_test(&self, point: Vec2) -> Option<NodeId> {
        // Top-most rendered element is last in the list, so test in reverse order
        for node in self.nodes.iter().rev() {
            if !node.visible || node.locked {
                continue;
            }
            // If frame, check its children first
            if node.kind == NodeKind::Frame {
                let local_point = Vec2::new(point.x - node.rect.x, point.y - node.rect.y);
                for child in node.children.iter().rev() {
                    if child.visible && !child.locked && child.rect.contains_point(local_point) {
                        return Some(child.id);
                    }
                }
            }
            if node.rect.contains_point(point) {
                return Some(node.id);
            }
        }
        None
    }

    pub fn selection_bounding_box(&self, ids: &[NodeId]) -> Option<Rect> {
        let mut rect: Option<Rect> = None;
        for &id in ids {
            if let Some(node) = self.find_node(id) {
                // If it's a child of a frame, find parent frame offset
                let mut world_rect = node.rect;
                for parent in &self.nodes {
                    if parent.children.iter().any(|c| c.id == id) {
                        world_rect.x += parent.rect.x;
                        world_rect.y += parent.rect.y;
                        break;
                    }
                }

                rect = match rect {
                    None => Some(world_rect),
                    Some(r) => Some(r.union(&world_rect)),
                };
            }
        }
        rect
    }

    pub fn align_selection(&mut self, ids: &[NodeId], mode: AlignmentMode) {
        let bbox = match self.selection_bounding_box(ids) {
            Some(b) => b,
            None => return,
        };

        for &id in ids {
            if let Some(node) = self.find_node_mut(id) {
                match mode {
                    AlignmentMode::Left => node.rect.x = bbox.x,
                    AlignmentMode::CenterHorizontal => {
                        node.rect.x = bbox.x + (bbox.width - node.rect.width) * 0.5;
                    }
                    AlignmentMode::Right => {
                        node.rect.x = bbox.x + bbox.width - node.rect.width;
                    }
                    AlignmentMode::Top => node.rect.y = bbox.y,
                    AlignmentMode::CenterVertical => {
                        node.rect.y = bbox.y + (bbox.height - node.rect.height) * 0.5;
                    }
                    AlignmentMode::Bottom => {
                        node.rect.y = bbox.y + bbox.height - node.rect.height;
                    }
                }
            }
        }
    }

    pub fn duplicate_node(&mut self, id: NodeId) -> Option<NodeId> {
        let cloned = self.find_node(id)?.clone();
        let mut new_node = cloned;
        let new_id = self.alloc_id();
        new_node.id = new_id;
        new_node.name = format!("{} Copy", new_node.name);
        new_node.rect.x += 20.0;
        new_node.rect.y += 20.0;
        self.nodes.push(new_node);
        Some(new_id)
    }

    pub fn load_default_template(&mut self) {
        // Create an iPhone Artboard Frame
        let frame_id = self.alloc_id();
        let mut frame = Node::new_frame(
            frame_id,
            "iPhone 16 Pro",
            Rect {
                x: 100.0,
                y: 100.0,
                width: 393.0,
                height: 852.0,
            },
        );
        frame.corner_radius = crate::style::CornerRadius::uniform(44.0);
        frame.fills = vec![crate::style::Fill::Solid {
            color: Color::rgba(248, 249, 250, 255),
            opacity: 1.0,
        }];

        // Header Card inside Frame
        let card_id = self.alloc_id();
        let mut card = Node::new_rectangle(
            card_id,
            "Hero Card",
            Rect {
                x: 24.0,
                y: 60.0,
                width: 345.0,
                height: 220.0,
            },
            Color::rgba(20, 20, 25, 255),
        );
        card.corner_radius = crate::style::CornerRadius::uniform(24.0);

        // Card Title Text
        let title_id = self.alloc_id();
        let title = Node::new_text(
            title_id,
            "Card Title",
            Vec2::new(48.0, 100.0),
            "Welcome to Sigma",
            24.0,
            Color::WHITE,
        );

        // Subtitle Text
        let sub_id = self.alloc_id();
        let subtitle = Node::new_text(
            sub_id,
            "Subtitle",
            Vec2::new(48.0, 140.0),
            "Blazing fast Rust vector editor",
            14.0,
            Color::rgba(180, 185, 195, 255),
        );

        // Call-to-action button
        let btn_id = self.alloc_id();
        let mut btn = Node::new_rectangle(
            btn_id,
            "Primary Button",
            Rect {
                x: 48.0,
                y: 190.0,
                width: 150.0,
                height: 44.0,
            },
            Color::rgba(10, 132, 255, 255),
        );
        btn.corner_radius = crate::style::CornerRadius::uniform(22.0);

        let btn_text_id = self.alloc_id();
        let btn_text = Node::new_text(
            btn_text_id,
            "Button Label",
            Vec2::new(82.0, 202.0),
            "Get Started",
            14.0,
            Color::WHITE,
        );

        // Add an illustrative circle outside
        let circle_id = self.alloc_id();
        let circle = Node::new_ellipse(
            circle_id,
            "Accent Sphere",
            Rect {
                x: 560.0,
                y: 180.0,
                width: 160.0,
                height: 160.0,
            },
            Color::rgba(255, 107, 107, 255),
        );

        self.nodes.push(frame);
        self.nodes.push(card);
        self.nodes.push(title);
        self.nodes.push(subtitle);
        self.nodes.push(btn);
        self.nodes.push(btn_text);
        self.nodes.push(circle);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignmentMode {
    Left,
    CenterHorizontal,
    Right,
    Top,
    CenterVertical,
    Bottom,
}
