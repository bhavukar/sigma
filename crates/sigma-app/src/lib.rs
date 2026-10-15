#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod auth;
mod canvas;
mod context_menu;
mod inspector;
mod logo;
mod network;
mod shortcuts_modal;
mod sidebar;
mod toolbar;

pub use app::SigmaApp;
pub use network::NetworkManager;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).ok();

    use wasm_bindgen::JsCast;

    let document = web_sys::window()
        .expect("No window found")
        .document()
        .expect("No document found");

    let canvas = document
        .get_element_by_id("sigma_canvas")
        .expect("Failed to find #sigma_canvas element")
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .expect("#sigma_canvas is not an HtmlCanvasElement");

    // Hide loading screen once initialized
    if let Some(loading) = document.get_element_by_id("loading") {
        let _ = loading.set_attribute("style", "display: none;");
    }

    let web_options = eframe::WebOptions::default();
    eframe::WebRunner::new()
        .start(
            canvas,
            web_options,
            Box::new(|_cc| Ok(Box::new(SigmaApp::default()))),
        )
        .await
}
