use crate::canvas::CanvasState;
use egui::{Color32, DragValue, RichText, ScrollArea, Slider, Stroke as EguiStroke, Ui, Vec2 as EguiVec2};
use sigma_core::{
    export_node_to_svg, export_to_css, AlignmentMode, Color, Document, Effect, NodeKind, Stroke,
    StrokeAlign, StrokeStyle, TextAlign, TextCase, TextDecoration,
};

pub fn render_inspector(ui: &mut Ui, doc: &mut Document, state: &mut CanvasState) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(8.0);

        if state.selected_ids.is_empty() {
            // Document / Canvas Properties
            ui.label(RichText::new("CANVAS SETTINGS").strong().size(11.0).color(Color32::from_rgb(140, 140, 150)));
            ui.separator();
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label("Background");
                let mut c = [
                    doc.canvas_background.r as f32 / 255.0,
                    doc.canvas_background.g as f32 / 255.0,
                    doc.canvas_background.b as f32 / 255.0,
                ];
                if ui.color_edit_button_rgb(&mut c).changed() {
                    doc.canvas_background = Color::rgb(
                        (c[0] * 255.0) as u8,
                        (c[1] * 255.0) as u8,
                        (c[2] * 255.0) as u8,
                    );
                }
            });

            // Quick Canvas Background Swatches
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                let presets = [
                    ("#1E1E1E", "Dark (Figma)"),
                    ("#121212", "AMOLED"),
                    ("#F5F5F7", "Light (Apple)"),
                    ("#FFFFFF", "Pure White"),
                    ("#0F172A", "Slate Navy"),
                ];
                for (hex, tip) in presets {
                    let col = Color::from_hex(hex).unwrap_or(Color::BLACK);
                    let c32 = Color32::from_rgb(col.r, col.g, col.b);
                    let (rect, resp) = ui.allocate_exact_size(EguiVec2::splat(18.0), egui::Sense::click());
                    let p = ui.painter_at(rect);
                    p.rect_filled(rect, 3.0, c32);
                    p.rect_stroke(rect, 3.0, EguiStroke::new(1.0, Color32::from_rgb(80, 80, 80)), egui::StrokeKind::Outside);
                    if resp.on_hover_text(tip).clicked() {
                        doc.canvas_background = col;
                    }
                }
            });

            ui.add_space(14.0);
            ui.label(RichText::new("ARTBOARD PRESETS").strong().size(11.0).color(Color32::from_rgb(140, 140, 150)));
            ui.separator();
            let center = sigma_core::Vec2::new(-state.pan.x + 350.0, -state.pan.y + 350.0);
            ui.horizontal_wrapped(|ui| {
                if ui.button("📱 iPhone 16 Pro").clicked() {
                    let id = doc.add_frame_preset("iPhone 16 Pro", 393.0, 852.0, center);
                    state.selected_ids = vec![id];
                }
                if ui.button("💻 Desktop HD").clicked() {
                    let id = doc.add_frame_preset("Desktop HD", 1440.0, 1024.0, center);
                    state.selected_ids = vec![id];
                }
                if ui.button("📸 Social Post").clicked() {
                    let id = doc.add_frame_preset("Instagram Post", 1080.0, 1080.0, center);
                    state.selected_ids = vec![id];
                }
            });

            ui.add_space(14.0);
            ui.label(RichText::new("KEYBOARD SHORTCUTS").strong().size(11.0).color(Color32::from_rgb(140, 140, 150)));
            ui.separator();
            ui.label("• V = Select / Move Tool");
            ui.label("• F = Frame / Artboard Tool");
            ui.label("• R = Rectangle | O = Ellipse");
            ui.label("• L = Line | A = Arrow");
            ui.label("• S = Star | P = Polygon");
            ui.label("• T = Text Tool | H = Hand");
            ui.label("• Space + Drag = Pan Canvas");
            ui.label("• Arrow keys = Nudge (Shift: 10px)");
            ui.label("• Cmd/Ctrl + D = Duplicate");
            ui.label("• Cmd/Ctrl + G = Group (Shift: Ungroup)");
            ui.label("• [ or ] = Layer Backward / Forward");
            ui.label("• Cmd/Ctrl + Z = Undo / Redo");
            return;
        }

        // 1. Alignment & Distribution Toolbar
        ui.horizontal(|ui| {
            if ui.button("⇥").on_hover_text("Align Left").clicked() {
                doc.align_selection(&state.selected_ids, AlignmentMode::Left);
            }
            if ui.button("↔").on_hover_text("Align Horizontal Center").clicked() {
                doc.align_selection(&state.selected_ids, AlignmentMode::CenterHorizontal);
            }
            if ui.button("⇤").on_hover_text("Align Right").clicked() {
                doc.align_selection(&state.selected_ids, AlignmentMode::Right);
            }
            ui.separator();
            if ui.button("⤒").on_hover_text("Align Top").clicked() {
                doc.align_selection(&state.selected_ids, AlignmentMode::Top);
            }
            if ui.button("↕").on_hover_text("Align Vertical Center").clicked() {
                doc.align_selection(&state.selected_ids, AlignmentMode::CenterVertical);
            }
            if ui.button("⤓").on_hover_text("Align Bottom").clicked() {
                doc.align_selection(&state.selected_ids, AlignmentMode::Bottom);
            }
            ui.separator();
            if ui.button("⫼").on_hover_text("Distribute Horizontal Spacing").clicked() {
                doc.distribute_selection(&state.selected_ids, true);
            }
            if ui.button("⫵").on_hover_text("Distribute Vertical Spacing").clicked() {
                doc.distribute_selection(&state.selected_ids, false);
            }
        });

        // Layer Arrange & Grouping Toolbar
        ui.horizontal(|ui| {
            if ui.small_button("▲").on_hover_text("Bring Forward (])").clicked() {
                doc.bring_forward(&state.selected_ids);
            }
            if ui.small_button("▼").on_hover_text("Send Backward ([)").clicked() {
                doc.send_backward(&state.selected_ids);
            }
            if ui.small_button("⤒").on_hover_text("Bring to Front (Shift+])").clicked() {
                doc.bring_to_front(&state.selected_ids);
            }
            if ui.small_button("⤓").on_hover_text("Send to Back (Shift+[)").clicked() {
                doc.send_to_back(&state.selected_ids);
            }
            ui.separator();
            if state.selected_ids.len() > 1 {
                if ui.small_button("⊞ Group").on_hover_text("Group Selection (Cmd+G)").clicked() {
                    if let Some(gid) = doc.group_selection(&state.selected_ids) {
                        state.selected_ids = vec![gid];
                    }
                }
            } else if state.selected_ids.len() == 1 {
                if ui.small_button("⊟ Ungroup").on_hover_text("Ungroup (Cmd+Shift+G)").clicked() {
                    let unpacked = doc.ungroup_selection(&state.selected_ids);
                    if !unpacked.is_empty() {
                        state.selected_ids = unpacked;
                    }
                }
            }
        });

        ui.add_space(8.0);
        ui.separator();

        // 2. Transform & Dimensions
        let first_id = state.selected_ids[0];
        if let Some(node) = doc.find_node_mut(first_id) {
            ui.label(RichText::new("TRANSFORM").strong().size(11.0).color(Color32::from_rgb(140, 140, 150)));
            ui.add_space(4.0);

            // Layer Name
            ui.horizontal(|ui| {
                ui.label("Name");
                ui.text_edit_singleline(&mut node.name);
            });

            // Position (X, Y)
            ui.horizontal(|ui| {
                ui.label("X");
                ui.add(DragValue::new(&mut node.rect.x).speed(1.0));
                ui.add_space(8.0);
                ui.label("Y");
                ui.add(DragValue::new(&mut node.rect.y).speed(1.0));
            });

            // Size (W, H)
            ui.horizontal(|ui| {
                ui.label("W");
                ui.add(DragValue::new(&mut node.rect.width).speed(1.0).range(1.0..=10000.0));
                ui.add_space(8.0);
                ui.label("H");
                ui.add(DragValue::new(&mut node.rect.height).speed(1.0).range(1.0..=10000.0));
            });

            // Rotation
            ui.horizontal(|ui| {
                ui.label("Angle");
                let mut deg = node.rotation.to_degrees();
                if ui.add(DragValue::new(&mut deg).speed(1.0).range(-360.0..=360.0).suffix("°")).changed() {
                    node.rotation = deg.to_radians();
                }
            });

            // Corner Radius (if rect or frame)
            if node.kind == NodeKind::Rectangle || node.kind == NodeKind::Frame {
                ui.horizontal(|ui| {
                    ui.label("Radius");
                    let mut r = node.corner_radius.top_left;
                    if ui.add(DragValue::new(&mut r).speed(1.0).range(0.0..=500.0)).changed() {
                        node.corner_radius = sigma_core::CornerRadius::uniform(r);
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Corners");
                    ui.add(DragValue::new(&mut node.corner_radius.top_left).speed(1.0).range(0.0..=500.0).prefix("TL: "));
                    ui.add(DragValue::new(&mut node.corner_radius.top_right).speed(1.0).range(0.0..=500.0).prefix("TR: "));
                });
                ui.horizontal(|ui| {
                    ui.label("       ");
                    ui.add(DragValue::new(&mut node.corner_radius.bottom_left).speed(1.0).range(0.0..=500.0).prefix("BL: "));
                    ui.add(DragValue::new(&mut node.corner_radius.bottom_right).speed(1.0).range(0.0..=500.0).prefix("BR: "));
                });
            }

            // Opacity
            ui.horizontal(|ui| {
                ui.label("Opacity");
                ui.add(Slider::new(&mut node.opacity, 0.0..=1.0).show_value(true));
            });

            // 3. TYPOGRAPHY SECTION (Fonts, Sizes, Weights, Align, Casing)
            if let Some(text_props) = &mut node.text_props {
                ui.add_space(8.0);
                ui.separator();
                ui.label(RichText::new("TYPOGRAPHY").strong().size(11.0).color(Color32::from_rgb(140, 140, 150)));
                ui.add_space(4.0);

                // Text Content Editor
                ui.label("Content");
                ui.text_edit_multiline(&mut text_props.content);

                // Font Family Dropdown
                ui.horizontal(|ui| {
                    ui.label("Font");
                    egui::ComboBox::from_id_salt("font_family_select")
                        .selected_text(&text_props.font_family)
                        .show_ui(ui, |ui| {
                            let families = [
                                "Inter",
                                "Roboto",
                                "Helvetica",
                                "Arial",
                                "SF Pro",
                                "Georgia",
                                "Courier New",
                                "Monospace",
                            ];
                            for family in families {
                                ui.selectable_value(&mut text_props.font_family, family.to_string(), family);
                            }
                        });
                });

                // Font Weight & Size
                ui.horizontal(|ui| {
                    ui.label("Weight");
                    egui::ComboBox::from_id_salt("font_weight_select")
                        .selected_text(match text_props.font_weight {
                            300 => "Light (300)",
                            400 => "Regular (400)",
                            500 => "Medium (500)",
                            600 => "SemiBold (600)",
                            700 => "Bold (700)",
                            900 => "Black (900)",
                            _ => "Regular (400)",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut text_props.font_weight, 300, "Light (300)");
                            ui.selectable_value(&mut text_props.font_weight, 400, "Regular (400)");
                            ui.selectable_value(&mut text_props.font_weight, 500, "Medium (500)");
                            ui.selectable_value(&mut text_props.font_weight, 600, "SemiBold (600)");
                            ui.selectable_value(&mut text_props.font_weight, 700, "Bold (700)");
                            ui.selectable_value(&mut text_props.font_weight, 900, "Black (900)");
                        });
                });

                // Font Size DragValue + Quick Presets
                ui.horizontal(|ui| {
                    ui.label("Size");
                    ui.add(DragValue::new(&mut text_props.font_size).speed(1.0).range(6.0..=144.0));
                });
                ui.horizontal_wrapped(|ui| {
                    for preset in [12.0, 14.0, 16.0, 20.0, 24.0, 32.0, 48.0] {
                        let is_active = text_props.font_size == preset;
                        if ui.selectable_label(is_active, format!("{:.0}", preset)).clicked() {
                            text_props.font_size = preset;
                            text_props.line_height = preset * 1.3;
                        }
                    }
                });

                // Line Height & Letter Spacing
                ui.horizontal(|ui| {
                    ui.label("Line H");
                    ui.add(DragValue::new(&mut text_props.line_height).speed(1.0).range(8.0..=200.0));
                    ui.add_space(8.0);
                    ui.label("Spacing");
                    ui.add(DragValue::new(&mut text_props.letter_spacing).speed(0.1).range(-5.0..=50.0));
                });

                // Text Alignment (Left, Center, Right)
                ui.horizontal(|ui| {
                    ui.label("Align");
                    let is_l = text_props.align == TextAlign::Left;
                    if ui.selectable_label(is_l, "Left").clicked() {
                        text_props.align = TextAlign::Left;
                    }
                    let is_c = text_props.align == TextAlign::Center;
                    if ui.selectable_label(is_c, "Center").clicked() {
                        text_props.align = TextAlign::Center;
                    }
                    let is_r = text_props.align == TextAlign::Right;
                    if ui.selectable_label(is_r, "Right").clicked() {
                        text_props.align = TextAlign::Right;
                    }
                });

                // Text Casing (Normal, UPPER, lower, Title)
                ui.horizontal(|ui| {
                    ui.label("Case");
                    let is_norm = text_props.text_case == TextCase::Normal;
                    if ui.selectable_label(is_norm, "Aa").on_hover_text("Normal case").clicked() {
                        text_props.text_case = TextCase::Normal;
                    }
                    let is_upper = text_props.text_case == TextCase::Uppercase;
                    if ui.selectable_label(is_upper, "AA").on_hover_text("UPPERCASE").clicked() {
                        text_props.text_case = TextCase::Uppercase;
                    }
                    let is_lower = text_props.text_case == TextCase::Lowercase;
                    if ui.selectable_label(is_lower, "aa").on_hover_text("lowercase").clicked() {
                        text_props.text_case = TextCase::Lowercase;
                    }
                });

                // Text Decoration (None, Underline, Strikethrough)
                ui.horizontal(|ui| {
                    ui.label("Decor");
                    let is_none = text_props.decoration == TextDecoration::None;
                    if ui.selectable_label(is_none, "—").on_hover_text("None").clicked() {
                        text_props.decoration = TextDecoration::None;
                    }
                    let is_under = text_props.decoration == TextDecoration::Underline;
                    if ui.selectable_label(is_under, "U̲").on_hover_text("Underline").clicked() {
                        text_props.decoration = TextDecoration::Underline;
                    }
                    let is_strike = text_props.decoration == TextDecoration::Strikethrough;
                    if ui.selectable_label(is_strike, "S̶").on_hover_text("Strikethrough").clicked() {
                        text_props.decoration = TextDecoration::Strikethrough;
                    }
                });
            }

            // 4. Auto Layout (Figma Pro feature)
            if node.kind == NodeKind::Frame {
                ui.add_space(8.0);
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(RichText::new("AUTO LAYOUT").strong().size(11.0).color(Color32::from_rgb(140, 140, 150)));
                    ui.checkbox(&mut node.auto_layout.enabled, "");
                });

                if node.auto_layout.enabled {
                    ui.horizontal(|ui| {
                        ui.label("Direction");
                        let is_h = node.auto_layout.direction == sigma_core::LayoutDirection::Horizontal;
                        if ui.selectable_label(is_h, "→ Horizontal").clicked() {
                            node.auto_layout.direction = sigma_core::LayoutDirection::Horizontal;
                            node.apply_auto_layout();
                        }
                        let is_v = node.auto_layout.direction == sigma_core::LayoutDirection::Vertical;
                        if ui.selectable_label(is_v, "↓ Vertical").clicked() {
                            node.auto_layout.direction = sigma_core::LayoutDirection::Vertical;
                            node.apply_auto_layout();
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Gap");
                        if ui.add(DragValue::new(&mut node.auto_layout.gap).speed(1.0).range(0.0..=200.0)).changed() {
                            node.apply_auto_layout();
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Padding");
                        let mut p = node.auto_layout.padding.top;
                        if ui.add(DragValue::new(&mut p).speed(1.0).range(0.0..=200.0)).changed() {
                            node.auto_layout.padding = sigma_core::Padding::all(p);
                            node.apply_auto_layout();
                        }
                    });
                }
            }

            // 5. FILL SECTION & COLOR SWATCHES
            ui.add_space(8.0);
            ui.separator();
            ui.label(RichText::new("FILL").strong().size(11.0).color(Color32::from_rgb(140, 140, 150)));
            ui.add_space(4.0);

            if let Some(fill) = node.fills.first_mut() {
                match fill {
                    sigma_core::Fill::Solid { color, opacity } => {
                        ui.horizontal(|ui| {
                            let mut c = [
                                color.r as f32 / 255.0,
                                color.g as f32 / 255.0,
                                color.b as f32 / 255.0,
                            ];
                            if ui.color_edit_button_rgb(&mut c).changed() {
                                *color = Color::rgb(
                                    (c[0] * 255.0) as u8,
                                    (c[1] * 255.0) as u8,
                                    (c[2] * 255.0) as u8,
                                );
                            }
                            let mut hex = color.to_hex();
                            if ui.add(egui::TextEdit::singleline(&mut hex).desired_width(70.0)).lost_focus() {
                                if let Some(parsed) = Color::from_hex(&hex) {
                                    *color = parsed;
                                }
                            }
                            ui.add(DragValue::new(opacity).speed(0.01).range(0.0..=1.0));
                        });

                        // Figma Color Swatches Presets
                        ui.add_space(4.0);
                        ui.horizontal_wrapped(|ui| {
                            let swatches = [
                                "#000000", "#FFFFFF", "#6B7280", "#0A84FF", "#10B981",
                                "#F59E0B", "#EF4444", "#8B5CF6", "#EC4899", "#14B8A6",
                            ];
                            for hex in swatches {
                                let col = Color::from_hex(hex).unwrap_or(Color::BLACK);
                                let c32 = Color32::from_rgb(col.r, col.g, col.b);
                                let (rect, resp) = ui.allocate_exact_size(EguiVec2::splat(16.0), egui::Sense::click());
                                let p = ui.painter_at(rect);
                                p.rect_filled(rect, 3.0, c32);
                                p.rect_stroke(rect, 3.0, EguiStroke::new(1.0, Color32::from_rgb(60, 60, 60)), egui::StrokeKind::Outside);
                                if resp.on_hover_text(hex).clicked() {
                                    *color = col;
                                }
                            }
                        });
                    }
                }
            } else {
                if ui.button("+ Add Fill").clicked() {
                    node.fills.push(sigma_core::Fill::Solid {
                        color: Color::WHITE,
                        opacity: 1.0,
                    });
                }
            }

            // 6. STROKE SECTION
            ui.add_space(8.0);
            ui.separator();
            ui.label(RichText::new("STROKE").strong().size(11.0).color(Color32::from_rgb(140, 140, 150)));
            ui.add_space(4.0);

            if let Some(stroke) = node.strokes.first_mut() {
                ui.horizontal(|ui| {
                    let mut c = [
                        stroke.color.r as f32 / 255.0,
                        stroke.color.g as f32 / 255.0,
                        stroke.color.b as f32 / 255.0,
                    ];
                    if ui.color_edit_button_rgb(&mut c).changed() {
                        stroke.color = Color::rgb(
                            (c[0] * 255.0) as u8,
                            (c[1] * 255.0) as u8,
                            (c[2] * 255.0) as u8,
                        );
                    }
                    ui.label(stroke.color.to_hex());
                    ui.add(DragValue::new(&mut stroke.width).speed(0.5).range(0.5..=50.0).prefix("W: "));
                });

                ui.horizontal(|ui| {
                    ui.label("Style");
                    let is_solid = stroke.style == StrokeStyle::Solid;
                    if ui.selectable_label(is_solid, "— Solid").clicked() {
                        stroke.style = StrokeStyle::Solid;
                    }
                    let is_dashed = stroke.style == StrokeStyle::Dashed;
                    if ui.selectable_label(is_dashed, "┄ Dashed").clicked() {
                        stroke.style = StrokeStyle::Dashed;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Align");
                    let is_in = stroke.align == StrokeAlign::Inside;
                    if ui.selectable_label(is_in, "Inside").clicked() {
                        stroke.align = StrokeAlign::Inside;
                    }
                    let is_ctr = stroke.align == StrokeAlign::Center;
                    if ui.selectable_label(is_ctr, "Center").clicked() {
                        stroke.align = StrokeAlign::Center;
                    }
                    let is_out = stroke.align == StrokeAlign::Outside;
                    if ui.selectable_label(is_out, "Outside").clicked() {
                        stroke.align = StrokeAlign::Outside;
                    }
                });
            } else {
                if ui.button("+ Add Stroke").clicked() {
                    node.strokes.push(Stroke {
                        color: Color::BLACK,
                        width: 1.0,
                        align: StrokeAlign::Inside,
                        opacity: 1.0,
                        style: StrokeStyle::Solid,
                    });
                }
            }

            // 7. EFFECTS (Drop Shadow)
            ui.add_space(8.0);
            ui.separator();
            ui.label(RichText::new("EFFECTS").strong().size(11.0).color(Color32::from_rgb(140, 140, 150)));
            ui.add_space(4.0);

            if let Some(effect) = node.effects.first_mut() {
                match effect {
                    Effect::DropShadow { offset, blur, spread: _, color } => {
                        ui.horizontal(|ui| {
                            ui.label("Drop Shadow");
                            let mut c = [
                                color.r as f32 / 255.0,
                                color.g as f32 / 255.0,
                                color.b as f32 / 255.0,
                            ];
                            if ui.color_edit_button_rgb(&mut c).changed() {
                                *color = Color::rgb(
                                    (c[0] * 255.0) as u8,
                                    (c[1] * 255.0) as u8,
                                    (c[2] * 255.0) as u8,
                                );
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("X");
                            ui.add(DragValue::new(&mut offset.x).speed(1.0).range(-100.0..=100.0));
                            ui.add_space(6.0);
                            ui.label("Y");
                            ui.add(DragValue::new(&mut offset.y).speed(1.0).range(-100.0..=100.0));
                            ui.add_space(6.0);
                            ui.label("Blur");
                            ui.add(DragValue::new(blur).speed(1.0).range(0.0..=100.0));
                        });
                    }
                    _ => {}
                }
            } else {
                if ui.button("+ Drop Shadow").clicked() {
                    node.effects.push(Effect::DropShadow {
                        offset: sigma_core::Vec2::new(0.0, 4.0),
                        blur: 12.0,
                        spread: 0.0,
                        color: Color::rgba(0, 0, 0, 80),
                    });
                }
            }

            // 8. DEV MODE (CSS & Tailwind)
            ui.add_space(12.0);
            ui.separator();
            ui.label(RichText::new("DEV MODE (CSS & CODE)").strong().size(11.0).color(Color32::from_rgb(10, 132, 255)));
            ui.add_space(4.0);

            let css_code = export_to_css(node);
            let mut css_display = css_code.clone();
            ui.text_edit_multiline(&mut css_display);
            ui.horizontal(|ui| {
                if ui.button("📋 Copy CSS").clicked() {
                    ui.ctx().copy_text(css_code);
                }
                if ui.button("⚡ Export SVG").clicked() {
                    let svg_code = export_node_to_svg(node);
                    ui.ctx().copy_text(svg_code);
                }
                if ui.button("📄 Copy JSON").clicked() {
                    if let Ok(json) = serde_json::to_string_pretty(node) {
                        ui.ctx().copy_text(json);
                    }
                }
            });
        }
    });
}
