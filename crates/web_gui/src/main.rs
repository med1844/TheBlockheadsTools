use eframe::wasm_bindgen::JsCast;
use the_blockheads_tools_gui::app::EditorApp;
use web_sys::{HtmlCanvasElement, window};

async fn start_app(canvas: HtmlCanvasElement) {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    eframe::WebLogger::init(log::LevelFilter::Trace).ok();

    eframe::WebRunner::new()
        .start(
            canvas,
            eframe::WebOptions::default(),
            Box::new(|cc| Ok(Box::new(EditorApp::new(cc)))),
        )
        .await
        .unwrap()
}

fn main() {
    console_error_panic_hook::set_once();

    let document = window()
        .and_then(|win| win.document())
        .expect("Could not access the document");
    let canvas_node = document
        .get_element_by_id("egui_canvas")
        .expect("no canvas with id `egui_canvas`")
        .dyn_into::<HtmlCanvasElement>()
        .expect("element should be an HtmlCanvasElement");
    wasm_bindgen_futures::spawn_local(start_app(canvas_node));
}
