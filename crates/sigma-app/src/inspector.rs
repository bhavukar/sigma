use crate::canvas::CanvasState;
use egui::{Color32, DragValue, RichText, ScrollArea, Slider, Ui};
use sigma_core::{export_to_css, AlignmentMode, Color, Document, NodeKind, Stroke, StrokeAlign};

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

            ui.add_space(16.0);
            ui.label(RichText::new("PRO TIPS").strong().size(11.0).color(Color32::from_rgb(140, 140, 150)));
            ui.separator();
            ui.label("• Space + Drag to Pan canvas");
            ui.label("• Mouse wheel to Zoom");
            ui.label("• R = Rectangle, F = Frame, O = Ellipse, T = Text");
            ui.label("• V = Select tool, H = Hand tool");
            return;
        }

        // 1. Alignment Toolbar
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
            if ui.button("⤒").on_hover_text("Align Top").clicked() {
                doc.align_selection(&state.selected_ids, AlignmentMode::Top);
            }
            if ui.button("↕").on_hover_text("Align Vertical Center").clicked() {
                doc.align_selection(&state.selected_ids, AlignmentMode::CenterVertical);
            }
            if ui.button("⤓").on_hover_text("Align Bottom").clicked() {
                doc.align_selection(&state.selected_ids, AlignmentMode::Bottom);
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

            // Corner Radius (if rect or frame)
            if node.kind == NodeKind::Rectangle || node.kind == NodeKind::Frame {
                ui.horizontal(|ui| {
                    ui.label("Radius");
                    let mut r = node.corner_radius.top_left;
                    if ui.add(DragValue::new(&mut r).speed(1.0).range(0.0..=500.0)).changed() {
                        node.corner_radius = sigma_core::CornerRadius::uniform(r);
                    }
                });
            }

            // Opacity
            ui.horizontal(|ui| {
                ui.label("Opacity");
                ui.add(Slider::new(&mut node.opacity, 0.0..=1.0).show_value(true));
            });

            // Text Properties (if text)
            if let Some(text_props) = &mut node.text_props {
                ui.add_space(8.0);
                ui.separator();
                ui.label(RichText::new("TYPOGRAPHY").strong().size(11.0).color(Color32::from_rgb(140, 140, 150)));
                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    ui.label("Text");
                    ui.text_edit_singleline(&mut text_props.content);
                });

                ui.horizontal(|ui| {
                    ui.label("Size");
                    ui.add(DragValue::new(&mut text_props.font_size).speed(1.0).range(6.0..=128.0));
                });
            }

            // Auto Layout (Figma Pro feature)
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

            // Fill Section
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
                            ui.label(color.to_hex());
                            ui.add(DragValue::new(opacity).speed(0.01).range(0.0..=1.0));
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

            // Stroke Section
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
                    ui.add(DragValue::new(&mut stroke.width).speed(0.5).range(0.5..=50.0));
                });
            } else {
                if ui.button("+ Add Stroke").clicked() {
                    node.strokes.push(Stroke {
                        color: Color::BLACK,
                        width: 1.0,
                        align: StrokeAlign::Inside,
                        opacity: 1.0,
                    });
                }
            }

            // Dev Mode / CSS Export
            ui.add_space(12.0);
            ui.separator();
            ui.label(RichText::new("DEV MODE (CSS)").strong().size(11.0).color(Color32::from_rgb(10, 132, 255)));
            ui.add_space(4.0);

            let css_code = export_to_css(node);
            let mut css_display = css_code.clone();
            ui.text_edit_multiline(&mut css_display);
            if ui.button("📋 Copy CSS").clicked() {
                ui.ctx().copy_text(css_code);
            }
        }
    });
}
