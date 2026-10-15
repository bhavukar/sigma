use sigma_core::{
    export_node_to_svg, export_to_svg, Color, Document, Node, NodeKind, Rect, Vec2,
};

#[test]
fn test_layer_ordering() {
    let mut doc = Document::new("Test");
    let id1 = doc.add_node(Node::new_rectangle(0, "A", Rect::new(0.0, 0.0, 10.0, 10.0), Color::WHITE));
    let id2 = doc.add_node(Node::new_rectangle(0, "B", Rect::new(10.0, 0.0, 10.0, 10.0), Color::WHITE));
    let id3 = doc.add_node(Node::new_rectangle(0, "C", Rect::new(20.0, 0.0, 10.0, 10.0), Color::WHITE));

    assert_eq!(doc.nodes.iter().map(|n| n.id).collect::<Vec<_>>(), vec![id1, id2, id3]);

    // Send id3 to back
    doc.send_to_back(&[id3]);
    assert_eq!(doc.nodes.iter().map(|n| n.id).collect::<Vec<_>>(), vec![id3, id1, id2]);

    // Bring id3 to front
    doc.bring_to_front(&[id3]);
    assert_eq!(doc.nodes.iter().map(|n| n.id).collect::<Vec<_>>(), vec![id1, id2, id3]);

    // Send id2 backward
    doc.send_backward(&[id2]);
    assert_eq!(doc.nodes.iter().map(|n| n.id).collect::<Vec<_>>(), vec![id2, id1, id3]);

    // Bring id2 forward
    doc.bring_forward(&[id2]);
    assert_eq!(doc.nodes.iter().map(|n| n.id).collect::<Vec<_>>(), vec![id1, id2, id3]);
}

#[test]
fn test_distribute_selection() {
    let mut doc = Document::new("Distribute");
    let id1 = doc.add_node(Node::new_rectangle(0, "A", Rect::new(0.0, 0.0, 10.0, 10.0), Color::WHITE));
    let id2 = doc.add_node(Node::new_rectangle(0, "B", Rect::new(20.0, 0.0, 10.0, 10.0), Color::WHITE));
    let id3 = doc.add_node(Node::new_rectangle(0, "C", Rect::new(100.0, 0.0, 10.0, 10.0), Color::WHITE));

    doc.distribute_selection(&[id1, id2, id3], true);

    let n1 = doc.find_node(id1).unwrap();
    let n2 = doc.find_node(id2).unwrap();
    let n3 = doc.find_node(id3).unwrap();

    assert_eq!(n1.rect.x, 0.0);
    assert_eq!(n3.rect.x, 100.0);
    // Total span: 110. Item sizes: 30. Remaining: 80. Gap: 40.
    // So n2 should be at 0 + 10 + 40 = 50.0.
    assert_eq!(n2.rect.x, 50.0);
}

#[test]
fn test_group_and_ungroup() {
    let mut doc = Document::new("Grouping");
    let id1 = doc.add_node(Node::new_rectangle(0, "Item 1", Rect::new(10.0, 10.0, 50.0, 50.0), Color::WHITE));
    let id2 = doc.add_node(Node::new_rectangle(0, "Item 2", Rect::new(100.0, 100.0, 50.0, 50.0), Color::WHITE));

    let group_id = doc.group_selection(&[id1, id2]).expect("Group creation failed");
    assert_eq!(doc.nodes.len(), 1);

    let group_node = doc.find_node(group_id).unwrap();
    assert_eq!(group_node.kind, NodeKind::Frame);
    assert_eq!(group_node.children.len(), 2);
    assert_eq!(group_node.rect.x, 10.0);
    assert_eq!(group_node.rect.y, 10.0);
    assert_eq!(group_node.rect.width, 140.0);
    assert_eq!(group_node.rect.height, 140.0);

    // Ungroup
    let unpacked = doc.ungroup_selection(&[group_id]);
    assert_eq!(unpacked.len(), 2);
    assert_eq!(doc.nodes.len(), 2);

    let n1 = doc.find_node(id1).unwrap();
    let n2 = doc.find_node(id2).unwrap();
    assert_eq!(n1.rect.x, 10.0);
    assert_eq!(n2.rect.x, 100.0);
}

