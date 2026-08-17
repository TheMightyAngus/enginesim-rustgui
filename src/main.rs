//! ENGINE·SIM entry — native desktop and WASM.

mod app;
mod graphs;
mod render;
mod sim;
mod theme;

use app::EngineApp;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_title("ENGINE·SIM"),
        ..Default::default()
    };
    eframe::run_native(
        "ENGINE·SIM",
        options,
        Box::new(|cc| {
            apply_style(&cc.egui_ctx);
            Ok(Box::new(EngineApp::default()))
        }),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    console_error_panic_hook::set_once();
    let web_options = eframe::WebOptions::default();
    wasm_bindgen_futures::spawn_local(async {
        let runner = eframe::WebRunner::new();
        runner
            .start(
                "the_canvas_id",
                web_options,
                Box::new(|cc| {
                    apply_style(&cc.egui_ctx);
                    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                        if let Some(el) = doc.get_element_by_id("loading_text") {
                            el.remove();
                        }
                    }
                    Ok(Box::new(EngineApp::default()))
                }),
            )
            .await
            .expect("failed to start eframe");
    });
}

fn apply_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.visuals.dark_mode = true;
    style.visuals.panel_fill = theme::PANEL;
    style.visuals.window_fill = theme::PANEL;
    style.visuals.override_text_color = Some(theme::TEXT);
    style.visuals.widgets.inactive.bg_fill = theme::BLOCK;
    style.visuals.widgets.hovered.bg_fill = theme::STEEL;
    style.visuals.widgets.active.bg_fill = theme::ACCENT_DIM;
    style.visuals.selection.bg_fill = theme::ACCENT_DIM;
    ctx.set_style(style);
}
