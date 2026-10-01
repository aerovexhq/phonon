#![deny(unsafe_code)]

//! Phonon GUI CAD interface, visual dynamics awareness widgets, custom window architecture, and interactive simulation studio.

pub mod app;
pub mod oscilloscope;
pub mod schematic;
pub mod thermal;
pub mod widgets;

pub use app::{PhononApp, ToolMode};
pub use egui::Theme;
pub use schematic::binary_format::{
    component_category_to_discriminant, component_kind_from_discriminant,
    component_kind_to_discriminant, compute_adler32, deserialize_project, load_project_from_file,
    save_project_to_file, serialize_project, BinaryFormatError, DeserializedProject,
    CURRENT_VERSION, PHONON_MAGIC,
};
pub use schematic::bus::{BusSignal, BusTapOff, SchematicBus};
pub use schematic::categories::ComponentCategory;
pub use schematic::components::{ComponentKind, SchematicComponent};
pub use schematic::erc::{ErcCode, ErcDiagnostic, ErcEngine, ErcSeverity};
pub use schematic::history::{CanvasCommand, HistoryStack};
pub use schematic::netlist_sync::{NetlistSyncEngine, NetlistSyncError, SyncDelta};
pub use schematic::sheet::{MultiSheetManager, SchematicSheet};
pub use schematic::subcircuit::{
    flatten_hierarchical_netlist, flatten_hierarchical_netlist_with_instances, PinDirection,
    SubcircuitDefinition, SubcircuitInstance, SubcircuitPin,
};
pub use schematic::wire::{compute_junction_dots, SchematicWire, WireSegment};
pub use widgets::dynamics_status::DynamicsStatusBadge;
pub use widgets::icon::{self, render_phonon_icon};
pub use widgets::palette::ComponentPalette;
pub use widgets::top_frame::{
    self, render_top_frame, render_top_frame_with_app, TopFrameAction, TopFrameConfig,
};

use phonon_core::PhysicsDynamicsBackend;

/// Instant boot theme configuration providing fast-path styling without D-Bus / X11 desktop portal theme queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BootThemeConfig {
    pub follow_system_theme: bool,
    pub default_theme: egui::Theme,
}

/// Constant representing the default boot theme configuration.
pub const DEFAULT_BOOT_THEME_CONFIG: BootThemeConfig = BootThemeConfig {
    follow_system_theme: false,
    default_theme: egui::Theme::Dark,
};

/// Returns the default boot theme configuration.
pub fn default_boot_theme_config() -> BootThemeConfig {
    DEFAULT_BOOT_THEME_CONFIG
}

/// Returns whether the system theme is followed at boot (always false to avoid slow IPC theme probes).
pub fn follow_system_theme() -> bool {
    false
}

/// Returns the default theme applied during instant boot (Dark).
pub fn default_theme() -> egui::Theme {
    egui::Theme::Dark
}

/// Determines the preferred graphics renderer based on the `PHONON_RENDERER` environment variable,
/// defaulting to OpenGL/EGL (`eframe::Renderer::Glow`) for instant sub-200ms cold startup.
pub fn determine_boot_renderer() -> eframe::Renderer {
    match std::env::var("PHONON_RENDERER").as_deref() {
        Ok("wgpu") | Ok("WGPU") => eframe::Renderer::Wgpu,
        _ => eframe::Renderer::Glow,
    }
}

/// Generates optimized `eframe::NativeOptions` for sub-200ms cold startup.
pub fn default_native_options() -> eframe::NativeOptions {
    let renderer = determine_boot_renderer();
    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 850.0])
            .with_min_inner_size([800.0, 600.0])
            .with_decorations(false)
            .with_title("Phonon Studio - Electro-Thermal CAD & Circuit Simulator"),
        renderer,
        ..Default::default()
    }
}

/// Runs the native desktop CAD interface and visualization studio with default auto-selecting dynamics backend.
pub fn run_gui() -> Result<(), Box<dyn std::error::Error>> {
    let native_options = default_native_options();

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
    let native_options = default_native_options();

    eframe::run_native(
        "Phonon Studio",
        native_options,
        Box::new(move |cc| Ok(Box::new(PhononApp::with_backend(cc, backend)))),
    )
    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}