#[test]
fn test_frame_preset_and_components() {
    let mut doc = Document::new("Presets");
    let iphone_id = doc.add_frame_preset("iPhone 16 Pro", 393.0, 852.0, Vec2::new(400.0, 400.0));
    let iphone = doc.find_node(iphone_id).unwrap();
    assert_eq!(iphone.rect.width, 393.0);
    assert_eq!(iphone.rect.height, 852.0);

    let btn_id = doc.insert_component("button", Vec2::new(100.0, 100.0));
    assert!(doc.find_node(btn_id).is_some());

    let card_id = doc.insert_component("card", Vec2::new(300.0, 300.0));
    assert!(doc.find_node(card_id).is_some());
}

#[test]
fn test_shapes_and_svg_export() {
    let mut doc = Document::new("Shapes");
    let star_id = doc.alloc_id();
    let star = Node::new_star(star_id, "Star 1", Rect::new(0.0, 0.0, 100.0, 100.0), Color::rgb(255, 180, 0));
    doc.nodes.push(star);

    let arrow_id = doc.alloc_id();
    let arrow = Node::new_arrow(arrow_id, "Arrow 1", Rect::new(50.0, 50.0, 120.0, 20.0), Color::FIGMA_BLUE);
    doc.nodes.push(arrow);

    let doc_svg = export_to_svg(&doc);
    assert!(doc_svg.contains("<polygon points="));
    assert!(doc_svg.contains("<line x1="));

    let star_node = doc.find_node(star_id).unwrap();
    let node_svg = export_node_to_svg(star_node);
    assert!(node_svg.contains("<svg xmlns=\"http://www.w3.org/2000/svg\""));
    assert!(node_svg.contains("<polygon points="));
}

#[test]
fn test_security_xss_sanitization() {
    use sigma_core::xml_escape;

    // Test raw xml_escape function
    let payload = "<script>alert('xss')</script> & \"injection\"";
    let sanitized = xml_escape(payload);
    assert!(!sanitized.contains('<'));
    assert!(!sanitized.contains('>'));
    assert!(!sanitized.contains('"'));
    assert!(!sanitized.contains('\''));
    assert!(sanitized.contains("&lt;script&gt;alert(&#39;xss&#39;)&lt;/script&gt; &amp; &quot;injection&quot;"));

    // Test SVG export with malicious content inside text node
    let mut doc = Document::new("SecurityTest");
    let text_id = doc.alloc_id();
    let malicious_text = Node::new_text(
        text_id,
        "Malicious <tag>",
        Vec2::new(10.0, 10.0),
        "<img src=x onerror=alert(1)>",
        16.0,
        Color::WHITE,
    );
    doc.nodes.push(malicious_text);

    let svg = export_to_svg(&doc);
    assert!(!svg.contains("<img src=x"));
    assert!(svg.contains("&lt;img src=x onerror=alert(1)&gt;"));
}

#[test]
fn test_security_nan_clamp_safety() {
    use sigma_core::safe_clamp;

    // Standard f32::clamp(min, max) panics on NaN in Rust!
    // safe_clamp must never panic and fall back safely.
    let clamped_nan = safe_clamp(f32::NAN, 10.0, 50.0);
    assert_eq!(clamped_nan, 10.0);

    let clamped_pos_inf = safe_clamp(f32::INFINITY, 10.0, 50.0);
    assert_eq!(clamped_pos_inf, 50.0);

    let clamped_neg_inf = safe_clamp(f32::NEG_INFINITY, 10.0, 50.0);
    assert_eq!(clamped_neg_inf, 10.0);

    let clamped_valid = safe_clamp(25.0, 10.0, 50.0);
    assert_eq!(clamped_valid, 25.0);

    // Vec2 safe_div
    let v = Vec2::new(10.0, 20.0);
    let divided_zero = v.safe_div(0.0);
    assert!(divided_zero.is_finite());
}

