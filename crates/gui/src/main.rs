use the_blockheads_tools_gui::app::EditorApp;

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "My egui App",
        native_options,
        Box::new(|cc| Ok(Box::new(EditorApp::new(cc)))),
    )
    .unwrap();
}
