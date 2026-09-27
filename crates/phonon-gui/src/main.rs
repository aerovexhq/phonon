//! Phonon GUI executable.

#![deny(unsafe_code)]

use eframe::NativeOptions;
use phonon_gui::PhononApp;

fn main() -> eframe::Result<()> {
    let native_options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 850.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("Phonon — Electro-Thermal CAD & Circuit Simulator"),
        ..Default::default()
    };

    eframe::run_native(
        "Phonon CAD",
        native_options,
        Box::new(|cc| Ok(Box::new(PhononApp::new(cc)))),
    )
}