#[test]
fn test_spatial_index_and_hit_testing() {
    let mut doc = Document::new("Spatial");

    // Add nodes in different quadrants
    let id1 = doc.add_node(Node::new_rectangle(0, "Quadrant 1", Rect::new(50.0, 50.0, 100.0, 100.0), Color::WHITE));
    let id2 = doc.add_node(Node::new_rectangle(0, "Quadrant 2", Rect::new(1000.0, 1000.0, 100.0, 100.0), Color::WHITE));
    let id3 = doc.add_node(Node::new_rectangle(0, "Quadrant 1 Overlap", Rect::new(80.0, 80.0, 50.0, 50.0), Color::WHITE));

    // Spatial hit test for id1 & id3: id3 is on top of id1 at (90, 90)
    let hit_overlap = doc.hit_test(Vec2::new(90.0, 90.0));
    assert_eq!(hit_overlap, Some(id3));

    // Hit test id1 where id3 does not overlap
    let hit_id1 = doc.hit_test(Vec2::new(60.0, 60.0));
    assert_eq!(hit_id1, Some(id1));

    // Hit test id2
    let hit_id2 = doc.hit_test(Vec2::new(1050.0, 1050.0));
    assert_eq!(hit_id2, Some(id2));

    // Hit test empty space
    let hit_empty = doc.hit_test(Vec2::new(500.0, 500.0));
    assert_eq!(hit_empty, None);

    // Marquee query around Quadrant 1
    let marquee = Rect::new(0.0, 0.0, 200.0, 200.0);
    let selected = doc.query_marquee_box(marquee);
    assert!(selected.contains(&id1));
    assert!(selected.contains(&id3));
    assert!(!selected.contains(&id2));
}

#[test]
fn test_delta_sync_serialization() {
    use sigma_core::SyncMessage;

    // NodeTransform delta
    let transform_msg = SyncMessage::NodeTransform {
        user_id: "user-123".to_string(),
        node_id: 42,
        rect: Rect::new(100.0, 200.0, 300.0, 400.0),
    };
    let json = serde_json::to_string(&transform_msg).expect("Serialize transform");
    assert!(json.len() < 120, "Delta message must be compact");
    let deserialized: SyncMessage = serde_json::from_str(&json).expect("Deserialize transform");
    match deserialized {
        SyncMessage::NodeTransform { user_id, node_id, rect } => {
            assert_eq!(user_id, "user-123");
            assert_eq!(node_id, 42);
            assert_eq!(rect.width, 300.0);
        }
        _ => panic!("Expected NodeTransform"),
    }

    // NodesMove delta
    let move_msg = SyncMessage::NodesMove {
        user_id: "user-456".to_string(),
        node_ids: vec![1, 2, 3],
        delta: Vec2::new(15.5, -30.0),
    };
    let json_move = serde_json::to_string(&move_msg).expect("Serialize move");
    assert!(json_move.len() < 120, "Delta move must be compact");
    let deserialized_move: SyncMessage = serde_json::from_str(&json_move).expect("Deserialize move");
    match deserialized_move {
        SyncMessage::NodesMove { user_id, node_ids, delta } => {
            assert_eq!(user_id, "user-456");
            assert_eq!(node_ids, vec![1, 2, 3]);
            assert_eq!(delta.x, 15.5);
        }
        _ => panic!("Expected NodesMove"),
    }
}

