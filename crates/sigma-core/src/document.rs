use crate::color::Color;
use crate::math::{Rect, Vec2};
use crate::node::{Node, NodeId, NodeKind};
use crate::spatial::SpatialIndex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub name: String,
    pub nodes: Vec<Node>,
    pub next_id: NodeId,
    pub canvas_background: Color,
    #[serde(skip, default)]
    pub spatial_index: SpatialIndex,
}

impl Default for Document {
    fn default() -> Self {
        let mut doc = Self {
            name: "Untitled Sigma".to_string(),
            nodes: vec![],
            next_id: 1,
            canvas_background: Color::rgba(30, 30, 30, 255),
            spatial_index: SpatialIndex::default(),
        };
        doc.load_default_template();
        doc.rebuild_spatial_index();
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
            spatial_index: SpatialIndex::default(),
        }
    }

    pub fn rebuild_spatial_index(&mut self) {
        self.spatial_index.rebuild(&self.nodes);
        for node in &self.nodes {
            if node.kind == NodeKind::Frame {
                for child in &node.children {
                    if child.visible {
                        let mut world_rect = child.rect;
                        world_rect.x += node.rect.x;
                        world_rect.y += node.rect.y;
                        self.spatial_index.insert(child.id, world_rect);
                    }
                }
            }
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
        if node.visible {
            self.spatial_index.insert(id, node.rect);
        }
        self.nodes.push(node);
        id
    }

    pub fn remove_node(&mut self, id: NodeId) -> Option<Node> {
        self.spatial_index.remove(id);
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
        let candidates = self.spatial_index.query_point(point);
        if !candidates.is_empty() {
            let cand_set: std::collections::HashSet<NodeId> = candidates.into_iter().collect();
            // Test in reverse render order to pick the topmost node
            for node in self.nodes.iter().rev() {
                if !node.visible || node.locked {
                    continue;
                }
                if node.kind == NodeKind::Frame {
                    let local_point = Vec2::new(point.x - node.rect.x, point.y - node.rect.y);
                    for child in node.children.iter().rev() {
                        if child.visible && !child.locked && (cand_set.contains(&child.id) || child.rect.contains_point(local_point)) {
                            return Some(child.id);
                        }
                    }
                }
                if cand_set.contains(&node.id) {
                    return Some(node.id);
                }
            }
        }

        // Fallback: full reverse-order scan
        for node in self.nodes.iter().rev() {
            if !node.visible || node.locked {
                continue;
            }
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

    pub fn query_marquee_box(&self, bbox: Rect) -> Vec<NodeId> {
        let candidates = self.spatial_index.query_rect(bbox);
        if !candidates.is_empty() {
            let cand_set: std::collections::HashSet<NodeId> = candidates.into_iter().collect();
            let mut result = Vec::new();
            for node in &self.nodes {
                if node.visible && !node.locked && cand_set.contains(&node.id) {
                    result.push(node.id);
                }
            }
            return result;
        }

        let mut result = Vec::new();
        for node in &self.nodes {
            if node.visible && !node.locked && node.rect.intersects(&bbox) {
                result.push(node.id);
            }
        }
        result
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
        self.rebuild_spatial_index();
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
        self.rebuild_spatial_index();
    }

    pub fn bring_to_front(&mut self, ids: &[NodeId]) {
        // Reorder root nodes
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

        // Also reorder within each frame's children
        for parent in &mut self.nodes {
            if !parent.children.is_empty() {
                let mut moved_ch = Vec::new();
                parent.children.retain(|c| {
                    if ids.contains(&c.id) {
                        moved_ch.push(c.clone());
                        false
                    } else {
                        true
                    }
                });
                parent.children.extend(moved_ch);
            }
        }
    }

    pub fn send_to_back(&mut self, ids: &[NodeId]) {
        // Reorder root nodes
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

        // Also reorder within each frame's children
        for parent in &mut self.nodes {
            if !parent.children.is_empty() {
                let mut moved_ch = Vec::new();
                parent.children.retain(|c| {
                    if ids.contains(&c.id) {
                        moved_ch.push(c.clone());
                        false
                    } else {
                        true
                    }
                });
                moved_ch.extend(parent.children.drain(..));
                parent.children = moved_ch;
            }
        }
    }

    pub fn bring_forward(&mut self, ids: &[NodeId]) {
        if self.nodes.len() >= 2 {
            for i in (0..self.nodes.len() - 1).rev() {
                if ids.contains(&self.nodes[i].id) && !ids.contains(&self.nodes[i + 1].id) {
                    self.nodes.swap(i, i + 1);
                }
            }
        }
        for parent in &mut self.nodes {
            if parent.children.len() >= 2 {
                for i in (0..parent.children.len() - 1).rev() {
                    if ids.contains(&parent.children[i].id) && !ids.contains(&parent.children[i + 1].id) {
                        parent.children.swap(i, i + 1);
                    }
                }
            }
        }
    }

    pub fn send_backward(&mut self, ids: &[NodeId]) {
        if self.nodes.len() >= 2 {
            for i in 1..self.nodes.len() {
                if ids.contains(&self.nodes[i].id) && !ids.contains(&self.nodes[i - 1].id) {
                    self.nodes.swap(i, i - 1);
                }
            }
        }
        for parent in &mut self.nodes {
            if parent.children.len() >= 2 {
                for i in 1..parent.children.len() {
                    if ids.contains(&parent.children[i].id) && !ids.contains(&parent.children[i - 1].id) {
                        parent.children.swap(i, i - 1);
                    }
                }
            }
        }
    }

    pub fn move_node_up(&mut self, id: NodeId) -> bool {
        // In render order, moving "up" means towards index len-1 (closer to front/top)
        if let Some(pos) = self.nodes.iter().position(|n| n.id == id) {
            if pos + 1 < self.nodes.len() {
                self.nodes.swap(pos, pos + 1);
                return true;
            }
            return false;
        }
        for parent in &mut self.nodes {
            if let Some(pos) = parent.children.iter().position(|c| c.id == id) {
                if pos + 1 < parent.children.len() {
                    parent.children.swap(pos, pos + 1);
                    return true;
                }
                return false;
            }
        }
        false
    }

    pub fn move_node_down(&mut self, id: NodeId) -> bool {
        // In render order, moving "down" means towards index 0 (closer to back/bottom)
        if let Some(pos) = self.nodes.iter().position(|n| n.id == id) {
            if pos > 0 {
                self.nodes.swap(pos, pos - 1);
                return true;
            }
            return false;
        }
        for parent in &mut self.nodes {
            if let Some(pos) = parent.children.iter().position(|c| c.id == id) {
                if pos > 0 {
                    parent.children.swap(pos, pos - 1);
                    return true;
                }
                return false;
            }
        }
        false
    }

    pub fn find_parent_frame_id(&self, child_id: NodeId) -> Option<NodeId> {
        for node in &self.nodes {
            if node.children.iter().any(|c| c.id == child_id) {
                return Some(node.id);
            }
        }
        None
    }

    pub fn extract_from_parent(&mut self, child_id: NodeId) -> bool {
        for frame in &mut self.nodes {
            if let Some(pos) = frame.children.iter().position(|c| c.id == child_id) {
                let mut child = frame.children.remove(pos);
                child.rect.x += frame.rect.x;
                child.rect.y += frame.rect.y;
                self.nodes.push(child);
                self.rebuild_spatial_index();
                return true;
            }
        }
        false
    }

    pub fn move_node_into_frame(&mut self, node_id: NodeId, frame_id: NodeId) -> bool {
        if node_id == frame_id {
            return false;
        }
        let frame_rect = match self.nodes.iter().find(|n| n.id == frame_id && n.kind == NodeKind::Frame) {
            Some(f) => f.rect,
            None => return false,
        };

        if let Some(mut node) = self.remove_node(node_id) {
            node.rect.x -= frame_rect.x;
            node.rect.y -= frame_rect.y;
            if let Some(frame) = self.find_node_mut(frame_id) {
                frame.children.push(node);
                self.rebuild_spatial_index();
                return true;
            }
        }
        false
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
        self.rebuild_spatial_index();
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
        self.rebuild_spatial_index();
        unpacked_ids
    }

    pub fn frame_selection(&mut self, ids: &[NodeId]) -> Option<NodeId> {
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

        let frame_id = self.alloc_id();
        let mut frame = Node::new_frame(frame_id, &format!("Frame {}", frame_id), bbox);
        frame.fills = vec![crate::style::Fill::Solid {
            color: Color::WHITE,
            opacity: 1.0,
        }];
        frame.corner_radius = crate::style::CornerRadius::uniform(8.0);
        frame.children = children;

        self.nodes.push(frame);
        self.rebuild_spatial_index();
        Some(frame_id)
    }

    pub fn toggle_lock_selection(&mut self, ids: &[NodeId]) -> bool {
        let mut any_locked = false;
        for &id in ids {
            if let Some(node) = self.find_node_mut(id) {
                node.locked = !node.locked;
                any_locked |= node.locked;
            }
        }
        any_locked
    }

    pub fn toggle_vis_selection(&mut self, ids: &[NodeId]) -> bool {
        let mut any_visible = false;
        for &id in ids {
            if let Some(node) = self.find_node_mut(id) {
                node.visible = !node.visible;
                any_visible |= node.visible;
            }
        }
        self.rebuild_spatial_index();
        any_visible
    }

    pub fn clone_nodes_for_clipboard(&self, ids: &[NodeId]) -> Vec<Node> {
        ids.iter().filter_map(|&id| self.find_node(id).cloned()).collect()
    }

    pub fn paste_nodes(&mut self, nodes: &[Node], target_pos: Option<Vec2>) -> Vec<NodeId> {
        if nodes.is_empty() {
            return Vec::new();
        }

        let offset = if let Some(target) = target_pos {
            let min_x = nodes.iter().map(|n| n.rect.x).fold(f32::INFINITY, f32::min);
            let min_y = nodes.iter().map(|n| n.rect.y).fold(f32::INFINITY, f32::min);
            Vec2::new(target.x - min_x, target.y - min_y)
        } else {
            Vec2::new(20.0, 20.0)
        };

        let mut new_ids = Vec::new();
        for node in nodes {
            let mut cloned = node.clone();
            self.reassign_ids_recursively(&mut cloned);
            cloned.rect.x += offset.x;
            cloned.rect.y += offset.y;
            new_ids.push(cloned.id);
            if cloned.visible {
                self.spatial_index.insert(cloned.id, cloned.rect);
            }
            self.nodes.push(cloned);
        }
        self.rebuild_spatial_index();
        new_ids
    }

    fn reassign_ids_recursively(&mut self, node: &mut Node) {
        node.id = self.alloc_id();
        for child in &mut node.children {
            self.reassign_ids_recursively(child);
        }
    }

    pub fn get_next_frame_position(&self, default_center: Vec2, width: f32) -> Vec2 {
        let root_frames: Vec<&Node> = self.nodes.iter().filter(|n| n.kind == NodeKind::Frame).collect();
        if root_frames.is_empty() {
            default_center
        } else {
            let max_x = root_frames
                .iter()
                .map(|f| f.rect.x + f.rect.width)
                .fold(f32::NEG_INFINITY, f32::max);
            let first_y = root_frames
                .first()
                .map(|f| f.rect.y + f.rect.height * 0.5)
                .unwrap_or(default_center.y);
            Vec2::new(max_x + 80.0 + width * 0.5, first_y)
        }
    }

    pub fn add_frame_preset_styled(
        &mut self,
        name: &str,
        width: f32,
        height: f32,
        center: Vec2,
        corner_radius: f32,
    ) -> NodeId {
        let id = self.alloc_id();
        let rect = Rect {
            x: center.x - width * 0.5,
            y: center.y - height * 0.5,
            width,
            height,
        };
        let mut frame = Node::new_frame(id, name, rect);
        frame.corner_radius = crate::style::CornerRadius::uniform(corner_radius);
        frame.fills = vec![crate::style::Fill::Solid {
            color: Color::WHITE,
            opacity: 1.0,
        }];
        self.spatial_index.insert(id, rect);
        self.nodes.push(frame);
        id
    }

    pub fn add_frame_preset(&mut self, name: &str, width: f32, height: f32, center: Vec2) -> NodeId {
        self.add_frame_preset_styled(name, width, height, center, 24.0)
    }

    pub fn insert_component(&mut self, component_type: &str, center: Vec2) -> NodeId {
        let id = match component_type {
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
        };
        self.rebuild_spatial_index();
        id
    }

    pub fn duplicate_node(&mut self, id: NodeId) -> Option<NodeId> {
        let cloned = self.find_node(id)?.clone();
        let mut new_node = cloned;
        let new_id = self.alloc_id();
        new_node.id = new_id;
        new_node.name = format!("{} Copy", new_node.name);
        new_node.rect.x += 20.0;
        new_node.rect.y += 20.0;
        if new_node.visible {
            self.spatial_index.insert(new_id, new_node.rect);
        }
        self.nodes.push(new_node);
        Some(new_id)
    }

    pub fn load_default_template(&mut self) {
        // 1. Create a flagship iPhone 16 Pro artboard with fully nested children
        let frame_id = self.alloc_id();
        let mut frame = Node::new_frame(
            frame_id,
            "iPhone 16 Pro",
            Rect {
                x: 100.0,
                y: 80.0,
                width: 393.0,
                height: 852.0,
            },
        );
        frame.corner_radius = crate::style::CornerRadius::uniform(48.0);
        frame.fills = vec![crate::style::Fill::Solid {
            color: Color::WHITE,
            opacity: 1.0,
        }];
        frame.strokes = vec![crate::style::Stroke {
            color: Color::rgb(226, 232, 240),
            width: 1.5,
            align: crate::style::StrokeAlign::Inside,
            opacity: 1.0,
            style: crate::style::StrokeStyle::Solid,
        }];

        let mut children = Vec::new();

        // 1.1 Dynamic Island
        let island_id = self.alloc_id();
        let mut island = Node::new_rectangle(
            island_id,
            "Dynamic Island",
            Rect::new(146.0, 12.0, 100.0, 28.0),
            Color::BLACK,
        );
        island.corner_radius = crate::style::CornerRadius::uniform(14.0);
        children.push(island);

        // 1.2 Status Bar Time
        let time_id = self.alloc_id();
        let time = Node::new_text(
            time_id,
            "Status Time",
            Vec2::new(32.0, 16.0),
            "9:41",
            14.0,
            Color::BLACK,
        );
        children.push(time);

        // 1.3 User Header: Avatar + Greeting
        let avatar_id = self.alloc_id();
        let avatar = Node::new_ellipse(
            avatar_id,
            "User Avatar",
            Rect::new(24.0, 58.0, 44.0, 44.0),
            Color::rgb(10, 132, 255),
        );
        children.push(avatar);

        let greet_id = self.alloc_id();
        let greet = Node::new_text(
            greet_id,
            "Greeting",
            Vec2::new(76.0, 60.0),
            "Welcome back,",
            12.0,
            Color::rgb(100, 116, 139),
        );
        children.push(greet);

        let name_id = self.alloc_id();
        let user_name = Node::new_text(
            name_id,
            "User Name",
            Vec2::new(76.0, 78.0),
            "Alex Morgan",
            17.0,
            Color::rgb(15, 23, 42),
        );
        children.push(user_name);

        // 1.4 Hero Balance Card
        let card_id = self.alloc_id();
        let mut card = Node::new_rectangle(
            card_id,
            "Balance Card",
            Rect::new(24.0, 120.0, 345.0, 190.0),
            Color::rgb(15, 23, 42),
        );
        card.corner_radius = crate::style::CornerRadius::uniform(24.0);
        card.strokes = vec![crate::style::Stroke {
            color: Color::rgb(30, 41, 59),
            width: 1.0,
            align: crate::style::StrokeAlign::Inside,
            opacity: 1.0,
            style: crate::style::StrokeStyle::Solid,
        }];
        children.push(card);

        let bal_lbl_id = self.alloc_id();
        let bal_lbl = Node::new_text(
            bal_lbl_id,
            "Balance Label",
            Vec2::new(48.0, 142.0),
            "TOTAL PORTFOLIO",
            11.0,
            Color::rgb(148, 163, 184),
        );
        children.push(bal_lbl);

        let amount_id = self.alloc_id();
        let amount = Node::new_text(
            amount_id,
            "Balance Amount",
            Vec2::new(48.0, 164.0),
            "$128,450.80",
            28.0,
            Color::WHITE,
        );
        children.push(amount);

        // Growth Badge
        let badge_id = self.alloc_id();
        let mut badge = Node::new_rectangle(
            badge_id,
            "Growth Badge",
            Rect::new(48.0, 206.0, 105.0, 26.0),
            Color::rgb(16, 185, 129),
        );
        badge.corner_radius = crate::style::CornerRadius::uniform(13.0);
        children.push(badge);

        let badge_txt_id = self.alloc_id();
        let badge_txt = Node::new_text(
            badge_txt_id,
            "Badge Text",
            Vec2::new(58.0, 212.0),
            "+14.8% this month",
            10.5,
            Color::WHITE,
        );
        children.push(badge_txt);

        // Quick Send Button
        let send_btn_id = self.alloc_id();
        let mut send_btn = Node::new_rectangle(
            send_btn_id,
            "Send Button",
            Rect::new(48.0, 246.0, 100.0, 40.0),
            Color::rgb(10, 132, 255),
        );
        send_btn.corner_radius = crate::style::CornerRadius::uniform(20.0);
        children.push(send_btn);

        let send_txt_id = self.alloc_id();
        let send_txt = Node::new_text(
            send_txt_id,
            "Send Label",
            Vec2::new(76.0, 258.0),
            "Send ↗",
            13.0,
            Color::WHITE,
        );
        children.push(send_txt);

        // Quick Receive Button
        let recv_btn_id = self.alloc_id();
        let mut recv_btn = Node::new_rectangle(
            recv_btn_id,
            "Receive Button",
            Rect::new(160.0, 246.0, 100.0, 40.0),
            Color::rgb(30, 41, 59),
        );
        recv_btn.corner_radius = crate::style::CornerRadius::uniform(20.0);
        children.push(recv_btn);

        let recv_txt_id = self.alloc_id();
        let recv_txt = Node::new_text(
            recv_txt_id,
            "Receive Label",
            Vec2::new(182.0, 258.0),
            "Receive ↙",
            13.0,
            Color::WHITE,
        );
        children.push(recv_txt);

        // 1.5 Section Header: Recent Activity
        let sec_id = self.alloc_id();
        let sec = Node::new_text(
            sec_id,
            "Section Title",
            Vec2::new(24.0, 334.0),
            "Recent Activity",
            17.0,
            Color::rgb(15, 23, 42),
        );
        children.push(sec);

        // 1.6 Activity Item 1: Apple Store
        let item1_id = self.alloc_id();
        let mut item1 = Node::new_rectangle(
            item1_id,
            "Tx Apple Store",
            Rect::new(24.0, 364.0, 345.0, 68.0),
            Color::rgb(248, 250, 252),
        );
        item1.corner_radius = crate::style::CornerRadius::uniform(16.0);
        item1.strokes = vec![crate::style::Stroke {
            color: Color::rgb(226, 232, 240),
            width: 1.0,
            align: crate::style::StrokeAlign::Inside,
            opacity: 1.0,
            style: crate::style::StrokeStyle::Solid,
        }];
        children.push(item1);

        let icon1_id = self.alloc_id();
        let mut icon1 = Node::new_rectangle(
            icon1_id,
            "Icon 1",
            Rect::new(38.0, 378.0, 40.0, 40.0),
            Color::rgb(224, 231, 255),
        );
        icon1.corner_radius = crate::style::CornerRadius::uniform(12.0);
        children.push(icon1);

        let itm1_t_id = self.alloc_id();
        let itm1_t = Node::new_text(itm1_t_id, "Item 1 Title", Vec2::new(90.0, 380.0), "Apple Store", 15.0, Color::rgb(15, 23, 42));
        children.push(itm1_t);

        let itm1_s_id = self.alloc_id();
        let itm1_s = Node::new_text(itm1_s_id, "Item 1 Sub", Vec2::new(90.0, 400.0), "MacBook Pro M4", 12.0, Color::rgb(100, 116, 139));
        children.push(itm1_s);

        let itm1_p_id = self.alloc_id();
        let itm1_p = Node::new_text(itm1_p_id, "Item 1 Price", Vec2::new(270.0, 388.0), "-$2,499.00", 14.5, Color::rgb(239, 68, 68));
        children.push(itm1_p);

        // 1.7 Activity Item 2: Client Payout
        let item2_id = self.alloc_id();
        let mut item2 = Node::new_rectangle(
            item2_id,
            "Tx Client Payout",
            Rect::new(24.0, 442.0, 345.0, 68.0),
            Color::rgb(248, 250, 252),
        );
        item2.corner_radius = crate::style::CornerRadius::uniform(16.0);
        item2.strokes = vec![crate::style::Stroke {
            color: Color::rgb(226, 232, 240),
            width: 1.0,
            align: crate::style::StrokeAlign::Inside,
            opacity: 1.0,
            style: crate::style::StrokeStyle::Solid,
        }];
        children.push(item2);

        let icon2_id = self.alloc_id();
        let mut icon2 = Node::new_rectangle(
            icon2_id,
            "Icon 2",
            Rect::new(38.0, 456.0, 40.0, 40.0),
            Color::rgb(220, 252, 231),
        );
        icon2.corner_radius = crate::style::CornerRadius::uniform(12.0);
        children.push(icon2);

        let itm2_t_id = self.alloc_id();
        let itm2_t = Node::new_text(itm2_t_id, "Item 2 Title", Vec2::new(90.0, 458.0), "Client Payout", 15.0, Color::rgb(15, 23, 42));
        children.push(itm2_t);

        let itm2_s_id = self.alloc_id();
        let itm2_s = Node::new_text(itm2_s_id, "Item 2 Sub", Vec2::new(90.0, 478.0), "Sigma Design System", 12.0, Color::rgb(100, 116, 139));
        children.push(itm2_s);

        let itm2_p_id = self.alloc_id();
        let itm2_p = Node::new_text(itm2_p_id, "Item 2 Price", Vec2::new(270.0, 466.0), "+$8,500.00", 14.5, Color::rgb(16, 185, 129));
        children.push(itm2_p);

        // 1.8 Bottom Tab Bar & Home Indicator
        let tab_bar_id = self.alloc_id();
        let mut tab_bar = Node::new_rectangle(
            tab_bar_id,
            "Tab Bar",
            Rect::new(0.0, 772.0, 393.0, 80.0),
            Color::WHITE,
        );
        tab_bar.strokes = vec![crate::style::Stroke {
            color: Color::rgb(226, 232, 240),
            width: 1.0,
            align: crate::style::StrokeAlign::Inside,
            opacity: 1.0,
            style: crate::style::StrokeStyle::Solid,
        }];
        children.push(tab_bar);

        let ind_id = self.alloc_id();
        let mut ind = Node::new_rectangle(
            ind_id,
            "Home Indicator",
            Rect::new(126.0, 840.0, 140.0, 4.0),
            Color::BLACK,
        );
        ind.corner_radius = crate::style::CornerRadius::uniform(2.0);
        children.push(ind);

        frame.children = children;
        self.nodes.push(frame);

        // 2. Next to it on canvas: A Desktop Card / UI Kit Artboard
        let kit_id = self.alloc_id();
        let mut kit = Node::new_frame(
            kit_id,
            "Design System Components",
            Rect::new(550.0, 80.0, 440.0, 460.0),
        );
        kit.corner_radius = crate::style::CornerRadius::uniform(24.0);
        kit.fills = vec![crate::style::Fill::Solid {
            color: Color::rgb(24, 24, 27),
            opacity: 1.0,
        }];
        kit.strokes = vec![crate::style::Stroke {
            color: Color::rgb(39, 39, 42),
            width: 1.0,
            align: crate::style::StrokeAlign::Inside,
            opacity: 1.0,
            style: crate::style::StrokeStyle::Solid,
        }];

        let mut kit_children = Vec::new();

        let kit_title_id = self.alloc_id();
        let kit_title = Node::new_text(
            kit_title_id,
            "Kit Title",
            Vec2::new(32.0, 32.0),
            "UI Kit & Buttons",
            20.0,
            Color::WHITE,
        );
        kit_children.push(kit_title);

        let kit_sub_id = self.alloc_id();
        let kit_sub = Node::new_text(
            kit_sub_id,
            "Kit Subtitle",
            Vec2::new(32.0, 62.0),
            "Reusable vector components in Rust",
            13.0,
            Color::rgb(161, 161, 170),
        );
        kit_children.push(kit_sub);

        // Primary Button Component
        let pbtn_id = self.alloc_id();
        let mut pbtn = Node::new_rectangle(
            pbtn_id,
            "Button / Primary",
            Rect::new(32.0, 110.0, 160.0, 44.0),
            Color::rgb(10, 132, 255),
        );
        pbtn.corner_radius = crate::style::CornerRadius::uniform(10.0);
        kit_children.push(pbtn);

        let pbtn_t_id = self.alloc_id();
        let pbtn_t = Node::new_text(
            pbtn_t_id,
            "Button Text",
            Vec2::new(64.0, 122.0),
            "Primary Action",
            14.0,
            Color::WHITE,
        );
        kit_children.push(pbtn_t);

        // Secondary Button Component
        let sbtn_id = self.alloc_id();
        let mut sbtn = Node::new_rectangle(
            sbtn_id,
            "Button / Secondary",
            Rect::new(210.0, 110.0, 160.0, 44.0),
            Color::rgb(39, 39, 42),
        );
        sbtn.corner_radius = crate::style::CornerRadius::uniform(10.0);
        sbtn.strokes = vec![crate::style::Stroke {
            color: Color::rgb(63, 63, 70),
            width: 1.0,
            align: crate::style::StrokeAlign::Inside,
            opacity: 1.0,
            style: crate::style::StrokeStyle::Solid,
        }];
        kit_children.push(sbtn);

        let sbtn_t_id = self.alloc_id();
        let sbtn_t = Node::new_text(
            sbtn_t_id,
            "Secondary Text",
            Vec2::new(245.0, 122.0),
            "Secondary",
            14.0,
            Color::WHITE,
        );
        kit_children.push(sbtn_t);

        // Badges Showcase
        let bg1_id = self.alloc_id();
        let mut bg1 = Node::new_rectangle(bg1_id, "Badge Success", Rect::new(32.0, 180.0, 80.0, 28.0), Color::rgb(16, 185, 129));
        bg1.corner_radius = crate::style::CornerRadius::uniform(14.0);
        kit_children.push(bg1);

        let bg1_t_id = self.alloc_id();
        let bg1_t = Node::new_text(bg1_t_id, "Success", Vec2::new(48.0, 186.0), "Active", 12.0, Color::WHITE);
        kit_children.push(bg1_t);

        let bg2_id = self.alloc_id();
        let mut bg2 = Node::new_rectangle(bg2_id, "Badge Warning", Rect::new(125.0, 180.0, 80.0, 28.0), Color::rgb(245, 158, 11));
        bg2.corner_radius = crate::style::CornerRadius::uniform(14.0);
        kit_children.push(bg2);

        let bg2_t_id = self.alloc_id();
        let bg2_t = Node::new_text(bg2_t_id, "Pending", Vec2::new(140.0, 186.0), "Review", 12.0, Color::WHITE);
        kit_children.push(bg2_t);

        let bg3_id = self.alloc_id();
        let mut bg3 = Node::new_rectangle(bg3_id, "Badge Pro", Rect::new(218.0, 180.0, 80.0, 28.0), Color::rgb(168, 85, 247));
        bg3.corner_radius = crate::style::CornerRadius::uniform(14.0);
        kit_children.push(bg3);

        let bg3_t_id = self.alloc_id();
        let bg3_t = Node::new_text(bg3_t_id, "Pro", Vec2::new(246.0, 186.0), "PRO", 12.0, Color::WHITE);
        kit_children.push(bg3_t);

        // Modern Input Field
        let inp_id = self.alloc_id();
        let mut inp = Node::new_rectangle(
            inp_id,
            "Input Field",
            Rect::new(32.0, 235.0, 360.0, 48.0),
            Color::rgb(18, 18, 20),
        );
        inp.corner_radius = crate::style::CornerRadius::uniform(8.0);
        inp.strokes = vec![crate::style::Stroke {
            color: Color::rgb(63, 63, 70),
            width: 1.0,
            align: crate::style::StrokeAlign::Inside,
            opacity: 1.0,
            style: crate::style::StrokeStyle::Solid,
        }];
        kit_children.push(inp);

        let inpt_id = self.alloc_id();
        let inpt = Node::new_text(
            inpt_id,
            "Input Placeholder",
            Vec2::new(48.0, 250.0),
            "Search or jump to command...",
            13.5,
            Color::rgb(113, 113, 122),
        );
        kit_children.push(inpt);

        kit.children = kit_children;
        self.nodes.push(kit);

        self.rebuild_spatial_index();
    }

    pub fn insert_fintech_mobile(&mut self, center: Vec2) -> NodeId {
        let frame_id = self.alloc_id();
        let w = 393.0;
        let h = 852.0;
        let rect = Rect::new(center.x - w * 0.5, center.y - h * 0.5, w, h);
        let mut frame = Node::new_frame(frame_id, "iPhone 16 Pro - Fintech", rect);
        frame.corner_radius = crate::style::CornerRadius::uniform(48.0);
        frame.fills = vec![crate::style::Fill::Solid {
            color: Color::WHITE,
            opacity: 1.0,
        }];
        frame.strokes = vec![crate::style::Stroke {
            color: Color::rgb(226, 232, 240),
            width: 1.5,
            align: crate::style::StrokeAlign::Inside,
            opacity: 1.0,
            style: crate::style::StrokeStyle::Solid,
        }];

        let mut children = Vec::new();

        // Dynamic Island
        let island_id = self.alloc_id();
        let mut island = Node::new_rectangle(
            island_id,
            "Dynamic Island",
            Rect::new(146.0, 12.0, 100.0, 28.0),
            Color::BLACK,
        );
        island.corner_radius = crate::style::CornerRadius::uniform(14.0);
        children.push(island);

        // Status Bar Time
        let time_id = self.alloc_id();
        let time = Node::new_text(
            time_id,
            "Status Time",
            Vec2::new(32.0, 16.0),
            "9:41",
            14.0,
            Color::BLACK,
        );
        children.push(time);

        // User Avatar & Greeting
        let avatar_id = self.alloc_id();
        let avatar = Node::new_ellipse(
            avatar_id,
            "User Avatar",
            Rect::new(24.0, 58.0, 44.0, 44.0),
            Color::rgb(10, 132, 255),
        );
        children.push(avatar);

        let greet_id = self.alloc_id();
        let greet = Node::new_text(
            greet_id,
            "Greeting",
            Vec2::new(76.0, 60.0),
            "Welcome back,",
            12.0,
            Color::rgb(100, 116, 139),
        );
        children.push(greet);

        let name_id = self.alloc_id();
        let user_name = Node::new_text(
            name_id,
            "User Name",
            Vec2::new(76.0, 78.0),
            "Alex Morgan",
            17.0,
            Color::rgb(15, 23, 42),
        );
        children.push(user_name);

        // Balance Card
        let card_id = self.alloc_id();
        let mut card = Node::new_rectangle(
            card_id,
            "Balance Card",
            Rect::new(24.0, 120.0, 345.0, 190.0),
            Color::rgb(15, 23, 42),
        );
        card.corner_radius = crate::style::CornerRadius::uniform(24.0);
        children.push(card);

        let bal_lbl_id = self.alloc_id();
        let bal_lbl = Node::new_text(
            bal_lbl_id,
            "Balance Label",
            Vec2::new(48.0, 142.0),
            "TOTAL PORTFOLIO",
            11.0,
            Color::rgb(148, 163, 184),
        );
        children.push(bal_lbl);

        let amount_id = self.alloc_id();
        let amount = Node::new_text(
            amount_id,
            "Balance Amount",
            Vec2::new(48.0, 164.0),
            "$128,450.80",
            28.0,
            Color::WHITE,
        );
        children.push(amount);

        // Badge
        let badge_id = self.alloc_id();
        let mut badge = Node::new_rectangle(
            badge_id,
            "Growth Badge",
            Rect::new(48.0, 206.0, 105.0, 26.0),
            Color::rgb(16, 185, 129),
        );
        badge.corner_radius = crate::style::CornerRadius::uniform(13.0);
        children.push(badge);

        let badge_txt_id = self.alloc_id();
        let badge_txt = Node::new_text(
            badge_txt_id,
            "Badge Text",
            Vec2::new(58.0, 212.0),
            "+14.8% this month",
            10.5,
            Color::WHITE,
        );
        children.push(badge_txt);

        // Send & Receive Buttons
        let send_btn_id = self.alloc_id();
        let mut send_btn = Node::new_rectangle(
            send_btn_id,
            "Send Button",
            Rect::new(48.0, 246.0, 100.0, 40.0),
            Color::rgb(10, 132, 255),
        );
        send_btn.corner_radius = crate::style::CornerRadius::uniform(20.0);
        children.push(send_btn);

        let send_txt_id = self.alloc_id();
        let send_txt = Node::new_text(
            send_txt_id,
            "Send Label",
            Vec2::new(76.0, 258.0),
            "Send ↗",
            13.0,
            Color::WHITE,
        );
        children.push(send_txt);

        let recv_btn_id = self.alloc_id();
        let mut recv_btn = Node::new_rectangle(
            recv_btn_id,
            "Receive Button",
            Rect::new(160.0, 246.0, 100.0, 40.0),
            Color::rgb(30, 41, 59),
        );
        recv_btn.corner_radius = crate::style::CornerRadius::uniform(20.0);
        children.push(recv_btn);

        let recv_txt_id = self.alloc_id();
        let recv_txt = Node::new_text(
            recv_txt_id,
            "Receive Label",
            Vec2::new(182.0, 258.0),
            "Receive ↙",
            13.0,
            Color::WHITE,
        );
        children.push(recv_txt);

        // Recent Activity Section
        let sec_id = self.alloc_id();
        let sec = Node::new_text(
            sec_id,
            "Section Title",
            Vec2::new(24.0, 334.0),
            "Recent Activity",
            17.0,
            Color::rgb(15, 23, 42),
        );
        children.push(sec);

        // Activity Item 1
        let item1_id = self.alloc_id();
        let mut item1 = Node::new_rectangle(
            item1_id,
            "Tx Apple Store",
            Rect::new(24.0, 364.0, 345.0, 68.0),
            Color::rgb(248, 250, 252),
        );
        item1.corner_radius = crate::style::CornerRadius::uniform(16.0);
        children.push(item1);

        let itm1_t_id = self.alloc_id();
        let itm1_t = Node::new_text(itm1_t_id, "Item 1 Title", Vec2::new(88.0, 380.0), "Apple Store", 15.0, Color::rgb(15, 23, 42));
        children.push(itm1_t);

        let itm1_p_id = self.alloc_id();
        let itm1_p = Node::new_text(itm1_p_id, "Item 1 Price", Vec2::new(270.0, 388.0), "-$2,499.00", 14.5, Color::rgb(239, 68, 68));
        children.push(itm1_p);

        // Activity Item 2
        let item2_id = self.alloc_id();
        let mut item2 = Node::new_rectangle(
            item2_id,
            "Tx Client Payout",
            Rect::new(24.0, 442.0, 345.0, 68.0),
            Color::rgb(248, 250, 252),
        );
        item2.corner_radius = crate::style::CornerRadius::uniform(16.0);
        children.push(item2);

        let itm2_t_id = self.alloc_id();
        let itm2_t = Node::new_text(itm2_t_id, "Item 2 Title", Vec2::new(88.0, 458.0), "Client Payout", 15.0, Color::rgb(15, 23, 42));
        children.push(itm2_t);

        let itm2_p_id = self.alloc_id();
        let itm2_p = Node::new_text(itm2_p_id, "Item 2 Price", Vec2::new(270.0, 466.0), "+$8,500.00", 14.5, Color::rgb(16, 185, 129));
        children.push(itm2_p);

        // Bottom Tab Bar
        let tab_bar_id = self.alloc_id();
        let tab_bar = Node::new_rectangle(
            tab_bar_id,
            "Tab Bar",
            Rect::new(0.0, 772.0, 393.0, 80.0),
            Color::WHITE,
        );
        children.push(tab_bar);

        let ind_id = self.alloc_id();
        let mut ind = Node::new_rectangle(
            ind_id,
            "Home Indicator",
            Rect::new(126.0, 840.0, 140.0, 4.0),
            Color::BLACK,
        );
        ind.corner_radius = crate::style::CornerRadius::uniform(2.0);
        children.push(ind);

        frame.children = children;
        self.spatial_index.insert(frame_id, frame.rect);
        self.nodes.push(frame);
        frame_id
    }

    pub fn insert_saas_hero(&mut self, center: Vec2) -> NodeId {
        let frame_id = self.alloc_id();
        let w = 1200.0;
        let h = 720.0;
        let rect = Rect::new(center.x - w * 0.5, center.y - h * 0.5, w, h);
        let mut frame = Node::new_frame(frame_id, "Desktop - SaaS Landing Hero", rect);
        frame.corner_radius = crate::style::CornerRadius::uniform(24.0);
        frame.fills = vec![crate::style::Fill::Solid {
            color: Color::rgb(9, 9, 11),
            opacity: 1.0,
        }];
        frame.strokes = vec![crate::style::Stroke {
            color: Color::rgb(39, 39, 42),
            width: 1.5,
            align: crate::style::StrokeAlign::Inside,
            opacity: 1.0,
            style: crate::style::StrokeStyle::Solid,
        }];

        let mut children = Vec::new();

        // 1. Navigation Bar
        let nav_id = self.alloc_id();
        let mut nav = Node::new_rectangle(nav_id, "Navbar Bar", Rect::new(0.0, 0.0, 1200.0, 72.0), Color::rgb(12, 12, 15));
        nav.strokes = vec![crate::style::Stroke {
            color: Color::rgb(39, 39, 42),
            width: 1.0,
            align: crate::style::StrokeAlign::Inside,
            opacity: 1.0,
            style: crate::style::StrokeStyle::Solid,
        }];
        children.push(nav);

        let logo_id = self.alloc_id();
        let logo = Node::new_text(logo_id, "Logo", Vec2::new(48.0, 24.0), "✦ SIGMA PRO", 18.0, Color::WHITE);
        children.push(logo);

        let link1_id = self.alloc_id();
        let link1 = Node::new_text(link1_id, "Nav Link 1", Vec2::new(400.0, 26.0), "Features", 14.0, Color::rgb(161, 161, 170));
        children.push(link1);

        let link2_id = self.alloc_id();
        let link2 = Node::new_text(link2_id, "Nav Link 2", Vec2::new(490.0, 26.0), "Architecture", 14.0, Color::rgb(161, 161, 170));
        children.push(link2);

        let link3_id = self.alloc_id();
        let link3 = Node::new_text(link3_id, "Nav Link 3", Vec2::new(600.0, 26.0), "Pricing", 14.0, Color::rgb(161, 161, 170));
        children.push(link3);

        let nav_btn_id = self.alloc_id();
        let mut nav_btn = Node::new_rectangle(nav_btn_id, "Nav CTA", Rect::new(1030.0, 16.0, 120.0, 40.0), Color::rgb(10, 132, 255));
        nav_btn.corner_radius = crate::style::CornerRadius::uniform(8.0);
        children.push(nav_btn);

        let nav_btn_t_id = self.alloc_id();
        let nav_btn_t = Node::new_text(nav_btn_t_id, "Nav CTA Text", Vec2::new(1052.0, 26.0), "Get Started", 13.5, Color::WHITE);
        children.push(nav_btn_t);

        // 2. Announcement Pill
        let pill_id = self.alloc_id();
        let mut pill = Node::new_rectangle(pill_id, "Hero Pill", Rect::new(420.0, 120.0, 360.0, 32.0), Color::rgb(24, 24, 27));
        pill.corner_radius = crate::style::CornerRadius::uniform(16.0);
        pill.strokes = vec![crate::style::Stroke {
            color: Color::rgb(63, 63, 70),
            width: 1.0,
            align: crate::style::StrokeAlign::Inside,
            opacity: 1.0,
            style: crate::style::StrokeStyle::Solid,
        }];
        children.push(pill);

        let pill_t_id = self.alloc_id();
        let pill_t = Node::new_text(pill_t_id, "Hero Pill Text", Vec2::new(440.0, 128.0), "⚡ NEXT-GEN VECTOR GRAPHICS IN RUST & WASM", 11.0, Color::rgb(148, 163, 184));
        children.push(pill_t);

        // 3. Main Headline
        let head_id = self.alloc_id();
        let head = Node::new_text(head_id, "Hero Headline", Vec2::new(310.0, 175.0), "Design at the speed of thought.", 42.0, Color::WHITE);
        children.push(head);

        // 4. Subheadline
        let sub_id = self.alloc_id();
        let sub = Node::new_text(
            sub_id,
            "Hero Subtitle",
            Vec2::new(260.0, 240.0),
            "Zero garbage collection, 120 FPS rendering, and real-time edge sync with Cloudflare.",
            16.0,
            Color::rgb(161, 161, 170),
        );
        children.push(sub);

        // 5. Hero CTA Buttons
        let hcta1_id = self.alloc_id();
        let mut hcta1 = Node::new_rectangle(hcta1_id, "Primary Hero CTA", Rect::new(450.0, 290.0, 150.0, 48.0), Color::rgb(10, 132, 255));
        hcta1.corner_radius = crate::style::CornerRadius::uniform(10.0);
        children.push(hcta1);

        let hcta1_t_id = self.alloc_id();
        let hcta1_t = Node::new_text(hcta1_t_id, "Hero CTA 1 Text", Vec2::new(475.0, 304.0), "Start Free ➔", 14.5, Color::WHITE);
        children.push(hcta1_t);

        let hcta2_id = self.alloc_id();
        let mut hcta2 = Node::new_rectangle(hcta2_id, "Secondary Hero CTA", Rect::new(616.0, 290.0, 130.0, 48.0), Color::rgb(24, 24, 27));
        hcta2.corner_radius = crate::style::CornerRadius::uniform(10.0);
        hcta2.strokes = vec![crate::style::Stroke {
            color: Color::rgb(63, 63, 70),
            width: 1.0,
            align: crate::style::StrokeAlign::Inside,
            opacity: 1.0,
            style: crate::style::StrokeStyle::Solid,
        }];
        children.push(hcta2);

        let hcta2_t_id = self.alloc_id();
        let hcta2_t = Node::new_text(hcta2_t_id, "Hero CTA 2 Text", Vec2::new(638.0, 304.0), "GitHub ↗", 14.5, Color::WHITE);
        children.push(hcta2_t);

        // 6. Showcase Dashboard Preview Card
        let dash_id = self.alloc_id();
        let mut dash = Node::new_rectangle(dash_id, "Dashboard Canvas Preview", Rect::new(120.0, 370.0, 960.0, 300.0), Color::rgb(18, 18, 22));
        dash.corner_radius = crate::style::CornerRadius::uniform(20.0);
        dash.strokes = vec![crate::style::Stroke {
            color: Color::rgb(39, 39, 42),
            width: 1.5,
            align: crate::style::StrokeAlign::Inside,
            opacity: 1.0,
            style: crate::style::StrokeStyle::Solid,
        }];
        children.push(dash);

        // Preview inner stats chips
        let stat1_id = self.alloc_id();
        let mut stat1 = Node::new_rectangle(stat1_id, "Stat 1", Rect::new(160.0, 410.0, 240.0, 100.0), Color::rgb(24, 24, 30));
        stat1.corner_radius = crate::style::CornerRadius::uniform(12.0);
        children.push(stat1);

        let s1_val_id = self.alloc_id();
        let s1_val = Node::new_text(s1_val_id, "Stat 1 Val", Vec2::new(184.0, 430.0), "120 FPS", 24.0, Color::rgb(52, 199, 89));
        children.push(s1_val);

        let s1_lbl_id = self.alloc_id();
        let s1_lbl = Node::new_text(s1_lbl_id, "Stat 1 Lbl", Vec2::new(184.0, 470.0), "V-Sync Hardware Render", 12.0, Color::rgb(148, 163, 184));
        children.push(s1_lbl);

        let stat2_id = self.alloc_id();
        let mut stat2 = Node::new_rectangle(stat2_id, "Stat 2", Rect::new(430.0, 410.0, 240.0, 100.0), Color::rgb(24, 24, 30));
        stat2.corner_radius = crate::style::CornerRadius::uniform(12.0);
        children.push(stat2);

        let s2_val_id = self.alloc_id();
        let s2_val = Node::new_text(s2_val_id, "Stat 2 Val", Vec2::new(454.0, 430.0), "35 B/frame", 24.0, Color::rgb(10, 132, 255));
        children.push(s2_val);

        let s2_lbl_id = self.alloc_id();
        let s2_lbl = Node::new_text(s2_lbl_id, "Stat 2 Lbl", Vec2::new(454.0, 470.0), "Cloudflare WebSocket Delta", 12.0, Color::rgb(148, 163, 184));
        children.push(s2_lbl);

        let stat3_id = self.alloc_id();
        let mut stat3 = Node::new_rectangle(stat3_id, "Stat 3", Rect::new(700.0, 410.0, 240.0, 100.0), Color::rgb(24, 24, 30));
        stat3.corner_radius = crate::style::CornerRadius::uniform(12.0);
        children.push(stat3);

        let s3_val_id = self.alloc_id();
        let s3_val = Node::new_text(s3_val_id, "Stat 3 Val", Vec2::new(724.0, 430.0), "0.0% GC", 24.0, Color::rgb(168, 85, 247));
        children.push(s3_val);

        let s3_lbl_id = self.alloc_id();
        let s3_lbl = Node::new_text(s3_lbl_id, "Stat 3 Lbl", Vec2::new(724.0, 470.0), "Zero-overhead Rust memory", 12.0, Color::rgb(148, 163, 184));
        children.push(s3_lbl);

        frame.children = children;
        self.spatial_index.insert(frame_id, frame.rect);
        self.nodes.push(frame);
        frame_id
    }

    pub fn insert_pricing_card(&mut self, center: Vec2) -> NodeId {
        let frame_id = self.alloc_id();
        let w = 360.0;
        let h = 540.0;
        let rect = Rect::new(center.x - w * 0.5, center.y - h * 0.5, w, h);
        let mut frame = Node::new_frame(frame_id, "Card - Pro Pricing", rect);
        frame.corner_radius = crate::style::CornerRadius::uniform(24.0);
        frame.fills = vec![crate::style::Fill::Solid {
            color: Color::rgb(24, 24, 27),
            opacity: 1.0,
        }];
        frame.strokes = vec![crate::style::Stroke {
            color: Color::rgb(39, 39, 42),
            width: 1.5,
            align: crate::style::StrokeAlign::Inside,
            opacity: 1.0,
            style: crate::style::StrokeStyle::Solid,
        }];

        let mut children = Vec::new();

        // Badge
        let badge_id = self.alloc_id();
        let mut badge = Node::new_rectangle(badge_id, "Popular Pill", Rect::new(32.0, 32.0, 110.0, 26.0), Color::rgb(10, 132, 255));
        badge.corner_radius = crate::style::CornerRadius::uniform(13.0);
        children.push(badge);

        let badge_t_id = self.alloc_id();
        let badge_t = Node::new_text(badge_t_id, "Popular Text", Vec2::new(42.0, 38.0), "★ MOST POPULAR", 9.5, Color::WHITE);
        children.push(badge_t);

        // Title & Price
        let title_id = self.alloc_id();
        let title = Node::new_text(title_id, "Plan Title", Vec2::new(32.0, 76.0), "Sigma Pro", 24.0, Color::WHITE);
        children.push(title);

        let price_id = self.alloc_id();
        let price = Node::new_text(price_id, "Price", Vec2::new(32.0, 110.0), "$29", 36.0, Color::WHITE);
        children.push(price);

        let cycle_id = self.alloc_id();
        let cycle = Node::new_text(cycle_id, "Billing Cycle", Vec2::new(105.0, 126.0), "/ user / month", 13.0, Color::rgb(148, 163, 184));
        children.push(cycle);

        let desc_id = self.alloc_id();
        let desc = Node::new_text(desc_id, "Description", Vec2::new(32.0, 160.0), "For software teams requiring blazing fast design workflow.", 12.5, Color::rgb(161, 161, 170));
        children.push(desc);

        // Divider
        let div_id = self.alloc_id();
        let div = Node::new_rectangle(div_id, "Divider", Rect::new(32.0, 205.0, 296.0, 1.0), Color::rgb(39, 39, 42));
        children.push(div);

        // Feature list
        let feats = [
            "✓ Unlimited canvas artboards & frames",
            "✓ Real-time multi-user cursor sync",
            "✓ 120 FPS WebAssembly rendering engine",
            "✓ Pixel-perfect SVG & CSS code generation",
            "✓ Better Auth session & cloud security",
        ];
        for (i, feat) in feats.iter().enumerate() {
            let f_id = self.alloc_id();
            let f_text = Node::new_text(f_id, "Feature Item", Vec2::new(32.0, 225.0 + (i as f32) * 36.0), feat, 12.5, Color::rgb(228, 228, 231));
            children.push(f_text);
        }

        // CTA Button
        let cta_id = self.alloc_id();
        let mut cta = Node::new_rectangle(cta_id, "CTA Button", Rect::new(32.0, 450.0, 296.0, 50.0), Color::rgb(10, 132, 255));
        cta.corner_radius = crate::style::CornerRadius::uniform(12.0);
        children.push(cta);

        let cta_t_id = self.alloc_id();
        let cta_t = Node::new_text(cta_t_id, "CTA Text", Vec2::new(118.0, 466.0), "Upgrade to Pro ➔", 14.5, Color::WHITE);
        children.push(cta_t);

        frame.children = children;
        self.spatial_index.insert(frame_id, frame.rect);
        self.nodes.push(frame);
        frame_id
    }

    pub fn insert_auth_card(&mut self, center: Vec2) -> NodeId {
        let frame_id = self.alloc_id();
        let w = 400.0;
        let h = 480.0;
        let rect = Rect::new(center.x - w * 0.5, center.y - h * 0.5, w, h);
        let mut frame = Node::new_frame(frame_id, "Modal - Sign In", rect);
        frame.corner_radius = crate::style::CornerRadius::uniform(20.0);
        frame.fills = vec![crate::style::Fill::Solid {
            color: Color::rgb(24, 24, 27),
            opacity: 1.0,
        }];
        frame.strokes = vec![crate::style::Stroke {
            color: Color::rgb(39, 39, 42),
            width: 1.5,
            align: crate::style::StrokeAlign::Inside,
            opacity: 1.0,
            style: crate::style::StrokeStyle::Solid,
        }];

        let mut children = Vec::new();

        let title_id = self.alloc_id();
        let title = Node::new_text(title_id, "Modal Title", Vec2::new(32.0, 36.0), "Welcome to Sigma", 22.0, Color::WHITE);
        children.push(title);

        let sub_id = self.alloc_id();
        let sub = Node::new_text(sub_id, "Modal Sub", Vec2::new(32.0, 68.0), "Sign in to access your cloud workspace", 13.0, Color::rgb(161, 161, 170));
        children.push(sub);

        // Google button
        let gbtn_id = self.alloc_id();
        let mut gbtn = Node::new_rectangle(gbtn_id, "Google Button", Rect::new(32.0, 110.0, 336.0, 44.0), Color::rgb(30, 30, 34));
        gbtn.corner_radius = crate::style::CornerRadius::uniform(8.0);
        gbtn.strokes = vec![crate::style::Stroke {
            color: Color::rgb(63, 63, 70),
            width: 1.0,
            align: crate::style::StrokeAlign::Inside,
            opacity: 1.0,
            style: crate::style::StrokeStyle::Solid,
        }];
        children.push(gbtn);

        let gbtn_t_id = self.alloc_id();
        let gbtn_t = Node::new_text(gbtn_t_id, "Google Text", Vec2::new(115.0, 124.0), "G  Continue with Google", 13.5, Color::WHITE);
        children.push(gbtn_t);

        // Divider
        let div_id = self.alloc_id();
        let div = Node::new_text(div_id, "Divider Text", Vec2::new(110.0, 176.0), "— or continue with email —", 11.0, Color::rgb(113, 113, 122));
        children.push(div);

        // Email input
        let inp1_id = self.alloc_id();
        let mut inp1 = Node::new_rectangle(inp1_id, "Email Input", Rect::new(32.0, 210.0, 336.0, 44.0), Color::rgb(18, 18, 20));
        inp1.corner_radius = crate::style::CornerRadius::uniform(8.0);
        inp1.strokes = vec![crate::style::Stroke {
            color: Color::rgb(50, 50, 56),
            width: 1.0,
            align: crate::style::StrokeAlign::Inside,
            opacity: 1.0,
            style: crate::style::StrokeStyle::Solid,
        }];
        children.push(inp1);

        let inp1_t_id = self.alloc_id();
        let inp1_t = Node::new_text(inp1_t_id, "Email Placeholder", Vec2::new(48.0, 224.0), "alex@acme.studio", 13.0, Color::rgb(148, 163, 184));
        children.push(inp1_t);

        // Password input
        let inp2_id = self.alloc_id();
        let mut inp2 = Node::new_rectangle(inp2_id, "Password Input", Rect::new(32.0, 270.0, 336.0, 44.0), Color::rgb(18, 18, 20));
        inp2.corner_radius = crate::style::CornerRadius::uniform(8.0);
        inp2.strokes = vec![crate::style::Stroke {
            color: Color::rgb(50, 50, 56),
            width: 1.0,
            align: crate::style::StrokeAlign::Inside,
            opacity: 1.0,
            style: crate::style::StrokeStyle::Solid,
        }];
        children.push(inp2);

        let inp2_t_id = self.alloc_id();
        let inp2_t = Node::new_text(inp2_t_id, "Password Placeholder", Vec2::new(48.0, 284.0), "••••••••••••••••", 13.0, Color::rgb(113, 113, 122));
        children.push(inp2_t);

        // Submit Button
        let sbtn_id = self.alloc_id();
        let mut sbtn = Node::new_rectangle(sbtn_id, "Sign In Button", Rect::new(32.0, 340.0, 336.0, 46.0), Color::rgb(10, 132, 255));
        sbtn.corner_radius = crate::style::CornerRadius::uniform(10.0);
        children.push(sbtn);

        let sbtn_t_id = self.alloc_id();
        let sbtn_t = Node::new_text(sbtn_t_id, "Sign In Text", Vec2::new(125.0, 354.0), "Sign In with Better Auth", 13.5, Color::WHITE);
        children.push(sbtn_t);

        frame.children = children;
        self.spatial_index.insert(frame_id, frame.rect);
        self.nodes.push(frame);
        frame_id
    }

    pub fn insert_button_kit(&mut self, center: Vec2) -> NodeId {
        let frame_id = self.alloc_id();
        let w = 440.0;
        let h = 320.0;
        let rect = Rect::new(center.x - w * 0.5, center.y - h * 0.5, w, h);
        let mut frame = Node::new_frame(frame_id, "Component Kit - Buttons", rect);
        frame.corner_radius = crate::style::CornerRadius::uniform(20.0);
        frame.fills = vec![crate::style::Fill::Solid {
            color: Color::rgb(24, 24, 27),
            opacity: 1.0,
        }];
        frame.strokes = vec![crate::style::Stroke {
            color: Color::rgb(39, 39, 42),
            width: 1.5,
            align: crate::style::StrokeAlign::Inside,
            opacity: 1.0,
            style: crate::style::StrokeStyle::Solid,
        }];

        let mut children = Vec::new();

        let title_id = self.alloc_id();
        let title = Node::new_text(title_id, "Kit Title", Vec2::new(32.0, 28.0), "Button Design System", 18.0, Color::WHITE);
        children.push(title);

        // Row 1: Primary, Secondary, Outline
        let b1_id = self.alloc_id();
        let mut b1 = Node::new_rectangle(b1_id, "Primary Btn", Rect::new(32.0, 72.0, 115.0, 42.0), Color::rgb(10, 132, 255));
        b1.corner_radius = crate::style::CornerRadius::uniform(8.0);
        children.push(b1);
        let b1_t = Node::new_text(self.alloc_id(), "Primary", Vec2::new(62.0, 84.0), "Primary", 13.0, Color::WHITE);
        children.push(b1_t);

        let b2_id = self.alloc_id();
        let mut b2 = Node::new_rectangle(b2_id, "Secondary Btn", Rect::new(160.0, 72.0, 115.0, 42.0), Color::rgb(39, 39, 42));
        b2.corner_radius = crate::style::CornerRadius::uniform(8.0);
        children.push(b2);
        let b2_t = Node::new_text(self.alloc_id(), "Secondary", Vec2::new(180.0, 84.0), "Secondary", 13.0, Color::WHITE);
        children.push(b2_t);

        let b3_id = self.alloc_id();
        let mut b3 = Node::new_rectangle(b3_id, "Outline Btn", Rect::new(290.0, 72.0, 115.0, 42.0), Color::rgb(18, 18, 20));
        b3.corner_radius = crate::style::CornerRadius::uniform(8.0);
        b3.strokes = vec![crate::style::Stroke {
            color: Color::rgb(82, 82, 91),
            width: 1.0,
            align: crate::style::StrokeAlign::Inside,
            opacity: 1.0,
            style: crate::style::StrokeStyle::Solid,
        }];
        children.push(b3);
        let b3_t = Node::new_text(self.alloc_id(), "Outline", Vec2::new(324.0, 84.0), "Outline", 13.0, Color::rgb(228, 228, 231));
        children.push(b3_t);

        // Row 2: Success, Destructive
        let b4_id = self.alloc_id();
        let mut b4 = Node::new_rectangle(b4_id, "Success Btn", Rect::new(32.0, 135.0, 175.0, 42.0), Color::rgb(16, 185, 129));
        b4.corner_radius = crate::style::CornerRadius::uniform(8.0);
        children.push(b4);
        let b4_t = Node::new_text(self.alloc_id(), "Success", Vec2::new(76.0, 147.0), "Confirm & Save", 13.0, Color::WHITE);
        children.push(b4_t);

        let b5_id = self.alloc_id();
        let mut b5 = Node::new_rectangle(b5_id, "Destructive Btn", Rect::new(230.0, 135.0, 175.0, 42.0), Color::rgb(239, 68, 68));
        b5.corner_radius = crate::style::CornerRadius::uniform(8.0);
        children.push(b5);
        let b5_t = Node::new_text(self.alloc_id(), "Destructive", Vec2::new(278.0, 147.0), "Delete Item", 13.0, Color::WHITE);
        children.push(b5_t);

        // Row 3: Pill action button
        let b6_id = self.alloc_id();
        let mut b6 = Node::new_rectangle(b6_id, "Pill Action Btn", Rect::new(32.0, 205.0, 375.0, 48.0), Color::rgb(99, 102, 241));
        b6.corner_radius = crate::style::CornerRadius::uniform(24.0);
        children.push(b6);
        let b6_t = Node::new_text(self.alloc_id(), "Pill", Vec2::new(140.0, 219.0), "✦ Explore Component Library", 14.0, Color::WHITE);
        children.push(b6_t);

        frame.children = children;
        self.spatial_index.insert(frame_id, frame.rect);
        self.nodes.push(frame);
        frame_id
    }

    pub fn insert_badge_kit(&mut self, center: Vec2) -> NodeId {
        let frame_id = self.alloc_id();
        let w = 440.0;
        let h = 260.0;
        let rect = Rect::new(center.x - w * 0.5, center.y - h * 0.5, w, h);
        let mut frame = Node::new_frame(frame_id, "Component Kit - Badges", rect);
        frame.corner_radius = crate::style::CornerRadius::uniform(20.0);
        frame.fills = vec![crate::style::Fill::Solid {
            color: Color::rgb(24, 24, 27),
            opacity: 1.0,
        }];
        frame.strokes = vec![crate::style::Stroke {
            color: Color::rgb(39, 39, 42),
            width: 1.5,
            align: crate::style::StrokeAlign::Inside,
            opacity: 1.0,
            style: crate::style::StrokeStyle::Solid,
        }];

        let mut children = Vec::new();

        let title_id = self.alloc_id();
        let title = Node::new_text(title_id, "Kit Title", Vec2::new(32.0, 28.0), "Status Badges & Tags", 18.0, Color::WHITE);
        children.push(title);

        let badges = [
            ("Active", Color::rgb(16, 185, 129), 32.0, 75.0, 85.0),
            ("In Progress", Color::rgb(59, 130, 246), 130.0, 75.0, 110.0),
            ("Pending", Color::rgb(245, 158, 11), 255.0, 75.0, 95.0),
            ("Error / Failed", Color::rgb(239, 68, 68), 32.0, 125.0, 120.0),
            ("Enterprise", Color::rgb(168, 85, 247), 165.0, 125.0, 105.0),
            ("✦ PRO", Color::rgb(14, 165, 233), 285.0, 125.0, 85.0),
        ];

        for (label, color, bx, by, bw) in badges {
            let bg_id = self.alloc_id();
            let mut bg = Node::new_rectangle(bg_id, "Badge", Rect::new(bx, by, bw, 28.0), color);
            bg.corner_radius = crate::style::CornerRadius::uniform(14.0);
            children.push(bg);

            let txt_id = self.alloc_id();
            let txt = Node::new_text(txt_id, "Badge Label", Vec2::new(bx + 14.0, by + 6.5), label, 11.5, Color::WHITE);
            children.push(txt);
        }

        frame.children = children;
        self.spatial_index.insert(frame_id, frame.rect);
        self.nodes.push(frame);
        frame_id
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
