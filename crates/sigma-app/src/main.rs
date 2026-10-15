#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1360.0, 840.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("Sigma — Fast Vector Design Engine"),
        ..Default::default()
    };

    eframe::run_native(
        "Sigma",
        native_options,
        Box::new(|_cc| Ok(Box::new(sigma_app::SigmaApp::default()))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use wasm_bindgen::JsCast;

    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window found")
            .document()
            .expect("No document found");

        let canvas = document
            .get_element_by_id("sigma_canvas")
            .expect("Failed to find #sigma_canvas element")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("#sigma_canvas is not an HtmlCanvasElement");

        if let Some(loading) = document.get_element_by_id("loading") {
            let _ = loading.set_attribute("style", "display: none;");
        }

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|_cc| Ok(Box::new(sigma_app::SigmaApp::default()))),
            )
            .await;

        if let Err(e) = start_result {
            log::error!("Failed to start Sigma WebRunner: {:?}", e);
        }
    });
}
