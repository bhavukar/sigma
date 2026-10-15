use crate::document::Document;
use crate::node::{Node, NodeKind};
use crate::style::Fill;

pub fn export_to_svg(doc: &Document) -> String {
    let bbox = doc
        .selection_bounding_box(&doc.nodes.iter().map(|n| n.id).collect::<Vec<_>>())
        .unwrap_or(crate::math::Rect {
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 600.0,
        });

    let pad = 20.0;
    let view_x = bbox.x - pad;
    let view_y = bbox.y - pad;
    let view_w = bbox.width + pad * 2.0;
    let view_h = bbox.height + pad * 2.0;

    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{} {} {} {}" width="{}" height="{}">"#,
        view_x, view_y, view_w, view_h, view_w, view_h
    );
    svg.push('\n');

    for node in &doc.nodes {
        render_node_svg(node, &mut svg);
    }

    svg.push_str("</svg>");
    svg
}

pub fn export_node_to_svg(node: &Node) -> String {
    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}">"#,
        node.rect.width, node.rect.height, node.rect.width, node.rect.height
    );
    svg.push('\n');
    let mut cloned = node.clone();
    cloned.rect.x = 0.0;
    cloned.rect.y = 0.0;
    render_node_svg(&cloned, &mut svg);
    svg.push_str("</svg>");
    svg
}

fn render_node_svg(node: &Node, svg: &mut String) {
    if !node.visible {
        return;
    }

    let fill_attr = if let Some(fill) = node.fills.first() {
        match fill {
            Fill::Solid { color, opacity } => {
                format!(r#"fill="{}" fill-opacity="{:.2}""#, color.to_hex(), opacity)
            }
        }
    } else {
        r#"fill="none""#.to_string()
    };

    let stroke_attr = if let Some(stroke) = node.strokes.first() {
        format!(
            r#"stroke="{}" stroke-width="{}" stroke-opacity="{:.2}""#,
            stroke.color.to_hex(),
            stroke.width,
            stroke.opacity
        )
    } else {
        "".to_string()
    };

    match node.kind {
        NodeKind::Frame | NodeKind::Rectangle => {
            let rx = node.corner_radius.top_left;
            svg.push_str(&format!(
                r#"  <rect x="{}" y="{}" width="{}" height="{}" rx="{}" {} {} />"#,
                node.rect.x, node.rect.y, node.rect.width, node.rect.height, rx, fill_attr, stroke_attr
            ));
            svg.push('\n');

            for child in &node.children {
                render_node_svg(child, svg);
            }
        }
        NodeKind::Ellipse => {
            let cx = node.rect.x + node.rect.width * 0.5;
            let cy = node.rect.y + node.rect.height * 0.5;
            let rx = node.rect.width * 0.5;
            let ry = node.rect.height * 0.5;
            svg.push_str(&format!(
                r#"  <ellipse cx="{}" cy="{}" rx="{}" ry="{}" {} {} />"#,
                cx, cy, rx, ry, fill_attr, stroke_attr
            ));
            svg.push('\n');
        }
        NodeKind::Text => {
            if let Some(text_props) = &node.text_props {
                let baseline_y = node.rect.y + text_props.font_size;
                svg.push_str(&format!(
                    r#"  <text x="{}" y="{}" font-size="{}" font-family="Inter, sans-serif" {}>{}</text>"#,
                    node.rect.x, baseline_y, text_props.font_size, fill_attr, text_props.content
                ));
                svg.push('\n');
            }
        }
        NodeKind::Line => {
            let x1 = node.rect.x;
            let y1 = node.rect.y + node.rect.height * 0.5;
            let x2 = node.rect.x + node.rect.width;
            let y2 = node.rect.y + node.rect.height * 0.5;
            svg.push_str(&format!(
                r#"  <line x1="{}" y1="{}" x2="{}" y2="{}" {} />"#,
                x1, y1, x2, y2, stroke_attr
            ));
            svg.push('\n');
        }
        NodeKind::Arrow => {
            let x1 = node.rect.x;
            let y1 = node.rect.y + node.rect.height * 0.5;
            let x2 = node.rect.x + node.rect.width;
            let y2 = node.rect.y + node.rect.height * 0.5;
            svg.push_str(&format!(
                r#"  <line x1="{}" y1="{}" x2="{}" y2="{}" {} />"#,
                x1, y1, x2, y2, stroke_attr
            ));
            svg.push('\n');
        }
        NodeKind::Star => {
            let cx = node.rect.x + node.rect.width * 0.5;
            let cy = node.rect.y + node.rect.height * 0.5;
            let r_outer = (node.rect.width * 0.5).min(node.rect.height * 0.5);
            let r_inner = r_outer * 0.382;
            let mut pts = String::new();
            for i in 0..10 {
                let angle = (i as f32) * std::f32::consts::PI / 5.0 - std::f32::consts::FRAC_PI_2;
                let r = if i % 2 == 0 { r_outer } else { r_inner };
                let px = cx + angle.cos() * r;
                let py = cy + angle.sin() * r;
                pts.push_str(&format!("{:.1},{:.1} ", px, py));
            }
            svg.push_str(&format!(
                r#"  <polygon points="{}" {} {} />"#,
                pts.trim_end(), fill_attr, stroke_attr
            ));
            svg.push('\n');
        }
        NodeKind::Polygon => {
            let cx = node.rect.x + node.rect.width * 0.5;
            let top_y = node.rect.y;
            let bot_y = node.rect.y + node.rect.height;
            let left_x = node.rect.x;
            let right_x = node.rect.x + node.rect.width;
            svg.push_str(&format!(
                r#"  <polygon points="{:.1},{:.1} {:.1},{:.1} {:.1},{:.1}" {} {} />"#,
                cx, top_y, right_x, bot_y, left_x, bot_y, fill_attr, stroke_attr
            ));
            svg.push('\n');
        }
    }
}

pub fn export_to_css(node: &Node) -> String {
    let mut css = String::new();
    css.push_str(&format!("/* CSS for {} */\n", node.name));
    css.push_str(&format!("width: {:.0}px;\n", node.rect.width));
    css.push_str(&format!("height: {:.0}px;\n", node.rect.height));

    if let Some(fill) = node.fills.first() {
        match fill {
            Fill::Solid { color, opacity } => {
                if *opacity < 1.0 {
                    css.push_str(&format!(
                        "background-color: rgba({}, {}, {}, {:.2});\n",
                        color.r, color.g, color.b, opacity
                    ));
                } else {
                    css.push_str(&format!("background-color: {};\n", color.to_hex()));
                }
            }
        }
    }

    if node.corner_radius.top_left > 0.0 {
        css.push_str(&format!("border-radius: {:.0}px;\n", node.corner_radius.top_left));
    }

    if let Some(stroke) = node.strokes.first() {
        css.push_str(&format!(
            "border: {:.0}px solid {};\n",
            stroke.width,
            stroke.color.to_hex()
        ));
    }

    if node.auto_layout.enabled {
        css.push_str("display: flex;\n");
        let dir = match node.auto_layout.direction {
            crate::layout::LayoutDirection::Horizontal => "row",
            crate::layout::LayoutDirection::Vertical => "column",
        };
        css.push_str(&format!("flex-direction: {};\n", dir));
        css.push_str(&format!("gap: {:.0}px;\n", node.auto_layout.gap));
        css.push_str(&format!(
            "padding: {:.0}px {:.0}px {:.0}px {:.0}px;\n",
            node.auto_layout.padding.top,
            node.auto_layout.padding.right,
            node.auto_layout.padding.bottom,
            node.auto_layout.padding.left
        ));
    }

    css
}
