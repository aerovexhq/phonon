#![deny(unsafe_code)]

//! Phonon GUI CAD interface, visual dynamics awareness widgets, custom window architecture, and interactive simulation studio.

pub mod actions;
pub mod app;
pub mod extraction;
pub mod oscilloscope;
pub mod preferences;
pub mod schematic;
pub mod scripting;
pub mod storage;
pub mod theme;
pub mod thermal;
pub mod widgets;

pub use scripting::{
    evaluate_expression, generate_trace, parse_expression, AssertionRecord, ExpressionGrapher,
    LuaEngine, LuaFunction, LuaTable, LuaValue, MathAst, MathOp, PermissionKind,
    PermissionManager, PermissionState, TableKey,
};

pub use actions::{ActionCategory, ActionDef, ActionId, ActionRegistry};
pub use app::{PhononApp, ToolMode};
pub use egui::Theme;
pub use preferences::AppPreferences;
pub use storage::{
    bytes_to_hex, hex_to_bytes, MemoryStorageAdapter, ProjectMetadata, ProjectStorageManager,
    StorageAdapter,
};
pub use theme::{
    color32_to_hex, hex_to_color32, PhononTheme, ThemePreset,
};
pub use widgets::preferences_dialog::{PreferencesDialog, PreferencesTab};
pub use widgets::command_palette::CommandPalette;
pub use widgets::confirmation_modal::{
    ConfirmationDecision, ConfirmationModal, DemoCircuitKind, PendingAction,
};
pub use widgets::floating_toolbar::{FloatingToolbarAction, FloatingToolbarState};
pub use widgets::project_dialog::{ProjectDialog, ProjectDialogAction, ProjectDialogMode};
pub use extraction::{DeviceModelKind, ExtractionWizardDialog};
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
pub use schematic::binary_history::{
    adler32, deserialize_history, read_command, serialize_history, write_command, ActionOpcode,
    BinaryHistoryError, HISTORY_MAGIC, HISTORY_VERSION,
};
pub use schematic::history::{CanvasCommand, HistoryStack};
pub use schematic::netlist_sync::{NetlistSyncEngine, NetlistSyncError, SyncDelta};
pub use schematic::sheet::{MultiSheetManager, SchematicSheet};
pub use schematic::subcircuit::{
    flatten_hierarchical_netlist, flatten_hierarchical_netlist_with_instances, PinDirection,
    SubcircuitDefinition, SubcircuitInstance, SubcircuitPin,
};
pub use schematic::wire::{compute_junction_dots, SchematicWire, WireSegment};
pub use widgets::cluster_dashboard_dialog::ClusterDashboardDialog;
pub use widgets::dynamics_status::DynamicsStatusBadge;
pub use widgets::exceptional_point_dialog::ExceptionalPointDialog;
pub use widgets::icon::{self, render_phonon_icon};
pub use widgets::palette::ComponentPalette;
pub use widgets::sensitivity_dialog::SensitivityDialog;
pub use widgets::monte_carlo_dialog::{MonteCarloParamEntry, MonteCarloYieldDialog};
pub use widgets::neuromorphic_snn_dialog::NeuromorphicSnnDialog;
pub use widgets::polariton_cavity_dialog::PolaritonCavityDialog;
pub use widgets::smith_chart_dialog::{HoveredMarker, SmithChartDialog};
pub use widgets::thermal_floorplan_dialog::ThermalFloorplanDialog;
pub use widgets::lua_console_dialog::{LuaConsoleDialog, ScriptPreset};
pub use widgets::top_frame::{
    self, render_top_frame, render_top_frame_with_app, TopFrameAction, TopFrameConfig,
};
pub use widgets::weyl_semimetal_dialog::{SemimetalMode, WeylSemimetalDialog};
pub use widgets::fqh_braiding_dialog::FqhBraidingDialog;
pub use widgets::jtwpa_dialog::JtwpaDialog;
pub use widgets::floquet_metasurface_dialog::FloquetMetasurfaceDialog;
pub use widgets::holonomic_processor_dialog::HolonomicProcessorDialog;
pub use widgets::twisted_moire_dialog::TwistedMoireDialog;
pub use widgets::chern_circulator_dialog::ChernCirculatorDialog;
pub use widgets::kerr_microcomb_dialog::KerrMicrocombDialog;
pub use widgets::exceptional_surface_dialog::ExceptionalSurfaceDialog;
pub use widgets::soti_corner_dialog::SotiCornerDialog;
pub use widgets::lieb_lattice_dialog::LiebLatticeDialog;
pub use widgets::axion_insulator_dialog::AxionInsulatorDialog;
pub use widgets::floquet_time_crystal_dialog::FloquetTimeCrystalDialog;
pub use widgets::quantum_braiding_lattice_dialog::QuantumBraidingLatticeDialog;
pub use widgets::optomechanical_squeezing_dialog::OptomechanicalSqueezingDialog;
pub use widgets::skyrmion_router_dialog::SkyrmionRouterDialog;
pub use widgets::acoustic_soliton_dialog::AcousticSolitonDialog;
pub use widgets::valley_multiplexer_dialog::{ValleyDialogTab, ValleyMultiplexerDialog};
pub use widgets::non_hermitian_skin_dialog::{NonHermitianSkinDialog, NonHermitianSkinDialogTab};
pub use widgets::quadrupole_shg_dialog::{QuadrupoleShgDialog, QuadrupoleShgDialogTab};
pub use widgets::synthetic_4d_dialog::{Synthetic4dDialog, Synthetic4dDialogTab};
pub use widgets::pt_symmetric_dialog::{PtSymmetricDialog, PtSymmetricDialogTab};
pub use widgets::acoustic_bic_dialog::{AcousticBicDialog, AcousticBicDialogTab};
pub use widgets::euler_acoustic_dialog::{EulerAcousticDialog, EulerAcousticDialogTab};
pub use widgets::octupole_insulator_dialog::{OctupoleDialogTab, OctupoleInsulatorDialog};
pub use widgets::aah_quasicrystal_dialog::{AahDialogTab, AahQuasicrystalDialog};
pub use widgets::valley_hall_vortex_dialog::{ValleyHallDialogTab, ValleyHallVortexDialog};
pub use widgets::skyrmion_deflector_dialog::{SkyrmionDeflectorDialog, SkyrmionDeflectorTab};
pub use widgets::floquet_frequency_dialog::{FloquetFrequencyDialog, FloquetFrequencyTab};
pub use widgets::corner_laser_dialog::{CornerLaserDialog, CornerLaserTab};
pub use widgets::metasurface_hologram_dialog::{MetasurfaceHologramDialog, MetasurfaceHologramTab};
pub use widgets::majorana_surface_code_dialog::{MajoranaSurfaceCodeDialog, MajoranaSurfaceCodeTab};
pub use widgets::optomechanical_transducer_dialog::{OptomechanicalTransducerDialog, OptomechanicalTransducerTab};
pub use widgets::corner_polariton_microcomb_dialog::{CornerPolaritonMicrocombDialog, CornerPolaritonMicrocombTab};
pub use widgets::directional_radiation_dialog::{DirectionalRadiationDialog, DirectionalRadiationTab};
pub use widgets::atmospheric_neutron_dialog::{AtmosphericNeutronDialog, AtmosphericNeutronTab};
pub use widgets::thermal_vacuum_dialog::{ThermalVacuumDialog, ThermalVacuumTab};
pub use widgets::space_avionics_bus_dialog::{SpaceAvionicsBusDialog, SpaceAvionicsBusTab};
pub use widgets::rhbd_self_healing_dialog::{RhbdSelfHealingDialog, RhbdTab};
pub use widgets::production_economics_dialog::{EconomicsTab, ProductionEconomicsDialog};
pub use widgets::chiplet_packaging_dialog::{ChipletPackagingDialog, PackagingTab};
pub use widgets::electrothermal_throttling_dialog::{ElectrothermalThrottlingDialog, ThrottlingTab};
pub use widgets::pdn_droop_dialog::{PdnDroopDialog, PdnTab};
pub use widgets::silicon_aging_dialog::{AgingTab, SiliconAgingDialog};
pub use widgets::wafer_yield_dialog::{WaferMapColorMode, WaferYieldDialog, WaferYieldTab};
pub use widgets::dse_optimization_dialog::{DseOptimizationDialog, DseTab};
pub use widgets::silicon_lifecycle_dialog::{SiliconLifecycleDialog, SlmTab};
pub use widgets::wasm_optimization_dialog::{WasmOptTab, WasmOptimizationDialog};
pub use widgets::pwa_offline_dialog::{PwaOfflineDialog, PwaTab};
pub use widgets::desktop_ipc_dialog::{
    DesktopIpcDialog, DesktopIpcTab, GerberLayer, IpcOpcode, IpcPacketError, IpcPacketHeader,
    PlatformAuditItem, decode_ipc_packet, encode_ipc_packet, IPC_MAGIC, IPC_VERSION,
};
pub use widgets::webrtc_mesh_dialog::{
    CrdtOpKind, CrdtOperation, DistributedSimChunk, LamportTimestamp, MeshAuditCriterion,
    MeshPeerNode, PeerConnectionState, PeerPresence, WebRtcMeshDialog, WebRtcMeshTab,
};
pub use widgets::webgpu_spice_dialog::{
    lttb_decimate, CsrMatrix, GpuBackendKind, WebGpuAuditCriterion, WebGpuSpiceDialog,
    WebGpuSpiceTab,
};
pub use widgets::wavepacket_scattering_dialog::{
    WavepacketAuditCriterion, WavepacketScatteringDialog, WavepacketScatteringTab,
};
pub use widgets::skyrmion_reservoir_dialog::{
    SkyrmionReservoirDialog, SkyrmionReservoirTab, SpintronicAuditCriterion,
};
pub use widgets::phonon_magnon_dialog::{
    PhononMagnonDialog, PhononMagnonTab, PolaritonAuditCriterion,
};
pub use widgets::quadrupole_parametric_dialog::{
    CanvasColormap, QuadrupoleAuditCriterion, QuadrupoleParametricDialog,
    QuadrupoleParametricTab, RealSpaceMode,
};
pub use widgets::non_hermitian_sensor_dialog::{
    NhSensorAuditCriterion, NonHermitianSensorDialog, NonHermitianSensorTab,
};
pub use widgets::corner_doubler_dialog::{
    CornerDoublerAuditCriterion, CornerDoublerDialog, CornerDoublerTab, DialogColormap,
    DisplayModeType,
};
pub use widgets::universal_braiding_dialog::{UniversalBraidingDialog, UniversalBraidingTab};
pub use widgets::chiral_circulator_dialog::{ChiralCirculatorDialog, ChiralCirculatorTab};
pub use widgets::josephson_parametric_dialog::{JosephsonParametricDialog, JosephsonParametricTab};
pub use widgets::optomagnonic_comb_dialog::{OptomagnonicCombDialog, OptomagnonicCombTab};
pub use widgets::floquet_sensor_dialog::{FloquetSensorDialog, FloquetSensorTab};

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(not(target_arch = "wasm32"))]
/// Determines the preferred graphics renderer based on the `PHONON_RENDERER` environment variable,
/// defaulting to OpenGL/EGL (`eframe::Renderer::Glow`) for instant sub-200ms cold startup.
pub fn determine_boot_renderer() -> eframe::Renderer {
    match std::env::var("PHONON_RENDERER").as_deref() {
        Ok("wgpu") | Ok("WGPU") => eframe::Renderer::Wgpu,
        _ => eframe::Renderer::Glow,
    }
}

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
/// Generates default `eframe::WebOptions` for WebAssembly execution inside browser canvas.
pub fn default_web_options() -> eframe::WebOptions {
    eframe::WebOptions::default()
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
/// Starts the Phonon Studio WebAssembly application mounted onto the specified HTML `<canvas>` ID.
pub async fn start(canvas_id: &str) -> Result<(), JsValue> {
    use wasm_bindgen::JsCast;
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window found"))?;
    let document = window.document().ok_or_else(|| JsValue::from_str("No document found"))?;
    let canvas = document
        .get_element_by_id(canvas_id)
        .ok_or_else(|| JsValue::from_str(&format!("Canvas element #{canvas_id} not found")))?
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| JsValue::from_str("Element is not an HtmlCanvasElement"))?;

    eframe::WebRunner::new()
        .start(
            canvas,
            default_web_options(),
            Box::new(|cc| Ok(Box::new(PhononApp::new(cc)))),
        )
        .await
}
