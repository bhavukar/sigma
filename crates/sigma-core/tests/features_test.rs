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
