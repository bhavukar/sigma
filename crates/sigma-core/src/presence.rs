use crate::color::Color;
use crate::document::Document;
use crate::math::{Rect, Vec2};
use crate::node::NodeId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerState {
    pub id: String,
    pub name: String,
    pub color: Color,
    pub cursor: Vec2,
    pub selected_ids: Vec<NodeId>,
}

impl PeerState {
    pub fn new(id: &str, name: &str, color: Color) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            color,
            cursor: Vec2::ZERO,
            selected_ids: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SyncMessage {
    #[serde(rename = "join")]
    Join {
        user_id: String,
        user_name: String,
        color: Color,
    },
    #[serde(rename = "cursor")]
    Cursor {
        user_id: String,
        x: f32,
        y: f32,
    },
    #[serde(rename = "selection")]
    Selection {
        user_id: String,
        selected_ids: Vec<NodeId>,
    },
    #[serde(rename = "node_transform")]
    NodeTransform {
        user_id: String,
        node_id: NodeId,
        rect: Rect,
    },
    #[serde(rename = "nodes_move")]
    NodesMove {
        user_id: String,
        node_ids: Vec<NodeId>,
        delta: Vec2,
    },
    #[serde(rename = "doc_update")]
    DocUpdate {
        user_id: String,
        doc: Document,
    },
    #[serde(rename = "room_state")]
    RoomState {
        peers: Vec<PeerState>,
        doc: Option<Document>,
    },
    #[serde(rename = "leave")]
    Leave {
        user_id: String,
    },
}

pub fn get_figma_palette(idx: usize) -> Color {
    let colors = [
        Color::rgb(255, 90, 95),   // Coral
        Color::rgb(10, 132, 255),  // Figma Blue
        Color::rgb(162, 89, 255),  // Purple
        Color::rgb(10, 207, 131),  // Green
        Color::rgb(255, 168, 0),   // Amber
        Color::rgb(242, 78, 30),   // Orange-Red
    ];
    colors[idx % colors.len()]
}
