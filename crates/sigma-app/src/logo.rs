use egui::{Color32, Pos2, Rect, Stroke, Ui, Vec2};

/// Renders the iconic Sigma geometric vector brandmark
pub fn render_sigma_mark_painter(painter: &egui::Painter, rect: Rect) {
    let w = rect.width();
    let h = rect.height();
    let r = rect.min;

    // Background tile: Deep obsidian with subtle inner gradient effect
    let tile_radius = w * 0.26;
    painter.rect_filled(rect, tile_radius, Color32::from_rgb(18, 19, 24));
    painter.rect_stroke(
        rect,
        tile_radius,
        Stroke::new(1.0, Color32::from_rgb(44, 46, 58)),
        egui::StrokeKind::Outside,
    );

    // Vector geometry coordinates for the modern Sigma (Σ) prism glyph:
    // 1. Top bar: Electric Cyan / Blue (#00d2ff -> #3b82f6)
    let p_top_l = Pos2::new(r.x + w * 0.24, r.y + h * 0.24);
    let p_top_r = Pos2::new(r.x + w * 0.76, r.y + h * 0.24);
    let p_top_inner = Pos2::new(r.x + w * 0.62, r.y + h * 0.38);
    let p_top_mid = Pos2::new(r.x + w * 0.36, r.y + h * 0.38);
    painter.add(egui::Shape::convex_polygon(
        vec![p_top_l, p_top_r, p_top_inner, p_top_mid],
        Color32::from_rgb(14, 165, 233), // Sky blue
        Stroke::NONE,
    ));

    // 2. Upper diagonal to center vertex: Electric Violet (#8b5cf6)
    let p_center = Pos2::new(r.x + w * 0.44, r.y + h * 0.50);
    painter.add(egui::Shape::convex_polygon(
        vec![p_top_r, p_center, p_top_inner],
        Color32::from_rgb(139, 92, 246), // Violet
        Stroke::NONE,
    ));

    // 3. Lower diagonal from center vertex: Vibrant Rose (#f43f5e)
    let p_bot_r = Pos2::new(r.x + w * 0.76, r.y + h * 0.76);
    let p_bot_inner = Pos2::new(r.x + w * 0.62, r.y + h * 0.62);
    painter.add(egui::Shape::convex_polygon(
        vec![p_center, p_bot_r, p_bot_inner],
        Color32::from_rgb(244, 63, 94), // Rose
        Stroke::NONE,
    ));

    // 4. Bottom bar: Emerald / Mint (#10b981)
    let p_bot_l = Pos2::new(r.x + w * 0.24, r.y + h * 0.76);
    let p_bot_mid = Pos2::new(r.x + w * 0.36, r.y + h * 0.62);
    painter.add(egui::Shape::convex_polygon(
        vec![p_bot_l, p_bot_mid, p_bot_inner, p_bot_r],
        Color32::from_rgb(16, 185, 129), // Emerald
        Stroke::NONE,
    ));

    // Central accent gem: Small diamond highlight
    let gem_c = Pos2::new(r.x + w * 0.44, r.y + h * 0.50);
    let gem_sz = w * 0.06;
    painter.add(egui::Shape::convex_polygon(
        vec![
            Pos2::new(gem_c.x, gem_c.y - gem_sz),
            Pos2::new(gem_c.x + gem_sz, gem_c.y),
            Pos2::new(gem_c.x, gem_c.y + gem_sz),
            Pos2::new(gem_c.x - gem_sz, gem_c.y),
        ],
        Color32::WHITE,
        Stroke::NONE,
    ));
}

/// Renders the complete brand lockup (Icon + Typography + PRO badge)
pub fn render_brand_lockup(ui: &mut Ui, icon_size: f32) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(icon_size, icon_size), egui::Sense::click());
    let painter = ui.painter_at(rect);
    render_sigma_mark_painter(&painter, rect);

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("SIGMA")
                .size(14.0)
                .strong()
                .color(Color32::from_rgb(245, 245, 247)),
        );
        ui.add_space(2.0);
        let badge_fill = Color32::from_rgb(30, 32, 42);
        let badge_stroke = Stroke::new(1.0, Color32::from_rgb(14, 165, 233));
        let (badge_rect, _) = ui.allocate_exact_size(Vec2::new(34.0, 16.0), egui::Sense::hover());
        let p = ui.painter_at(badge_rect);
        p.rect_filled(badge_rect, 4.0, badge_fill);
        p.rect_stroke(badge_rect, 4.0, badge_stroke, egui::StrokeKind::Outside);
        p.text(
            badge_rect.center(),
            egui::Align2::CENTER_CENTER,
            "PRO",
            egui::FontId::proportional(9.5),
            Color32::from_rgb(56, 189, 248),
        );
    });

    resp
}
