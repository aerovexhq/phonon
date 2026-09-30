//! Phonon GUI CAD interface and visualization library.

#![deny(unsafe_code)]

pub mod app;
pub mod oscilloscope;
pub mod schematic;
pub mod thermal;

pub use app::PhononApp;

/// Runs the native desktop CAD interface and visualization studio.
pub fn run_gui() -> Result<(), Box<dyn std::error::Error>> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 850.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("Phonon - Electro-Thermal CAD & Circuit Simulator"),
        ..Default::default()
    };

    eframe::run_native(
        "Phonon CAD",
        native_options,
        Box::new(|cc| Ok(Box::new(PhononApp::new(cc)))),
    )
    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}