#[test]
fn test_advanced_layer_reordering() {
    let mut doc = Document::new("AdvancedReordering");
    let a = doc.add_node(Node::new_rectangle(0, "A", Rect::new(0.0, 0.0, 10.0, 10.0), Color::WHITE));
    let b = doc.add_node(Node::new_rectangle(0, "B", Rect::new(10.0, 0.0, 10.0, 10.0), Color::WHITE));
    let c = doc.add_node(Node::new_rectangle(0, "C", Rect::new(20.0, 0.0, 10.0, 10.0), Color::WHITE));

    // Initial order: [a, b, c]
    assert_eq!(doc.nodes.iter().map(|n| n.id).collect::<Vec<_>>(), vec![a, b, c]);

    // Move 'a' up: swaps with 'b' -> [b, a, c]
    assert!(doc.move_node_up(a));
    assert_eq!(doc.nodes.iter().map(|n| n.id).collect::<Vec<_>>(), vec![b, a, c]);

    // Move 'a' down: swaps back with 'b' -> [a, b, c]
    assert!(doc.move_node_down(a));
    assert_eq!(doc.nodes.iter().map(|n| n.id).collect::<Vec<_>>(), vec![a, b, c]);

    // Group [a, b] into a frame
    let frame_id = doc.group_selection(&[a, b]).unwrap();
    assert_eq!(doc.nodes.len(), 2); // [c, frame]

    // Frame children: [a, b]
    let frame = doc.find_node(frame_id).unwrap();
    assert_eq!(frame.children.iter().map(|n| n.id).collect::<Vec<_>>(), vec![a, b]);

    // Move 'a' up inside frame -> [b, a]
    assert!(doc.move_node_up(a));
    let frame_after = doc.find_node(frame_id).unwrap();
    assert_eq!(frame_after.children.iter().map(|n| n.id).collect::<Vec<_>>(), vec![b, a]);

    // Extract 'a' from parent frame to root
    assert!(doc.extract_from_parent(a));
    assert_eq!(doc.find_parent_frame_id(a), None);
    assert_eq!(doc.nodes.len(), 3);

    // Move 'c' into frame
    assert!(doc.move_node_into_frame(c, frame_id));
    assert_eq!(doc.find_parent_frame_id(c), Some(frame_id));
}

#[test]
fn test_production_ui_kits() {
    let mut doc = Document::new("UI Kits Test");
    let origin = Vec2::new(0.0, 0.0);

    let fintech_id = doc.insert_fintech_mobile(origin);
    let saas_id = doc.insert_saas_hero(origin);
    let pricing_id = doc.insert_pricing_card(origin);
    let auth_id = doc.insert_auth_card(origin);
    let btn_id = doc.insert_button_kit(origin);
    let badge_id = doc.insert_badge_kit(origin);

    assert_eq!(doc.nodes.len(), 6);

    let fintech = doc.find_node(fintech_id).unwrap();
    assert_eq!(fintech.kind, NodeKind::Frame);
    assert!(fintech.children.len() >= 10);
    assert_eq!(fintech.rect.width, 393.0);
    assert_eq!(fintech.rect.height, 852.0);

    let saas = doc.find_node(saas_id).unwrap();
    assert_eq!(saas.kind, NodeKind::Frame);
    assert!(saas.children.len() >= 10);
    assert_eq!(saas.rect.width, 1200.0);

    let pricing = doc.find_node(pricing_id).unwrap();
    assert_eq!(pricing.kind, NodeKind::Frame);
    assert!(pricing.children.len() >= 8);

    let auth = doc.find_node(auth_id).unwrap();
    assert_eq!(auth.kind, NodeKind::Frame);
    assert!(auth.children.len() >= 8);

    let btn_kit = doc.find_node(btn_id).unwrap();
    assert_eq!(btn_kit.kind, NodeKind::Frame);
    assert!(btn_kit.children.len() >= 6);

    let badge_kit = doc.find_node(badge_id).unwrap();
    assert_eq!(badge_kit.kind, NodeKind::Frame);
    assert!(badge_kit.children.len() >= 6);

    assert_eq!(doc.spatial_index.len(), 6);
}

