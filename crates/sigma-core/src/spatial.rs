use crate::math::{Rect, Vec2};
use crate::node::{Node, NodeId};
use std::collections::{HashMap, HashSet};

/// High-performance 2D spatial hash grid for O(1) hit testing and O(K) viewport frustum culling.
#[derive(Debug, Clone)]
pub struct SpatialIndex {
    cell_size: f32,
    cells: HashMap<(i32, i32), Vec<NodeId>>,
    node_bounds: HashMap<NodeId, Rect>,
}

impl Default for SpatialIndex {
    fn default() -> Self {
        Self::new(256.0)
    }
}

impl SpatialIndex {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size: cell_size.max(32.0),
            cells: HashMap::new(),
            node_bounds: HashMap::new(),
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.node_bounds.is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.node_bounds.len()
    }

    #[inline]
    fn to_cell_coord(&self, pos: f32) -> i32 {
        (pos / self.cell_size).floor() as i32
    }

    #[inline]
    fn get_cell_range(&self, rect: Rect) -> (i32, i32, i32, i32) {
        let min_x = self.to_cell_coord(rect.x);
        let max_x = self.to_cell_coord(rect.x + rect.width);
        let min_y = self.to_cell_coord(rect.y);
        let max_y = self.to_cell_coord(rect.y + rect.height);
        (min_x, max_x, min_y, max_y)
    }

    pub fn insert(&mut self, id: NodeId, rect: Rect) {
        let norm = rect.normalized();
        self.node_bounds.insert(id, norm);
        let (min_x, max_x, min_y, max_y) = self.get_cell_range(norm);

        for cx in min_x..=max_x {
            for cy in min_y..=max_y {
                self.cells.entry((cx, cy)).or_default().push(id);
            }
        }
    }

    pub fn remove(&mut self, id: NodeId) {
        if let Some(rect) = self.node_bounds.remove(&id) {
            let (min_x, max_x, min_y, max_y) = self.get_cell_range(rect);
            for cx in min_x..=max_x {
                for cy in min_y..=max_y {
                    if let Some(bucket) = self.cells.get_mut(&(cx, cy)) {
                        bucket.retain(|&node_id| node_id != id);
                    }
                }
            }
        }
    }

    pub fn update(&mut self, id: NodeId, rect: Rect) {
        self.remove(id);
        self.insert(id, rect);
    }

    pub fn rebuild<'a>(&mut self, nodes: impl IntoIterator<Item = &'a Node>) {
        self.cells.clear();
        self.node_bounds.clear();
        for node in nodes {
            if node.visible {
                self.insert(node.id, node.rect);
            }
        }
    }

    /// Query all node candidates that intersect a given bounding box (e.g. screen viewport or marquee selection)
    pub fn query_rect(&self, bbox: Rect) -> Vec<NodeId> {
        let norm = bbox.normalized();
        let (min_x, max_x, min_y, max_y) = self.get_cell_range(norm);
        let mut candidates = HashSet::new();

        for cx in min_x..=max_x {
            for cy in min_y..=max_y {
                if let Some(bucket) = self.cells.get(&(cx, cy)) {
                    for &id in bucket {
                        candidates.insert(id);
                    }
                }
            }
        }

        // Exact geometric intersection filter
        let mut result = Vec::with_capacity(candidates.len());
        for id in candidates {
            if let Some(&bounds) = self.node_bounds.get(&id) {
                if bounds.intersects(&norm) {
                    result.push(id);
                }
            }
        }
        result
    }

    /// Query all node candidates containing a specific world point (e.g. mouse cursor)
    pub fn query_point(&self, pt: Vec2) -> Vec<NodeId> {
        let cx = self.to_cell_coord(pt.x);
        let cy = self.to_cell_coord(pt.y);

        let bucket = match self.cells.get(&(cx, cy)) {
            Some(b) => b,
            None => return Vec::new(),
        };

        let mut result = Vec::new();
        for &id in bucket {
            if let Some(&bounds) = self.node_bounds.get(&id) {
                if bounds.contains_point(pt) {
                    result.push(id);
                }
            }
        }
        result
    }
}
