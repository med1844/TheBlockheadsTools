mod app;
mod dw_impl;
mod fps_counter;
mod gpu;
mod image_type;
mod renderer;

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "My egui App",
        native_options,
        Box::new(|cc| Ok(Box::new(app::EditorApp::new(cc)))),
    )
    .unwrap();
}
