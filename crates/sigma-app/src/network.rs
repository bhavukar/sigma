use sigma_core::{Color, Document, NodeId, Rect, SyncMessage, Vec2};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct NetworkManager {
    pub room_id: String,
    pub user_id: String,
    pub user_name: String,
    pub user_color: Color,
    pub incoming: Arc<Mutex<Vec<SyncMessage>>>,
    #[cfg(target_arch = "wasm32")]
    ws: Option<web_sys::WebSocket>,
}

impl NetworkManager {
    pub fn new(room_id: &str, user_name: &str, user_color: Color) -> Self {
        let rand_suffix = (js_sys_random() * 10000.0) as u32;
        let user_id = format!("usr_{}", rand_suffix);
        let incoming = Arc::new(Mutex::new(Vec::new()));

        #[cfg(target_arch = "wasm32")]
        let ws = {
            let ws_url = format!(
                "wss://sigma-sync.bhavuk-arora03.workers.dev/ws?room={}",
                room_id
            );
            match web_sys::WebSocket::new(&ws_url) {
                Ok(socket) => {
                    let inc_clone = Arc::clone(&incoming);
                    let uid = user_id.clone();
                    let uname = user_name.to_string();
                    let ucolor = user_color;
                    let sock_clone = socket.clone();

                    // onopen handler: send join message
                    let onopen_callback = wasm_bindgen::closure::Closure::<dyn FnMut()>::new(move || {
                        let join_msg = SyncMessage::Join {
                            user_id: uid.clone(),
                            user_name: uname.clone(),
                            color: ucolor,
                        };
                        if let Ok(json) = serde_json::to_string(&join_msg) {
                            let _ = sock_clone.send_with_str(&json);
                        }
                    });
                    socket.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
                    onopen_callback.forget();

                    // onmessage handler: parse message and enqueue
                    let onmessage_callback = wasm_bindgen::closure::Closure::<
                        dyn FnMut(web_sys::MessageEvent),
                    >::new(move |e: web_sys::MessageEvent| {
                        if let Some(text) = e.data().as_string() {
                            if let Ok(msg) = serde_json::from_str::<SyncMessage>(&text) {
                                if let Ok(mut lock) = inc_clone.lock() {
                                    lock.push(msg);
                                }
                            }
                        }
                    });
                    socket.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
                    onmessage_callback.forget();

                    Some(socket)
                }
                Err(e) => {
                    log::warn!("Failed to create WebSocket: {:?}", e);
                    None
                }
            }
        };

        Self {
            room_id: room_id.to_string(),
            user_id,
            user_name: user_name.to_string(),
            user_color,
            incoming,
            #[cfg(target_arch = "wasm32")]
            ws,
        }
    }

    pub fn poll_incoming(&self) -> Vec<SyncMessage> {
        if let Ok(mut lock) = self.incoming.lock() {
            let messages = lock.clone();
            lock.clear();
            messages
        } else {
            Vec::new()
        }
    }

    pub fn send_cursor(&self, x: f32, y: f32) {
        let msg = SyncMessage::Cursor {
            user_id: self.user_id.clone(),
            x,
            y,
        };
        self.send_message(&msg);
    }

    pub fn send_selection(&self, selected_ids: &[NodeId]) {
        let msg = SyncMessage::Selection {
            user_id: self.user_id.clone(),
            selected_ids: selected_ids.to_vec(),
        };
        self.send_message(&msg);
    }

    pub fn send_node_transform(&self, node_id: NodeId, rect: Rect) {
        let msg = SyncMessage::NodeTransform {
            user_id: self.user_id.clone(),
            node_id,
            rect,
        };
        self.send_message(&msg);
    }

    pub fn send_nodes_move(&self, node_ids: &[NodeId], delta: Vec2) {
        let msg = SyncMessage::NodesMove {
            user_id: self.user_id.clone(),
            node_ids: node_ids.to_vec(),
            delta,
        };
        self.send_message(&msg);
    }

    pub fn send_doc_update(&self, doc: &Document) {
        let msg = SyncMessage::DocUpdate {
            user_id: self.user_id.clone(),
            doc: doc.clone(),
        };
        self.send_message(&msg);
    }

    pub fn update_user_profile(&mut self, user_name: &str, user_color: Color) {
        self.user_name = user_name.to_string();
        self.user_color = user_color;
        let join_msg = SyncMessage::Join {
            user_id: self.user_id.clone(),
            user_name: self.user_name.clone(),
            color: self.user_color,
        };
        self.send_message(&join_msg);
    }

    fn send_message(&self, msg: &SyncMessage) {
        #[cfg(target_arch = "wasm32")]
        if let Some(ws) = &self.ws {
            if ws.ready_state() == web_sys::WebSocket::OPEN {
                if let Ok(json) = serde_json::to_string(msg) {
                    let _ = ws.send_with_str(&json);
                }
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _ = msg;
    }
}

fn js_sys_random() -> f64 {
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

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
