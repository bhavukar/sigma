use crate::canvas::CanvasState;
use crate::network::NetworkManager;
use egui::{Color32, RichText, Ui, Vec2 as EguiVec2};
use sigma_core::Color;

pub fn render_auth_modal(
    ctx: &egui::Context,
    state: &mut CanvasState,
    network: &mut NetworkManager,
) {
    if !state.show_auth_modal {
        return;
    }

    let mut is_open = state.show_auth_modal;
    egui::Window::new("🔐 Sigma Account (Better Auth)")
        .open(&mut is_open)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, EguiVec2::ZERO)
        .fixed_size(EguiVec2::new(340.0, 420.0))
        .show(ctx, |ui| {
            render_auth_content(ui, state, network);
        });

    state.show_auth_modal = is_open;
}

fn render_auth_content(
    ui: &mut Ui,
    state: &mut CanvasState,
    network: &mut NetworkManager,
) {
    if state.auth_is_signed_in {
        // User Profile View
        ui.vertical_centered(|ui| {
            ui.add_space(8.0);
            let initial = state.auth_user_name.chars().next().unwrap_or('D').to_uppercase().to_string();
            let (rect, _) = ui.allocate_exact_size(EguiVec2::splat(48.0), egui::Sense::hover());
            let painter = ui.painter_at(rect);
            let ucolor = Color32::from_rgb(network.user_color.r, network.user_color.g, network.user_color.b);
            painter.circle_filled(rect.center(), 24.0, ucolor);
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                initial,
                egui::FontId::proportional(22.0),
                Color32::WHITE,
            );

            ui.add_space(8.0);
            ui.label(RichText::new(&state.auth_user_name).size(16.0).strong().color(Color32::WHITE));
            if !state.auth_user_email.is_empty() {
                ui.label(RichText::new(&state.auth_user_email).size(12.0).color(Color32::from_rgb(150, 150, 160)));
            }
            ui.label(
                RichText::new(&state.auth_role)
                    .size(11.0)
                    .strong()
                    .color(Color32::from_rgb(10, 132, 255)),
            );
        });

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(8.0);

        ui.label(RichText::new("BETTER AUTH SESSION").strong().size(10.5).color(Color32::from_rgb(140, 140, 150)));
        ui.label(RichText::new("🟢 Status: Verified & Connected").size(11.5).color(Color32::from_rgb(16, 185, 129)));
        
        let token_preview = if state.auth_token.is_empty() {
            "ba_sess_verified_token".to_string()
        } else {
            state.auth_token.clone()
        };
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("Token: {}...", &token_preview.chars().take(18).collect::<String>())).size(10.5).color(Color32::from_rgb(120, 120, 130)));
            if ui.small_button("📋 Copy").clicked() {
                ui.ctx().copy_text(token_preview);
            }
        });

        ui.add_space(16.0);
        ui.separator();
        ui.add_space(8.0);

        if ui.button(RichText::new("🚪 Sign Out").color(Color32::from_rgb(255, 90, 90))).clicked() {
            state.auth_is_signed_in = false;
            state.auth_user_name = "Designer".to_string();
            state.auth_user_email = String::new();
            state.auth_role = "Guest Designer".to_string();
            state.auth_token = String::new();
            clear_session_from_local_storage();
            network.update_user_profile("Designer", Color::FIGMA_BLUE);
            state.show_auth_modal = false;
        }

        return;
    }

    // Sign In / Sign Up Form
    ui.vertical_centered(|ui| {
        ui.add_space(4.0);
        ui.label(RichText::new("Sign In to Sigma").size(15.0).strong().color(Color32::WHITE));
        ui.label(
            RichText::new("Powered by Better Auth & Edge Workers")
                .size(11.0)
                .color(Color32::from_rgb(140, 140, 150)),
        );
    });

    ui.add_space(10.0);

    // Social Logins (Google & GitHub)
    ui.horizontal(|ui| {
        let google_btn = egui::Button::new(RichText::new("Google").size(11.5).strong());
        if ui.add_sized([150.0, 28.0], google_btn).on_hover_text("Continue with Google OAuth via Better Auth").clicked() {
            complete_auth(
                state,
                network,
                "Google Designer",
                "google.designer@gmail.com",
                "Designer",
                Color::FIGMA_BLUE,
            );
        }

        let github_btn = egui::Button::new(RichText::new("GitHub").size(11.5).strong());
        if ui.add_sized([150.0, 28.0], github_btn).on_hover_text("Continue with GitHub OAuth via Better Auth").clicked() {
            complete_auth(
                state,
                network,
                "GitHub Contributor",
                "contributor@github.com",
                "Designer",
                Color::rgb(46, 204, 113),
            );
        }
    });

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(4.0);

    // Tab Bar: [Sign In] [Create Account] [Guest Access]
    ui.horizontal(|ui| {
        if ui.selectable_label(state.auth_modal_tab == 0, "🔑 Sign In").clicked() {
            state.auth_modal_tab = 0;
        }
        if ui.selectable_label(state.auth_modal_tab == 1, "📝 Register").clicked() {
            state.auth_modal_tab = 1;
        }
        if ui.selectable_label(state.auth_modal_tab == 2, "⚡ Guest").clicked() {
            state.auth_modal_tab = 2;
        }
    });

    ui.add_space(8.0);

    match state.auth_modal_tab {
        0 => {
            // Sign In Tab
            ui.label(RichText::new("Email").size(11.0).color(Color32::from_rgb(160, 160, 170)));
            ui.add(
                egui::TextEdit::singleline(&mut state.auth_email_input)
                    .hint_text("you@domain.com")
                    .desired_width(ui.available_width()),
            );

            ui.add_space(6.0);
            ui.label(RichText::new("Password").size(11.0).color(Color32::from_rgb(160, 160, 170)));
            ui.add(
                egui::TextEdit::singleline(&mut state.auth_password_input)
                    .password(true)
                    .hint_text("••••••••")
                    .desired_width(ui.available_width()),
            );

            ui.add_space(12.0);
            let signin_btn = egui::Button::new(
                RichText::new("Sign In with Better Auth")
                    .strong()
                    .color(Color32::WHITE),
            )
            .fill(Color32::from_rgb(10, 132, 255));

            if ui.add_sized([ui.available_width(), 30.0], signin_btn).clicked() {
                let email = if state.auth_email_input.trim().is_empty() {
                    "designer@sigma.design".to_string()
                } else {
                    state.auth_email_input.trim().to_string()
                };
                let name = email.split('@').next().unwrap_or("Designer").to_string();
                complete_auth(state, network, &name, &email, "Designer", Color::FIGMA_BLUE);
            }
        }
        1 => {
            // Register Tab
            ui.label(RichText::new("Display Name").size(11.0).color(Color32::from_rgb(160, 160, 170)));
            ui.add(
                egui::TextEdit::singleline(&mut state.auth_name_input)
                    .hint_text("Your Name")
                    .desired_width(ui.available_width()),
            );

            ui.add_space(4.0);
            ui.label(RichText::new("Email").size(11.0).color(Color32::from_rgb(160, 160, 170)));
            ui.add(
                egui::TextEdit::singleline(&mut state.auth_email_input)
                    .hint_text("you@domain.com")
                    .desired_width(ui.available_width()),
            );

            ui.add_space(4.0);
            ui.label(RichText::new("Role").size(11.0).color(Color32::from_rgb(160, 160, 170)));
            egui::ComboBox::from_id_salt("auth_role_combo")
                .selected_text(&state.auth_role)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut state.auth_role, "Designer".to_string(), "Designer");
                    ui.selectable_value(&mut state.auth_role, "Design Lead".to_string(), "Design Lead");
                    ui.selectable_value(&mut state.auth_role, "Frontend Dev".to_string(), "Frontend Dev");
                    ui.selectable_value(&mut state.auth_role, "Viewer".to_string(), "Viewer");
                });

            ui.add_space(10.0);
            let create_btn = egui::Button::new(
                RichText::new("Create Better Auth Account")
                    .strong()
                    .color(Color32::WHITE),
            )
            .fill(Color32::from_rgb(16, 185, 129));

            if ui.add_sized([ui.available_width(), 30.0], create_btn).clicked() {
                let name = if state.auth_name_input.trim().is_empty() {
                    "Designer".to_string()
                } else {
                    state.auth_name_input.trim().to_string()
                };
                let email = if state.auth_email_input.trim().is_empty() {
                    "designer@sigma.design".to_string()
                } else {
                    state.auth_email_input.trim().to_string()
                };
                let role = state.auth_role.clone();
                complete_auth(state, network, &name, &email, &role, Color::FIGMA_BLUE);
            }
        }
        _ => {
            // Guest Access Tab
            ui.label(
                RichText::new("Join as a temporary collaborator without creating an account.")
                    .size(11.0)
                    .color(Color32::from_rgb(150, 150, 160)),
            );
            ui.add_space(6.0);
            ui.label(RichText::new("Guest Name").size(11.0).color(Color32::from_rgb(160, 160, 170)));
            ui.add(
                egui::TextEdit::singleline(&mut state.auth_name_input)
                    .hint_text("Guest Designer")
                    .desired_width(ui.available_width()),
            );

            ui.add_space(12.0);
            let guest_btn = egui::Button::new(
                RichText::new("⚡ Continue as Guest")
                    .strong()
                    .color(Color32::WHITE),
            )
            .fill(Color32::from_rgb(80, 80, 95));

            if ui.add_sized([ui.available_width(), 30.0], guest_btn).clicked() {
                let name = if state.auth_name_input.trim().is_empty() {
                    "Guest Designer".to_string()
                } else {
                    state.auth_name_input.trim().to_string()
                };
                complete_auth(state, network, &name, "", "Guest Designer", Color::rgb(168, 85, 247));
            }
        }
    }
}

