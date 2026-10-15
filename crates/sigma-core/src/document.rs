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

    pub fn distribute_selection(&mut self, ids: &[NodeId], horizontal: bool) {
        if ids.len() < 3 {
            return;
        }
        let mut items: Vec<(usize, f32, f32)> = Vec::new();
        for (idx, node) in self.nodes.iter().enumerate() {
            if ids.contains(&node.id) {
                let (pos, size) = if horizontal {
                    (node.rect.x, node.rect.width)
                } else {
                    (node.rect.y, node.rect.height)
                };
                items.push((idx, pos, size));
            }
        }
        if items.len() < 3 {
            return;
        }
        items.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        let first_pos = items.first().unwrap().1;
        let last_item = items.last().unwrap();
        let total_span = (last_item.1 + last_item.2) - first_pos;
        let total_items_size: f32 = items.iter().map(|(_, _, s)| *s).sum();
        let remaining_space = (total_span - total_items_size).max(0.0);
        let gap = remaining_space / (items.len() - 1) as f32;

        let mut current_pos = first_pos;
        for (idx, _, size) in items {
            let node = &mut self.nodes[idx];
            if horizontal {
                node.rect.x = current_pos;
            } else {
                node.rect.y = current_pos;
            }
            current_pos += size + gap;
        }
    }

    pub fn bring_to_front(&mut self, ids: &[NodeId]) {
        let mut moved = Vec::new();
        self.nodes.retain(|n| {
            if ids.contains(&n.id) {
                moved.push(n.clone());
                false
            } else {
                true
            }
        });
        self.nodes.extend(moved);
    }

    pub fn send_to_back(&mut self, ids: &[NodeId]) {
        let mut moved = Vec::new();
        self.nodes.retain(|n| {
            if ids.contains(&n.id) {
                moved.push(n.clone());
                false
            } else {
                true
            }
        });
        moved.extend(self.nodes.drain(..));
        self.nodes = moved;
    }

    pub fn bring_forward(&mut self, ids: &[NodeId]) {
        if self.nodes.len() < 2 {
            return;
        }
        for i in (0..self.nodes.len() - 1).rev() {
            if ids.contains(&self.nodes[i].id) && !ids.contains(&self.nodes[i + 1].id) {
                self.nodes.swap(i, i + 1);
            }
        }
    }

    pub fn send_backward(&mut self, ids: &[NodeId]) {
        if self.nodes.len() < 2 {
            return;
        }
        for i in 1..self.nodes.len() {
            if ids.contains(&self.nodes[i].id) && !ids.contains(&self.nodes[i - 1].id) {
                self.nodes.swap(i, i - 1);
            }
        }
    }

    pub fn group_selection(&mut self, ids: &[NodeId]) -> Option<NodeId> {
        if ids.is_empty() {
            return None;
        }
        let bbox = self.selection_bounding_box(ids)?;
        let mut children = Vec::new();

        self.nodes.retain(|n| {
            if ids.contains(&n.id) {
                let mut child = n.clone();
                child.rect.x -= bbox.x;
                child.rect.y -= bbox.y;
                children.push(child);
                false
            } else {
                true
            }
        });

        if children.is_empty() {
            return None;
        }

        let group_id = self.alloc_id();
        let mut group = Node::new_frame(group_id, &format!("Group {}", group_id), bbox);
        group.fills.clear();
        group.strokes.clear();
        group.corner_radius = crate::style::CornerRadius::ZERO;
        group.children = children;

        self.nodes.push(group);
        Some(group_id)
    }

    pub fn ungroup_selection(&mut self, ids: &[NodeId]) -> Vec<NodeId> {
        let mut unpacked_ids = Vec::new();
        let mut new_root_nodes = Vec::new();

        self.nodes.retain(|n| {
            if ids.contains(&n.id) && !n.children.is_empty() {
                for mut child in n.children.clone() {
                    child.rect.x += n.rect.x;
                    child.rect.y += n.rect.y;
                    unpacked_ids.push(child.id);
                    new_root_nodes.push(child);
                }
                false
            } else {
                true
            }
        });

        self.nodes.extend(new_root_nodes);
        unpacked_ids
    }

    pub fn add_frame_preset(&mut self, name: &str, width: f32, height: f32, center: Vec2) -> NodeId {
        let id = self.alloc_id();
        let rect = Rect {
            x: center.x - width * 0.5,
            y: center.y - height * 0.5,
            width,
            height,
        };
        let mut frame = Node::new_frame(id, name, rect);
        frame.corner_radius = crate::style::CornerRadius::uniform(24.0);
        self.nodes.push(frame);
        id
    }

    pub fn insert_component(&mut self, component_type: &str, center: Vec2) -> NodeId {
        match component_type {
            "button" => {
                let id = self.alloc_id();
                let rect = Rect {
                    x: center.x - 75.0,
                    y: center.y - 22.0,
                    width: 150.0,
                    height: 44.0,
                };
                let mut btn = Node::new_rectangle(id, "Primary Button", rect, Color::rgb(10, 132, 255));
                btn.corner_radius = crate::style::CornerRadius::uniform(10.0);
                self.nodes.push(btn);

                let text_id = self.alloc_id();
                let text = Node::new_text(
                    text_id,
                    "Button Text",
                    Vec2::new(rect.x + 42.0, rect.y + 12.0),
                    "Button",
                    15.0,
                    Color::WHITE,
                );
                self.nodes.push(text);
                id
            }
            "card" => {
                let id = self.alloc_id();
                let rect = Rect {
                    x: center.x - 160.0,
                    y: center.y - 110.0,
                    width: 320.0,
                    height: 220.0,
                };
                let mut card = Node::new_frame(id, "Glass Card", rect);
                card.corner_radius = crate::style::CornerRadius::uniform(16.0);
                card.fills = vec![crate::style::Fill::Solid {
                    color: Color::rgb(30, 41, 59),
                    opacity: 0.95,
                }];
                card.strokes = vec![crate::style::Stroke {
                    color: Color::rgb(51, 65, 85),
                    width: 1.0,
                    align: crate::style::StrokeAlign::Inside,
                    opacity: 1.0,
                    style: crate::style::StrokeStyle::Solid,
                }];
                card.effects = vec![crate::style::Effect::DropShadow {
                    offset: Vec2::new(0.0, 10.0),
                    blur: 25.0,
                    spread: 0.0,
                    color: Color::rgba(0, 0, 0, 120),
                }];
                self.nodes.push(card);

                let title_id = self.alloc_id();
                let title = Node::new_text(
                    title_id,
                    "Card Header",
                    Vec2::new(rect.x + 24.0, rect.y + 24.0),
                    "Modern Card",
                    20.0,
                    Color::WHITE,
                );
                self.nodes.push(title);

                let desc_id = self.alloc_id();
                let desc = Node::new_text(
                    desc_id,
                    "Card Description",
                    Vec2::new(rect.x + 24.0, rect.y + 60.0),
                    "Ultra high-performance Rust UI.",
                    13.0,
                    Color::rgb(148, 163, 184),
                );
                self.nodes.push(desc);

                id
            }
            "input" => {
                let id = self.alloc_id();
                let rect = Rect {
                    x: center.x - 140.0,
                    y: center.y - 24.0,
                    width: 280.0,
                    height: 48.0,
                };
                let mut input = Node::new_rectangle(id, "Text Input", rect, Color::rgb(24, 24, 27));
                input.corner_radius = crate::style::CornerRadius::uniform(8.0);
                input.strokes = vec![crate::style::Stroke {
                    color: Color::rgb(63, 63, 70),
                    width: 1.0,
                    align: crate::style::StrokeAlign::Inside,
                    opacity: 1.0,
                    style: crate::style::StrokeStyle::Solid,
                }];
                self.nodes.push(input);

                let placeholder_id = self.alloc_id();
                let placeholder = Node::new_text(
                    placeholder_id,
                    "Placeholder",
                    Vec2::new(rect.x + 16.0, rect.y + 14.0),
                    "Enter your text...",
                    14.0,
                    Color::rgb(113, 113, 122),
                );
                self.nodes.push(placeholder);
                id
            }
            "avatar" => {
                let id = self.alloc_id();
                let rect = Rect {
                    x: center.x - 24.0,
                    y: center.y - 24.0,
                    width: 48.0,
                    height: 48.0,
                };
                let avatar = Node::new_ellipse(id, "Avatar Badge", rect, Color::rgb(139, 92, 246));
                self.nodes.push(avatar);

                let text_id = self.alloc_id();
                let text = Node::new_text(
                    text_id,
                    "Initials",
                    Vec2::new(rect.x + 14.0, rect.y + 14.0),
                    "SA",
                    15.0,
                    Color::WHITE,
                );
                self.nodes.push(text);
                id
            }
            _ => self.alloc_id(),
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
