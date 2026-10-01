#![deny(unsafe_code)]

//! Phonon GUI CAD interface, visual dynamics awareness widgets, custom window architecture, and interactive simulation studio.

pub mod app;
pub mod oscilloscope;
pub mod schematic;
pub mod thermal;
pub mod widgets;

pub use app::{PhononApp, ToolMode};
pub use schematic::categories::ComponentCategory;
pub use schematic::history::{CanvasCommand, HistoryStack};
pub use widgets::dynamics_status::DynamicsStatusBadge;
pub use widgets::icon::{self, render_phonon_icon};
pub use widgets::palette::ComponentPalette;
pub use widgets::top_frame::{
    self, render_top_frame, render_top_frame_with_app, TopFrameAction, TopFrameConfig,
};

use phonon_core::PhysicsDynamicsBackend;

/// Runs the native desktop CAD interface and visualization studio with default auto-selecting dynamics backend.
pub fn run_gui() -> Result<(), Box<dyn std::error::Error>> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 850.0])
            .with_min_inner_size([800.0, 600.0])
            .with_decorations(false)
            .with_title("Phonon Studio - Electro-Thermal CAD & Circuit Simulator"),
        ..Default::default()
    };

    eframe::run_native(
        "Phonon Studio",
        native_options,
        Box::new(|cc| Ok(Box::new(PhononApp::new(cc)))),
    )
    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

/// Runs the native desktop CAD interface and visualization studio with a custom physics dynamics backend injected in-process.
pub fn run_gui_with_custom_backend(
    backend: Box<dyn PhysicsDynamicsBackend>,
) -> Result<(), Box<dyn std::error::Error>> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 850.0])
            .with_min_inner_size([800.0, 600.0])
            .with_decorations(false)
            .with_title("Phonon Studio - Electro-Thermal CAD & Circuit Simulator"),
        ..Default::default()
    };

    eframe::run_native(
        "Phonon Studio",
        native_options,
        Box::new(move |cc| Ok(Box::new(PhononApp::with_backend(cc, backend)))),
    )
    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}