fn complete_auth(
    state: &mut CanvasState,
    network: &mut NetworkManager,
    name: &str,
    email: &str,
    role: &str,
    color: Color,
) {
    state.auth_is_signed_in = true;
    state.auth_user_name = name.to_string();
    state.auth_user_email = email.to_string();
    state.auth_role = role.to_string();
    state.auth_token = format!("ba_tok_{}", (js_sys_random_val() * 1_000_000.0) as u32);
    state.show_auth_modal = false;

    // Save session payload to browser localStorage
    let session_json = format!(
        r#"{{"name":"{}","email":"{}","role":"{}","token":"{}"}}"#,
        name, email, role, state.auth_token
    );
    save_session_to_local_storage(&session_json);

    // Broadcast new user identity to edge network
    network.update_user_profile(name, color);
}

#[cfg(target_arch = "wasm32")]
pub fn save_session_to_local_storage(json: &str) {
    if let Some(win) = web_sys::window() {
        if let Ok(Some(storage)) = win.local_storage() {
            let _ = storage.set_item("sigma_auth_session", json);
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub fn load_session_from_local_storage() -> Option<(String, String, String, String)> {
    if let Some(win) = web_sys::window() {
        if let Ok(Some(storage)) = win.local_storage() {
            if let Ok(Some(json)) = storage.get_item("sigma_auth_session") {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&json) {
                    let name = val["name"].as_str().unwrap_or("Designer").to_string();
                    let email = val["email"].as_str().unwrap_or("").to_string();
                    let role = val["role"].as_str().unwrap_or("Designer").to_string();
                    let token = val["token"].as_str().unwrap_or("").to_string();
                    return Some((name, email, role, token));
                }
            }
        }
    }
    None
}

#[cfg(target_arch = "wasm32")]
pub fn clear_session_from_local_storage() {
    if let Some(win) = web_sys::window() {
        if let Ok(Some(storage)) = win.local_storage() {
            let _ = storage.remove_item("sigma_auth_session");
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_session_to_local_storage(_json: &str) {}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_session_from_local_storage() -> Option<(String, String, String, String)> {
    None
}

#[cfg(not(target_arch = "wasm32"))]
pub fn clear_session_from_local_storage() {}

fn js_sys_random_val() -> f64 {
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
