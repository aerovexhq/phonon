#![deny(unsafe_code)]

//! The central Phonon GUI application orchestrator, CAD layout, and interactive simulation.

use crate::extraction::ExtractionWizardDialog;
use crate::oscilloscope::{MultiGraphManager, OscilloscopePanel, WaveformTrace};
use crate::schematic::{
    compile_schematic, compute_junction_dots, compute_wire_telemetry, deserialize_project,
    load_project_from_file, save_project_to_file, serialize_project, BinaryFormatError,
    CanvasCommand, CompiledCircuit, ComponentKind, ErcDiagnostic, ErcEngine, ErcSeverity,
    HistoryStack, MultiSheetManager, NetlistSyncEngine, SchematicBus, SchematicCanvas,
    SchematicComponent, SchematicWire, SubcircuitDefinition, SubcircuitRegistry, SymbolLibrary,
};
use crate::thermal::{Colormap, ThermalOverlay};
use crate::storage::ProjectStorageManager;
use crate::widgets::{
    render_top_frame_with_app, AxionInsulatorDialog, ChernCirculatorDialog, ClusterDashboardDialog,
    CommandPalette, ComponentPalette, ConfirmationDecision, ConfirmationModal, DemoCircuitKind,
    ExceptionalPointDialog, ExceptionalSurfaceDialog, FloatingToolbarAction, FloatingToolbarState,
    FloquetMetasurfaceDialog, FqhBraidingDialog, HolonomicProcessorDialog, JtwpaDialog,
    KerrMicrocombDialog, LiebLatticeDialog, MonteCarloYieldDialog, NeuromorphicSnnDialog,
    PaletteAction, PendingAction, PolaritonCavityDialog, PreferencesDialog, ProjectDialog,
    ProjectDialogAction, SensitivityDialog, SmithChartDialog, SotiCornerDialog,
    SubcircuitDialogAction, SubcircuitPackageDialog, SymbolEditorDialog, ThermalFloorplanDialog,
    TopFrameAction, TopFrameConfig, TwistedMoireDialog, WeylSemimetalDialog, LuaConsoleDialog,
    FloquetTimeCrystalDialog, QuantumBraidingLatticeDialog, OptomechanicalSqueezingDialog,
    SkyrmionRouterDialog, AcousticSolitonDialog, ValleyMultiplexerDialog,
    NonHermitianSkinDialog, QuadrupoleShgDialog, Synthetic4dDialog, PtSymmetricDialog,
    AcousticBicDialog, EulerAcousticDialog, OctupoleInsulatorDialog, AahQuasicrystalDialog,
    ValleyHallVortexDialog, SkyrmionDeflectorDialog, FloquetFrequencyDialog,
    CornerLaserDialog, MetasurfaceHologramDialog, MajoranaSurfaceCodeDialog,
    DirectionalRadiationDialog, AtmosphericNeutronDialog,
    ThermalVacuumDialog, SpaceAvionicsBusDialog, RhbdSelfHealingDialog,
    ProductionEconomicsDialog, ChipletPackagingDialog, ElectrothermalThrottlingDialog,
    PdnDroopDialog, SiliconAgingDialog, WaferYieldDialog, DseOptimizationDialog,
    SiliconLifecycleDialog, WasmOptimizationDialog, PwaOfflineDialog,
    DesktopIpcDialog, WebRtcMeshDialog, WebGpuSpiceDialog, WavepacketScatteringDialog,
    SkyrmionReservoirDialog, PhononMagnonDialog, QuadrupoleParametricDialog,
    NonHermitianSensorDialog, CornerDoublerDialog, UniversalBraidingDialog,
    ChiralCirculatorDialog, JosephsonParametricDialog, OptomagnonicCombDialog,
    FloquetSensorDialog, OptomechanicalTransducerDialog, CornerPolaritonMicrocombDialog,
};
use crate::preferences::AppPreferences;
use crate::actions::{ActionId, ActionRegistry};
use eframe::{App, Frame};
use egui::{
    CentralPanel, Color32, FontId, Key, Panel, PointerButton, Pos2, Rect, RichText, Sense, Stroke, Ui, Vec2,
};
use phonon_core::{AutoSelectingDynamicsBackend, PhysicsDynamicsBackend};
use phonon_solver::{solve_dc_non_linear, NewtonOptions};
use std::collections::{HashMap, HashSet};

/// Current interaction mode of the CAD canvas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolMode {
    Select,
    Wire,
    Bus,
    Place(ComponentKind),
    PlaceComponent(ComponentKind),
    Probe,
}

impl ToolMode {
    pub fn is_place(&self) -> bool {
        matches!(self, ToolMode::Place(_) | ToolMode::PlaceComponent(_))
    }

    pub fn place_kind(&self) -> Option<&ComponentKind> {
        match self {
            ToolMode::Place(k) | ToolMode::PlaceComponent(k) => Some(k),
            _ => None,
        }
    }
}

/// The unified Phonon desktop CAD application.
pub struct PhononApp {
    pub canvas: SchematicCanvas,
    pub components: Vec<SchematicComponent>,
    pub wires: Vec<SchematicWire>,
    pub next_comp_id: usize,
    pub next_wire_id: usize,
    pub selected_tool: ToolMode,
    /// Set of selected component IDs for multi-selection.
    pub selected_component_ids: HashSet<usize>,
    /// Set of selected wire IDs for multi-selection.
    pub selected_wire_ids: HashSet<usize>,
    pub selected_component_id: Option<usize>,
    pub selected_wire_id: Option<usize>,
    pub active_wire_start: Option<Pos2>,
    /// Rubberband marquee drag start point in world coordinates.
    pub marquee_start: Option<Pos2>,
    /// Rubberband marquee drag current point in world coordinates.
    pub marquee_current: Option<Pos2>,
    /// Whether the floating CAD tools island is visible.
    pub show_floating_toolbar: bool,
    /// State and coordinates of the floating CAD tools island.
    pub floating_toolbar_state: FloatingToolbarState,
    /// Searchable command palette modal overlay.
    pub command_palette: CommandPalette,
    /// Centralized registry of all executable CAD commands.
    pub action_registry: ActionRegistry,
    /// Active multi-item drag tracking state.
    pub dragging_selection: bool,
    /// Original positions of components before drag started.
    pub drag_start_positions: Vec<(usize, Pos2)>,
    /// Pre-drag snapshot of schematic wires for reversible history recording.
    pub drag_start_wires: Vec<SchematicWire>,

    /// Multi-sheet schematic canvas manager.
    pub sheets: MultiSheetManager,

    /// Library of hierarchical subcircuit macro-model definitions.
    pub subcircuits: HashMap<String, SubcircuitDefinition>,

    /// High-density vectorized bus routes on the schematic canvas.
    pub buses: Vec<SchematicBus>,

    pub oscilloscope: OscilloscopePanel,
    pub multi_graph: MultiGraphManager,
    pub thermal: ThermalOverlay,

    pub show_oscilloscope: bool,
    pub show_thermal_overlay: bool,
    pub show_netlist_window: bool,

    pub sim_status: String,
    pub dc_node_voltages: HashMap<String, f64>,
    pub wire_voltages: HashMap<usize, f64>,
    pub wire_currents: HashMap<usize, f64>,
    pub component_temperatures: HashMap<String, f64>,
    pub compiled_circuit: Option<CompiledCircuit>,
    pub subcircuit_registry: SubcircuitRegistry,
    pub subcircuit_dialog: SubcircuitPackageDialog,
    pub spice_netlist_text: String,

    /// Real-time bidirectional SPICE netlist synchronization engine.
    pub netlist_sync: NetlistSyncEngine,

    /// Active Electrical Rules Check (ERC) diagnostic findings.
    pub erc_diagnostics: Vec<ErcDiagnostic>,

    /// Whether the visual ERC diagnostic overlay is rendered on canvas.
    pub show_erc_overlay: bool,

    /// SPICE model parameter extraction wizard modal dialog.
    pub extraction_wizard: ExtractionWizardDialog,

    /// Non-linear transient sensitivity analysis and worst-case optimization dialog.
    pub sensitivity_dialog: SensitivityDialog,

    /// Interactive Monte Carlo Yield & Latin Hypercube Sampling Inspector dialog.
    pub monte_carlo_dialog: MonteCarloYieldDialog,

    /// Interactive RF S-Parameters, Smith Chart & Harmonic Balance visualizer dialog.
    pub smith_chart_dialog: SmithChartDialog,

    /// Interactive Thermal Floorplan & Transient Co-Simulation Studio dialog.
    pub thermal_floorplan_dialog: ThermalFloorplanDialog,

    /// Interactive Polariton Waveguide & Topological Photonic Cavity Simulator dialog.
    pub polariton_cavity_dialog: PolaritonCavityDialog,

    /// Interactive Distributed Cloud Parameter Sweep Cluster Engine Dashboard dialog.
    pub cluster_dashboard_dialog: ClusterDashboardDialog,

    /// Interactive Neuromorphic Studio Canvas & Synaptic Weight Visualizer dialog.
    pub neuromorphic_snn_dialog: NeuromorphicSnnDialog,

    /// Interactive Exceptional Point Sensor & PT-Symmetric Circuit Simulator dialog.
    pub exceptional_point_dialog: ExceptionalPointDialog,

    /// Interactive Universal Topological Dirac & Weyl Semimetal Metamaterial Studio dialog.
    pub weyl_semimetal_dialog: WeylSemimetalDialog,

    /// Interactive Fractional Quantum Hall Anyon Braiding & Non-Abelian Topological Circuit Co-Simulator dialog.
    pub fqh_braiding_dialog: FqhBraidingDialog,

    /// Interactive Superconducting Josephson Traveling-Wave Parametric Amplifier (JTWPA) Studio dialog.
    pub jtwpa_dialog: JtwpaDialog,

    /// Interactive Floquet Engineered Spatio-Temporal Acoustic Metasurface Studio dialog.
    pub floquet_metasurface_dialog: FloquetMetasurfaceDialog,

    /// Interactive Non-Abelian Holonomic Geometric Phase Quantum Acoustic Processor dialog.
    pub holonomic_processor_dialog: HolonomicProcessorDialog,

    /// Interactive Twisted Bilayer Moiré Phonon Polariton Magic-Angle Superlattice Studio dialog.
    pub twisted_moire_dialog: TwistedMoireDialog,

    /// Interactive Acoustic Chern Insulator Chiral Circulator & Non-Reciprocal Router Studio dialog.
    pub chern_circulator_dialog: ChernCirculatorDialog,

    /// Interactive Non-Linear Soliton Kerr Microcomb Phononic Frequency Comb Studio dialog.
    pub kerr_microcomb_dialog: KerrMicrocombDialog,

    /// Interactive Non-Hermitian Chiral Exceptional Surface Acoustic Sensing Array Studio dialog.
    pub exceptional_surface_dialog: ExceptionalSurfaceDialog,

    /// Interactive Topological Higher-Order Corner State Acoustic Resonator Studio dialog.
    pub soti_corner_dialog: SotiCornerDialog,

    /// Interactive Topological Acoustic Flat-Band Lieb-Lattice Gauge Simulator Studio dialog.
    pub lieb_lattice_dialog: LiebLatticeDialog,

    /// Interactive Quantum Metamaterial Higher-Order Axion Insulator Simulator Studio dialog.
    pub axion_insulator_dialog: AxionInsulatorDialog,

    /// Interactive Floquet-Bloch Quantum Acoustic Discrete Time Crystal Simulator Studio dialog.
    pub floquet_time_crystal_dialog: FloquetTimeCrystalDialog,

    /// Interactive Quantum Acoustic Protected Braiding Lattice Studio dialog.
    pub quantum_braiding_lattice_dialog: QuantumBraidingLatticeDialog,

    /// Interactive Cavity Optomechanical Squeezing & Phonon Counting Studio dialog.
    pub optomechanical_squeezing_dialog: OptomechanicalSqueezingDialog,

    /// Interactive Topological Acoustic Skyrmion Vortex Lattice & Domain Wall Router Studio dialog.
    pub skyrmion_router_dialog: SkyrmionRouterDialog,

    /// Interactive Non-Linear Acoustic Domain Wall Kink & Soliton Waveguide Studio dialog.
    pub acoustic_soliton_dialog: AcousticSolitonDialog,

    /// Interactive Valley-Polarized Topological Acoustic Multiplexer & Beam Splitter Studio dialog.
    pub valley_multiplexer_dialog: ValleyMultiplexerDialog,

    /// Interactive Non-Hermitian Skin Effect Acoustic Sensor & Directional Funnel Studio dialog.
    pub non_hermitian_skin_dialog: NonHermitianSkinDialog,

    /// Interactive Topological Acoustic Quadrupole Second-Harmonic Generation Metamaterial Studio dialog.
    pub quadrupole_shg_dialog: QuadrupoleShgDialog,

    /// Interactive Topological Acoustic Synthetic Dimension & 4D Quantum Hall Effect Metamaterial Studio dialog.
    pub synthetic_4d_dialog: Synthetic4dDialog,

    /// Interactive Parity-Time (PT) Symmetric Acoustic Metamaterial & Unidirectional Invisibility Studio dialog.
    pub pt_symmetric_dialog: PtSymmetricDialog,

    /// Interactive Topological Acoustic Bound States in the Continuum (BIC) & Vortex Cavity Studio dialog.
    pub acoustic_bic_dialog: AcousticBicDialog,

    /// Interactive Non-Abelian Euler Class Topological Acoustic Studio dialog.
    pub euler_acoustic_dialog: EulerAcousticDialog,

    /// Interactive Higher-Order Topological Acoustic Octupole Insulator & 3D Corner State Studio dialog.
    pub octupole_insulator_dialog: OctupoleInsulatorDialog,

    /// Interactive Topological Acoustic Moire Quasicrystal & AAH Mobility Edge Studio dialog.
    pub aah_quasicrystal_dialog: AahQuasicrystalDialog,

    /// Interactive Acoustic Valley-Hall Vortex Pumping & Synthetic Chiral Gauge Field dialog.
    pub valley_hall_vortex_dialog: ValleyHallVortexDialog,

    /// Interactive Acoustic Higher-Order Skyrmion Beam Deflector & Chiral Router dialog.
    pub skyrmion_deflector_dialog: SkyrmionDeflectorDialog,

    /// Interactive Floquet Synthetic Frequency Dimension & Frequency Soliton dialog.
    pub floquet_frequency_dialog: FloquetFrequencyDialog,

    /// Interactive Non-Hermitian Higher-Order Topological Corner Laser Studio dialog.
    pub corner_laser_dialog: CornerLaserDialog,

    /// Interactive Multi-Octave Acoustic Metasurface Wavefront Hologram & Ultrasonic Tractor Beam dialog.
    pub metasurface_hologram_dialog: MetasurfaceHologramDialog,

    /// Interactive Quantum Metamaterial Non-Abelian Majorana Braid Interconnect & Surface Code Co-Processor dialog.
    pub majorana_surface_code_dialog: MajoranaSurfaceCodeDialog,

    /// Interactive Cryogenic Quantum Optomechanical Transducer & Microwave-to-Acoustic Interconnect dialog.
    pub optomechanical_transducer_dialog: OptomechanicalTransducerDialog,

    /// Interactive Topological Corner-Polariton Micro-Comb Soliton & Frequency Synthesizer dialog.
    pub corner_polariton_microcomb_dialog: CornerPolaritonMicrocombDialog,

    /// Interactive Directional Cosmic Heavy Ion Radiation Track & 3D Anisotropic Shielding Studio dialog.
    pub directional_radiation_dialog: DirectionalRadiationDialog,

    /// Interactive Atmospheric Secondary Neutron Spallation Cascade & DO-254 DAL-A SER Studio dialog.
    pub atmospheric_neutron_dialog: AtmosphericNeutronDialog,

    /// Interactive Aerospace Thermal-Vacuum Radiation Dissipation & Orbital Cycling Studio dialog.
    pub thermal_vacuum_dialog: ThermalVacuumDialog,

    /// Interactive SpaceWire/SpaceFibre & Avionics AFDX Bus Contention Studio dialog.
    pub space_avionics_bus_dialog: SpaceAvionicsBusDialog,

    /// Interactive RHBD DRC, Fast SEL Quenching & Autonomous Self-Healing Studio dialog.
    pub rhbd_self_healing_dialog: RhbdSelfHealingDialog,

    /// Interactive Production Economics & Hierarchical BOM Cost Estimator Studio dialog.
    pub production_economics_dialog: ProductionEconomicsDialog,

    /// Interactive 2.5D/3D Multi-Die & Chiplet Packaging Studio dialog.
    pub chiplet_packaging_dialog: ChipletPackagingDialog,

    /// Interactive Closed-Loop Dynamic Electro-Thermal & Power Throttling Studio dialog.
    pub electrothermal_throttling_dialog: ElectrothermalThrottlingDialog,

    /// Interactive Power Delivery Network (PDN) & Ultra-High di/dt Dynamic Droop Co-Simulator dialog.
    pub pdn_droop_dialog: PdnDroopDialog,

    /// Interactive Physics-Based Silicon Aging, Reliability & Electromigration (EM) Engine dialog.
    pub silicon_aging_dialog: SiliconAgingDialog,

    /// Interactive Wafer-Scale Yield, DFM & Harvesting Economics Co-Simulator dialog.
    pub wafer_yield_dialog: WaferYieldDialog,

    /// Interactive Automated Multi-Objective PPA-C Design Space Exploration (DSE) dialog.
    pub dse_dialog: DseOptimizationDialog,

    /// Interactive Silicon Lifecycle Management (SLM) & On-Die Telemetry Digital Twin dialog.
    pub slm_dialog: SiliconLifecycleDialog,

    /// Interactive Web Studio WASM Binary Size Optimization & Cache Invalidation dialog.
    pub wasm_optimization_dialog: WasmOptimizationDialog,

    /// Interactive Progressive Web App (PWA) Offline ServiceWorker & Asset Cache dialog.
    pub pwa_dialog: PwaOfflineDialog,

    /// Interactive Native Desktop Studio, Tauri v2 Shell & Zero-Copy Binary IPC Co-Processor dialog.
    pub desktop_ipc_dialog: DesktopIpcDialog,

    /// Interactive Real-Time Collaborative WebRTC Peer-to-Peer Multi-User CAD Mesh dialog.
    pub webrtc_mesh_dialog: WebRtcMeshDialog,

    /// Interactive WebGL2 / WebGPU Compute Shader Hardware-Accelerated SPICE Co-Processor dialog.
    pub webgpu_spice_dialog: WebGpuSpiceDialog,

    /// Interactive Microscopic Electron & Phonon Wavepacket Scattering Simulator dialog.
    pub wavepacket_scattering_dialog: WavepacketScatteringDialog,

    /// Interactive Quantum Spin-Torque Oscillator & Magnetic Skyrmion Reservoir Computing Co-Processor dialog.
    pub skyrmion_reservoir_dialog: SkyrmionReservoirDialog,

    /// Interactive Coherent Phonon-Magnon Polariton Transducer & Quantum Microwave-to-Acoustic Interface dialog.
    pub phonon_magnon_dialog: PhononMagnonDialog,

    /// Interactive Topological Higher-Order Acoustic Quadrupole Parametric Waveguide & Second-Harmonic Generation dialog.
    pub quadrupole_parametric_dialog: QuadrupoleParametricDialog,

    /// Interactive Non-Hermitian Floquet Skin-Effect Sensor & Exceptional Point Magnetometer dialog.
    pub non_hermitian_sensor_dialog: NonHermitianSensorDialog,

    /// Interactive Phonon Studio Topological Corner-Induced Acoustic Second-Harmonic Waveguide Interconnect & Nonlinear Frequency Doubler dialog.
    pub corner_doubler_dialog: CornerDoublerDialog,

    /// Interactive Universal Non-Abelian Anyon Braiding & Topological Quantum Acoustic Co-Processor dialog.
    pub universal_braiding_dialog: UniversalBraidingDialog,

    /// Interactive Topological Floquet Chiral Magnon-Phonon Polariton Circulator & Cryogenic Isolator dialog.
    pub chiral_circulator_dialog: ChiralCirculatorDialog,

    /// Interactive Superconducting Josephson Parametric Acoustic Waveguide Amplification & Squeezed Vacuum dialog.
    pub josephson_parametric_dialog: JosephsonParametricDialog,

    /// Interactive Cavity Optomagnonic Polariton Frequency Comb & Dissipative Kerr Soliton dialog.
    pub optomagnonic_comb_dialog: OptomagnonicCombDialog,

    /// Interactive Floquet Discrete Time-Crystal Magnetometer & Subharmonic Sensor Network dialog.
    pub floquet_sensor_dialog: FloquetSensorDialog,

    /// Interactive Lua Testbench Scripting Console & Expression Grapher dialog.
    pub lua_console_dialog: LuaConsoleDialog,

    /// Interactive Logisim/KiCad-style component symbol and shape editor dialog.
    pub symbol_editor: SymbolEditorDialog,

    /// User library storing custom component symbol definitions.
    pub symbol_library: SymbolLibrary,

    // Drag tracking for selected component
    pub dragging_component: bool,
    pub drag_start_pos: Option<(usize, Pos2)>,

    /// Reversible undo/redo history stack and command engine.
    pub history: HistoryStack,

    /// Active editing buffer tracking component value before modification.
    editing_comp_value: Option<(usize, String)>,

    /// Active rotation angle index for components being placed (0 = 0 deg, 1 = 90 deg, 2 = 180 deg, 3 = 270 deg).
    pub placement_rotation: u8,

    /// Configuration for the custom window top frame.
    pub top_frame_config: TopFrameConfig,

    /// Active physics dynamics backend driving physical simulation state.
    pub dynamics_backend: Box<dyn PhysicsDynamicsBackend>,

    /// Hierarchical categorized component palette drawer.
    pub palette: ComponentPalette,

    /// Whether the left component palette panel is visible (collapsible to burger menu).
    pub show_palette: bool,

    /// Instant boot theme configuration.
    pub boot_theme_config: crate::BootThemeConfig,

    /// Whether auto-centering of the viewport on bounding box is requested.
    pub pending_auto_center: bool,

    /// Active project title (displayed in top frame title bar).
    pub project_title: String,

    /// Whether project has unsaved modifications.
    pub is_modified: bool,

    /// Session modification epoch tracking changes since clean baseline.
    pub modification_epoch: u64,

    /// Last confirmed clean epoch timestamp.
    pub clean_epoch: u64,

    /// Active file system path for project saving on desktop.
    pub current_project_path: Option<std::path::PathBuf>,

    /// Pending lifecycle action awaiting confirmation modal decision.
    pub pending_confirmation_action: Option<PendingAction>,

    /// Unified project storage and virtual file system manager.
    pub storage_manager: ProjectStorageManager,

    /// Project Manager modal dialog.
    pub project_dialog: ProjectDialog,

    /// Application preferences and configuration state.
    pub preferences: AppPreferences,

    /// Preferences modal settings dialog.
    pub preferences_dialog: PreferencesDialog,

    /// Whether close has been confirmed and should trigger viewport close.
    pub should_close: bool,

    /// Last recorded interactive canvas viewport rect.
    pub last_canvas_rect: egui::Rect,
}

impl std::fmt::Debug for PhononApp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PhononApp")
            .field("next_comp_id", &self.next_comp_id)
            .field("next_wire_id", &self.next_wire_id)
            .field("selected_tool", &self.selected_tool)
            .field("top_frame_config", &self.top_frame_config)
            .field("dynamics_backend", &self.dynamics_backend.info())
            .field("palette", &self.palette)
            .field("boot_theme_config", &self.boot_theme_config)
            .field("project_title", &self.project_title)
            .field("is_modified", &self.is_modified)
            .field("modification_epoch", &self.modification_epoch)
            .field("clean_epoch", &self.clean_epoch)
            .finish()
    }
}

impl Default for PhononApp {
    fn default() -> Self {
        let mut app = Self {
            canvas: SchematicCanvas::new(),
            components: Vec::with_capacity(64),
            wires: Vec::with_capacity(64),
            next_comp_id: 1,
            next_wire_id: 1,
            selected_tool: ToolMode::Select,
            selected_component_ids: HashSet::new(),
            selected_wire_ids: HashSet::new(),
            selected_component_id: None,
            selected_wire_id: None,
            active_wire_start: None,
            marquee_start: None,
            marquee_current: None,
            show_floating_toolbar: true,
            floating_toolbar_state: FloatingToolbarState::new(),
            command_palette: CommandPalette::new(),
            action_registry: ActionRegistry::new(),
            dragging_selection: false,
            drag_start_positions: Vec::new(),
            drag_start_wires: Vec::new(),
            sheets: MultiSheetManager::new("Main"),
            subcircuits: HashMap::new(),
            buses: Vec::new(),
            oscilloscope: OscilloscopePanel::new(),
            multi_graph: MultiGraphManager::new(),
            thermal: ThermalOverlay::new(),
            show_oscilloscope: true,
            show_thermal_overlay: true,
            show_netlist_window: false,
            sim_status: String::new(),
            dc_node_voltages: HashMap::with_capacity(32),
            wire_voltages: HashMap::with_capacity(64),
            wire_currents: HashMap::with_capacity(64),
            component_temperatures: HashMap::with_capacity(32),
            compiled_circuit: None,
            subcircuit_registry: SubcircuitRegistry::new(),
            subcircuit_dialog: SubcircuitPackageDialog::new(),
            spice_netlist_text: String::new(),
            netlist_sync: NetlistSyncEngine::new(),
            erc_diagnostics: Vec::new(),
            show_erc_overlay: true,
            extraction_wizard: ExtractionWizardDialog::new(),
            sensitivity_dialog: SensitivityDialog::new(),
            monte_carlo_dialog: MonteCarloYieldDialog::new(),
            smith_chart_dialog: SmithChartDialog::new(),
            thermal_floorplan_dialog: ThermalFloorplanDialog::new(),
            polariton_cavity_dialog: PolaritonCavityDialog::new(),
            cluster_dashboard_dialog: ClusterDashboardDialog::new(),
            neuromorphic_snn_dialog: NeuromorphicSnnDialog::new(),
            exceptional_point_dialog: ExceptionalPointDialog::new(),
            weyl_semimetal_dialog: WeylSemimetalDialog::new(),
            fqh_braiding_dialog: FqhBraidingDialog::new(),
            jtwpa_dialog: JtwpaDialog::new(),
            floquet_metasurface_dialog: FloquetMetasurfaceDialog::new(),
            holonomic_processor_dialog: HolonomicProcessorDialog::new(),
            twisted_moire_dialog: TwistedMoireDialog::new(),
            chern_circulator_dialog: ChernCirculatorDialog::new(),
            kerr_microcomb_dialog: KerrMicrocombDialog::new(),
            exceptional_surface_dialog: ExceptionalSurfaceDialog::new(),
            soti_corner_dialog: SotiCornerDialog::new(),
            lieb_lattice_dialog: LiebLatticeDialog::new(),
            axion_insulator_dialog: AxionInsulatorDialog::new(),
            floquet_time_crystal_dialog: FloquetTimeCrystalDialog::new(),
            quantum_braiding_lattice_dialog: QuantumBraidingLatticeDialog::new(),
            optomechanical_squeezing_dialog: OptomechanicalSqueezingDialog::new(),
            skyrmion_router_dialog: SkyrmionRouterDialog::new(),
            acoustic_soliton_dialog: AcousticSolitonDialog::new(),
            valley_multiplexer_dialog: ValleyMultiplexerDialog::new(),
            non_hermitian_skin_dialog: NonHermitianSkinDialog::new(),
            quadrupole_shg_dialog: QuadrupoleShgDialog::new(),
            synthetic_4d_dialog: Synthetic4dDialog::new(),
            pt_symmetric_dialog: PtSymmetricDialog::new(),
            acoustic_bic_dialog: AcousticBicDialog::new(),
            euler_acoustic_dialog: EulerAcousticDialog::new(),
            octupole_insulator_dialog: OctupoleInsulatorDialog::new(),
            aah_quasicrystal_dialog: AahQuasicrystalDialog::new(),
            valley_hall_vortex_dialog: ValleyHallVortexDialog::new(),
            skyrmion_deflector_dialog: SkyrmionDeflectorDialog::new(),
            floquet_frequency_dialog: FloquetFrequencyDialog::new(),
            corner_laser_dialog: CornerLaserDialog::new_fast(),
            metasurface_hologram_dialog: MetasurfaceHologramDialog::new_fast(),
            majorana_surface_code_dialog: MajoranaSurfaceCodeDialog::new_fast(),
            optomechanical_transducer_dialog: OptomechanicalTransducerDialog::new_fast(),
            corner_polariton_microcomb_dialog: CornerPolaritonMicrocombDialog::new_fast(),
            directional_radiation_dialog: DirectionalRadiationDialog::new_fast(),
            atmospheric_neutron_dialog: AtmosphericNeutronDialog::new_fast(),
            thermal_vacuum_dialog: ThermalVacuumDialog::new_fast(),
            space_avionics_bus_dialog: SpaceAvionicsBusDialog::new_fast(),
            rhbd_self_healing_dialog: RhbdSelfHealingDialog::new_fast(),
            production_economics_dialog: ProductionEconomicsDialog::new_fast(),
            chiplet_packaging_dialog: ChipletPackagingDialog::new_fast(),
            electrothermal_throttling_dialog: ElectrothermalThrottlingDialog::new_fast(),
            pdn_droop_dialog: PdnDroopDialog::new_fast(),
            silicon_aging_dialog: SiliconAgingDialog::new_fast(),
            wafer_yield_dialog: WaferYieldDialog::new_fast(),
            dse_dialog: DseOptimizationDialog::new_fast(),
            slm_dialog: SiliconLifecycleDialog::new_fast(),
            wasm_optimization_dialog: WasmOptimizationDialog::new_fast(),
            pwa_dialog: PwaOfflineDialog::new_fast(),
            desktop_ipc_dialog: DesktopIpcDialog::new_fast(),
            webrtc_mesh_dialog: WebRtcMeshDialog::new_fast(),
            webgpu_spice_dialog: WebGpuSpiceDialog::new_fast(),
            wavepacket_scattering_dialog: WavepacketScatteringDialog::new_fast(),
            skyrmion_reservoir_dialog: SkyrmionReservoirDialog::new_fast(),
            phonon_magnon_dialog: PhononMagnonDialog::new_fast(),
            quadrupole_parametric_dialog: QuadrupoleParametricDialog::new_fast(),
            non_hermitian_sensor_dialog: NonHermitianSensorDialog::new_fast(),
            corner_doubler_dialog: CornerDoublerDialog::new_fast(),
            universal_braiding_dialog: UniversalBraidingDialog::new(),
            chiral_circulator_dialog: ChiralCirculatorDialog::new(),
            josephson_parametric_dialog: JosephsonParametricDialog::new_fast(),
            optomagnonic_comb_dialog: OptomagnonicCombDialog::new_fast(),
            floquet_sensor_dialog: FloquetSensorDialog::new_fast(),
            lua_console_dialog: LuaConsoleDialog::new(),
            symbol_editor: SymbolEditorDialog::new(),
            symbol_library: SymbolLibrary::new(),
            dragging_component: false,
            drag_start_pos: None,
            history: HistoryStack::with_capacity(500, 64),
            editing_comp_value: None,
            placement_rotation: 0,
            top_frame_config: TopFrameConfig::default(),
            dynamics_backend: Box::new(AutoSelectingDynamicsBackend::new()),
            palette: ComponentPalette::new(),
            show_palette: true,
            boot_theme_config: crate::default_boot_theme_config(),
            pending_auto_center: true,
            project_title: "Untitled1".to_string(),
            is_modified: false,
            modification_epoch: 0,
            clean_epoch: 0,
            current_project_path: None,
            pending_confirmation_action: None,
            storage_manager: ProjectStorageManager::new(),
            project_dialog: ProjectDialog::new(),
            preferences: AppPreferences::default(),
            preferences_dialog: PreferencesDialog::new(),
            should_close: false,
            last_canvas_rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::new(800.0, 600.0)),
        };

        // Load saved preferences
        let loaded_prefs = AppPreferences::load_from_storage(&app.storage_manager);
        app.canvas.show_grid = loaded_prefs.show_grid;
        app.canvas.grid_size = loaded_prefs.grid_size;
        app.show_thermal_overlay = loaded_prefs.thermal_overlay_enabled;
        app.thermal.colormap = match loaded_prefs.thermal_colormap.as_str() {
            "Magma" => Colormap::Magma,
            "Inferno" => Colormap::Inferno,
            _ => Colormap::Turbo,
        };
        app.history.max_depth = loaded_prefs.in_memory_history_limit;
        app.preferences = loaded_prefs;

        // If a saved project from last time exists, continue from there. Otherwise load default Voltage Divider demo.
        if !app.try_restore_last_project() {
            app.load_voltage_divider_demo();
            app.project_title = "Untitled1".to_string();
            app.top_frame_config.circuit_name = "Untitled1".to_string();
        }
        app.history.clear();
        app.modification_epoch = 0;
        app.clean_epoch = 0;
        app.is_modified = false;
        app.top_frame_config.is_modified = false;
        app
    }
}

impl PhononApp {
    /// Creates a default `PhononApp` with an auto-selecting dynamics backend.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_theme(egui::Theme::Dark);
        Self::default()
    }

    /// Creates a `PhononApp` instance with an injected custom physics dynamics backend.
    pub fn with_backend(
        cc: &eframe::CreationContext<'_>,
        backend: Box<dyn PhysicsDynamicsBackend>,
    ) -> Self {
        cc.egui_ctx.set_theme(egui::Theme::Dark);
        let mut app = Self::default();
        app.dynamics_backend = backend;
        app
    }

    /// Returns a shared reference to the active physics dynamics backend.
    pub fn backend(&self) -> &dyn PhysicsDynamicsBackend {
        self.dynamics_backend.as_ref()
    }

    /// Returns a mutable reference to the active physics dynamics backend.
    pub fn backend_mut(&mut self) -> &mut dyn PhysicsDynamicsBackend {
        self.dynamics_backend.as_mut()
    }

    /// Returns the world-coordinate bounding box of the schematic canvas content.
    pub fn bounding_box(&self) -> Option<egui::Rect> {
        self.canvas.bounding_box()
    }

    /// Automatically centers and fits the schematic canvas within the given viewport rect.
    pub fn center_on_bounding_box(&mut self, viewport: egui::Rect) {
        self.canvas.center_on_bounding_box(viewport);
    }

    /// Updates the active project title and synchronization metadata.
    pub fn rename_project(&mut self, new_name: impl Into<String>) {
        let name = new_name.into();
        if self.project_title != name {
            self.project_title = name;
            self.top_frame_config.circuit_name = self.project_title.clone();
            self.mark_dirty();
        }
    }

    /// Checks whether the project has unsaved modifications.
    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.modification_epoch != self.clean_epoch
    }

    /// Marks the project state as modified and records a new modification epoch.
    pub fn mark_dirty(&mut self) {
        self.modification_epoch += 1;
        self.is_modified = true;
        self.top_frame_config.is_modified = true;
        self.autosave_current_project();
    }

    /// Marks the project state as clean, synchronizing clean_epoch to modification_epoch.
    pub fn mark_clean(&mut self) {
        self.clean_epoch = self.modification_epoch;
        self.is_modified = false;
        self.top_frame_config.is_modified = false;
    }

    /// Intercepts user requests (demo load, new project, clear canvas, close) with confirmation if dirty.
    pub fn request_action(&mut self, action: PendingAction) {
        if self.is_dirty() {
            self.pending_confirmation_action = Some(action);
        } else {
            self.execute_confirmation_action(action);
        }
    }

    /// Executes a confirmed or uncontested lifecycle action.
    pub fn execute_confirmation_action(&mut self, action: PendingAction) {
        match action {
            PendingAction::NewProject => self.new_project(),
            PendingAction::OpenProject(path) => {
                if let Some(p) = path {
                    #[cfg(not(target_arch = "wasm32"))]
                    let is_file = p.exists();
                    #[cfg(target_arch = "wasm32")]
                    let is_file = false;

                    if is_file {
                        let _ = self.load_project_from_path(p);
                    } else {
                        let name = p.to_string_lossy().to_string();
                        let _ = self.load_project_by_name(&name);
                    }
                } else {
                    self.open_open_dialog();
                }
            }
            PendingAction::LoadDemo(demo) => match demo {
                DemoCircuitKind::VoltageDivider => self.load_voltage_divider_demo(),
                DemoCircuitKind::DiodeClipper => self.load_diode_clipper_demo(),
                DemoCircuitKind::BjtAmplifier => self.load_bjt_amplifier_demo(),
                DemoCircuitKind::CmosInverter => self.load_cmos_inverter_demo(),
                DemoCircuitKind::NmosSwitch => self.load_nmos_switch_demo(),
            },
            PendingAction::ClearCanvas => self.clear_canvas_user(),
            PendingAction::CloseApp => {
                self.should_close = true;
            }
        }
    }

    /// Initializes a fresh untitled project.
    pub fn new_project(&mut self) {
        self.clear_canvas_state();
        self.history.clear();
        self.project_title = "Untitled1".to_string();
        self.current_project_path = None;
        self.modification_epoch = 0;
        self.clean_epoch = 0;
        self.is_modified = false;
        self.top_frame_config.circuit_name = self.project_title.clone();
        self.top_frame_config.is_modified = false;
        self.pending_auto_center = true;
        self.sim_status = "New Project".to_string();
    }

    /// Clears canvas by explicit user request and marks project as modified.
    pub fn clear_canvas_user(&mut self) {
        self.clear_all();
        self.mark_dirty();
        self.sim_status = "Canvas cleared".to_string();
    }

    /// Opens the Project Manager in Open mode.
    pub fn open_open_dialog(&mut self) {
        self.project_dialog.open_for_open();
    }

    /// Opens the Project Manager in SaveAs mode.
    pub fn open_save_as_dialog(&mut self) {
        self.project_dialog.open_for_save_as(&self.project_title);
    }

    /// Saves the current project state to storage and filesystem.
    pub fn save_project(&mut self) -> Result<(), String> {
        let title = self.project_title.clone();
        self.storage_manager
            .save_project(&title, &self.components, &self.wires)?;

        if let Some(path) = &self.current_project_path {
            let _ = save_project_to_file(path, &title, &self.components, &self.wires);
        }

        self.mark_clean();
        self.sim_status = format!("Project '{}' saved", title);
        Ok(())
    }

    /// Saves the current project state under a new title.
    pub fn save_project_as(&mut self, new_name: &str) -> Result<(), String> {
        self.rename_project(new_name);
        self.save_project()
    }

    /// Loads a project by name from the virtual project file system.
    pub fn load_project_by_name(&mut self, name: &str) -> Result<(), String> {
        let proj = self.storage_manager.load_project(name)?;
        self.clear_canvas_state();
        self.history.clear();
        self.project_title = proj.title;
        self.components = proj.components;
        self.wires = proj.wires;
        self.next_comp_id = self.components.iter().map(|c| c.id).max().unwrap_or(0) + 1;
        self.next_wire_id = self.wires.iter().map(|w| w.id).max().unwrap_or(0) + 1;
        self.sync_canvas_state();
        self.modification_epoch = 0;
        self.clean_epoch = 0;
        self.is_modified = false;
        self.top_frame_config.circuit_name = self.project_title.clone();
        self.top_frame_config.is_modified = false;
        self.pending_auto_center = true;
        self.run_erc();
        self.sim_status = format!("Loaded project '{}'", name);
        Ok(())
    }

    /// Loads a project from an explicit filesystem path on disk.
    pub fn load_project_from_path(&mut self, path: impl AsRef<std::path::Path>) -> Result<(), String> {
        let p = path.as_ref();
        let proj = load_project_from_file(p).map_err(|e| e.to_string())?;
        self.clear_canvas_state();
        self.history.clear();
        self.project_title = proj.title;
        self.current_project_path = Some(p.to_path_buf());
        self.components = proj.components;
        self.wires = proj.wires;
        self.next_comp_id = self.components.iter().map(|c| c.id).max().unwrap_or(0) + 1;
        self.next_wire_id = self.wires.iter().map(|w| w.id).max().unwrap_or(0) + 1;
        self.sync_canvas_state();
        self.modification_epoch = 0;
        self.clean_epoch = 0;
        self.is_modified = false;
        self.top_frame_config.circuit_name = self.project_title.clone();
        self.top_frame_config.is_modified = false;
        self.pending_auto_center = true;
        self.run_erc();
        self.sim_status = format!("Loaded from {}", p.display());
        Ok(())
    }

    /// Automatically records the active project in autosave session storage.
    pub fn autosave_current_project(&mut self) {
        let _ = self.storage_manager.save_autosave(
            &self.project_title,
            &self.components,
            &self.wires,
        );
    }

    /// Attempts to restore the previous session from autosave storage. Returns true if restored.
    pub fn try_restore_last_project(&mut self) -> bool {
        if let Some(proj) = self.storage_manager.load_autosave() {
            if !proj.components.is_empty() || !proj.wires.is_empty() {
                self.clear_canvas_state();
                self.history.clear();
                self.project_title = proj.title;
                self.components = proj.components;
                self.wires = proj.wires;
                self.next_comp_id = self.components.iter().map(|c| c.id).max().unwrap_or(0) + 1;
                self.next_wire_id = self.wires.iter().map(|w| w.id).max().unwrap_or(0) + 1;
                self.sync_canvas_state();
                self.pending_auto_center = true;
                return true;
            }
        }
        false
    }

    /// Exports a project payload to a browser download or desktop file.
    pub fn export_project(&mut self, name: &str) {
        let data = if name == self.project_title {
            serialize_project(&self.project_title, &self.components, &self.wires)
        } else if let Ok(proj) = self.storage_manager.load_project(name) {
            serialize_project(&proj.title, &proj.components, &proj.wires)
        } else {
            return;
        };

        let filename = format!("{}.phn", name);
        #[cfg(target_arch = "wasm32")]
        {
            let _ = crate::storage::BrowserStorageAdapter::trigger_download(&filename, &data);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let export_dir = std::env::temp_dir().join("phonon").join("exports");
            let _ = std::fs::create_dir_all(&export_dir);
            let path = export_dir.join(&filename);
            let _ = std::fs::write(&path, &data);
            self.sim_status = format!("Exported to {}", path.display());
        }
    }

    /// Clears canvas state without pushing an undo command.
    pub fn clear_canvas_state(&mut self) {
        self.components.clear();
        self.wires.clear();
        self.buses.clear();
        self.next_comp_id = 1;
        self.next_wire_id = 1;
        self.selected_component_id = None;
        self.selected_wire_id = None;
        self.clear_selection();
        self.active_wire_start = None;
        self.dc_node_voltages.clear();
        self.wire_voltages.clear();
        self.wire_currents.clear();
        self.component_temperatures.clear();
        self.compiled_circuit = None;
        self.spice_netlist_text.clear();
        self.sim_status.clear();
        self.drag_start_pos = None;
        self.drag_start_positions.clear();
        self.drag_start_wires.clear();
        self.dragging_selection = false;
        self.editing_comp_value = None;
        self.canvas.clear();
        self.erc_diagnostics.clear();
        self.netlist_sync.invalidate();
    }

    /// Synchronizes app components, wires, and buses into the active canvas object and active sheet.
    pub fn sync_canvas_state(&mut self) {
        self.canvas.components = self.components.clone();
        self.canvas.wires = self.wires.clone();
        self.canvas.buses = self.buses.clone();
        self.sync_selection_to_canvas();

        let active = self.sheets.active_sheet_mut();
        active.canvas.components = self.components.clone();
        active.canvas.wires = self.wires.clone();
        active.canvas.subcircuit_instances = self.canvas.subcircuit_instances.clone();
        active.canvas.buses = self.buses.clone();
        active.camera_offset = self.canvas.pan;
        active.camera_zoom = self.canvas.zoom;
    }

    /// Adds a new sheet, synchronizing the current canvas, and switches to the new sheet.
    pub fn add_sheet(&mut self, name: &str) -> usize {
        self.sync_canvas_state();
        let idx = self.sheets.add_sheet(name);
        self.switch_to_sheet(idx);
        idx
    }

    /// Removes a sheet by index, preventing deletion of the last remaining sheet.
    pub fn remove_sheet(&mut self, idx: usize) -> Result<(), String> {
        self.sync_canvas_state();
        self.sheets.remove_sheet(idx)?;
        self.load_active_sheet();
        Ok(())
    }

    /// Switches the active sheet, saving the current sheet canvas state and restoring target sheet state.
    pub fn switch_to_sheet(&mut self, idx: usize) -> bool {
        self.sync_canvas_state();
        if self.sheets.switch_sheet(idx) {
            self.load_active_sheet();
            true
        } else {
            false
        }
    }

    /// Loads the active sheet's canvas and camera into the primary app workspace.
    pub fn load_active_sheet(&mut self) {
        let active = self.sheets.active_sheet();
        self.components = active.canvas.components.clone();
        self.wires = active.canvas.wires.clone();
        self.canvas.components = self.components.clone();
        self.canvas.wires = self.wires.clone();
        self.canvas.subcircuit_instances = active.canvas.subcircuit_instances.clone();
        self.canvas.buses = active.canvas.buses.clone();
        self.buses = active.canvas.buses.clone();
        self.canvas.pan = active.camera_offset;
        self.canvas.zoom = active.camera_zoom;
        self.selected_component_id = None;
        self.selected_wire_id = None;
        self.active_wire_start = None;
        self.netlist_sync.invalidate();
    }

    /// Runs the Electrical Rules Check (ERC) diagnostic engine on the current schematic canvas.
    pub fn run_erc(&mut self) {
        self.sync_canvas_state();
        self.erc_diagnostics = ErcEngine::evaluate_canvas(&self.canvas);
        let error_count = self
            .erc_diagnostics
            .iter()
            .filter(|d| d.severity == ErcSeverity::Error)
            .count();
        let warn_count = self
            .erc_diagnostics
            .iter()
            .filter(|d| d.severity == ErcSeverity::Warning)
            .count();
        self.sim_status = format!(
            "ERC Checked: {} errors, {} warnings",
            error_count, warn_count
        );
    }

    /// Clears the canvas, removing all components and wires, recording the action in history.
    pub fn clear_all(&mut self) {
        if !self.components.is_empty() || !self.wires.is_empty() {
            self.history.record(CanvasCommand::ClearAll {
                components: self.components.clone(),
                wires: self.wires.clone(),
            });
            self.mark_dirty();
        }
        self.clear_canvas_state();
    }

    /// Reverses the most recent canvas mutation action from the history stack.
    pub fn undo(&mut self) -> bool {
        let success = self.history.undo(&mut self.components, &mut self.wires);
        if success {
            self.selected_component_id = None;
            self.selected_wire_id = None;
            self.selected_component_ids.clear();
            self.selected_wire_ids.clear();
            self.wires.sort_by_key(|w| w.id);
            self.sync_canvas_state();
            self.active_wire_start = None;
            self.sim_status = "Undo".to_string();
            let max_c_id = self.components.iter().map(|c| c.id).max().unwrap_or(0);
            if self.next_comp_id <= max_c_id {
                self.next_comp_id = max_c_id + 1;
            }
            let max_w_id = self.wires.iter().map(|w| w.id).max().unwrap_or(0);
            if self.next_wire_id <= max_w_id {
                self.next_wire_id = max_w_id + 1;
            }
            self.mark_dirty();
        }
        success
    }

    /// Re-applies the most recent undone canvas mutation action from the history stack.
    pub fn redo(&mut self) -> bool {
        let success = self.history.redo(&mut self.components, &mut self.wires);
        if success {
            self.selected_component_id = None;
            self.selected_wire_id = None;
            self.selected_component_ids.clear();
            self.selected_wire_ids.clear();
            self.wires.sort_by_key(|w| w.id);
            self.sync_canvas_state();
            self.active_wire_start = None;
            self.sim_status = "Redo".to_string();
            let max_c_id = self.components.iter().map(|c| c.id).max().unwrap_or(0);
            if self.next_comp_id <= max_c_id {
                self.next_comp_id = max_c_id + 1;
            }
            let max_w_id = self.wires.iter().map(|w| w.id).max().unwrap_or(0);
            if self.next_wire_id <= max_w_id {
                self.next_wire_id = max_w_id + 1;
            }
            self.mark_dirty();
        }
        success
    }

    /// Modifies a component value and records the change into history.
    pub fn modify_component_value(&mut self, id: usize, new_val: impl Into<String>) {
        let new_val = new_val.into();
        if let Some(comp) = self.components.iter_mut().find(|c| c.id == id) {
            if comp.value_str != new_val {
                let old_val = std::mem::replace(&mut comp.value_str, new_val.clone());
                self.history.record(CanvasCommand::ModifyComponentValue {
                    id,
                    old_val,
                    new_val,
                });
                self.mark_dirty();
            }
        }
    }

    /// Serializes current schematic project into ultra-compact binary format (.phn).
    pub fn save_to_bytes(&self) -> Vec<u8> {
        let title = &self.top_frame_config.circuit_name;
        serialize_project(title, &self.components, &self.wires)
    }

    /// Loads schematic project from ultra-compact binary format (.phn) bytes, resetting history clean index.
    pub fn load_from_bytes(&mut self, bytes: &[u8]) -> Result<(), BinaryFormatError> {
        let proj = deserialize_project(bytes)?;
        self.components = proj.components;
        self.wires = proj.wires;
        self.history.clear();
        let max_c_id = self.components.iter().map(|c| c.id).max().unwrap_or(0);
        self.next_comp_id = max_c_id + 1;
        let max_w_id = self.wires.iter().map(|w| w.id).max().unwrap_or(0);
        self.next_wire_id = max_w_id + 1;
        self.selected_component_id = None;
        self.selected_wire_id = None;
        self.active_wire_start = None;
        self.dc_node_voltages.clear();
        self.component_temperatures.clear();
        self.compiled_circuit = None;
        self.spice_netlist_text.clear();
        self.top_frame_config.circuit_name = proj.title.clone();
        self.sim_status = format!("Loaded project: {}", proj.title);
        Ok(())
    }

    /// Serializes project and resets the history clean index to mark current state as saved.
    pub fn save_project_to_bytes(&mut self) -> Vec<u8> {
        self.history.mark_clean();
        self.save_to_bytes()
    }

    /// Saves project state to disk at `path` and resets history clean index.
    pub fn save_project_file<P: AsRef<std::path::Path>>(
        &mut self,
        path: P,
    ) -> Result<(), BinaryFormatError> {
        self.history.mark_clean();
        let title = &self.top_frame_config.circuit_name;
        save_project_to_file(path, title, &self.components, &self.wires)?;
        self.sim_status = "Saved project (.phn)".to_string();
        Ok(())
    }

    /// Loads project state from disk at `path` and resets history clean index.
    pub fn load_project_file<P: AsRef<std::path::Path>>(
        &mut self,
        path: P,
    ) -> Result<(), BinaryFormatError> {
        let proj = load_project_from_file(path)?;
        self.components = proj.components;
        self.wires = proj.wires;
        self.history.clear();
        let max_c_id = self.components.iter().map(|c| c.id).max().unwrap_or(0);
        self.next_comp_id = max_c_id + 1;
        let max_w_id = self.wires.iter().map(|w| w.id).max().unwrap_or(0);
        self.next_wire_id = max_w_id + 1;
        self.selected_component_id = None;
        self.selected_wire_id = None;
        self.active_wire_start = None;
        self.dc_node_voltages.clear();
        self.component_temperatures.clear();
        self.compiled_circuit = None;
        self.spice_netlist_text.clear();
        self.top_frame_config.circuit_name = proj.title.clone();
        self.sim_status = format!("Loaded project: {}", proj.title);
        Ok(())
    }

    /// Loads an interactive Voltage Divider demo circuit.
    pub fn load_voltage_divider_demo(&mut self) {
        self.clear_canvas_state();
        self.history.clear();

        // 1. Components
        let v1 =
            SchematicComponent::new(1, ComponentKind::VoltageSource, Pos2::new(200.0, 300.0), 1);
        let r1 = SchematicComponent::new(2, ComponentKind::Resistor, Pos2::new(360.0, 240.0), 1);
        let r2 = SchematicComponent::new(3, ComponentKind::Resistor, Pos2::new(360.0, 360.0), 2);
        let gnd = SchematicComponent::new(4, ComponentKind::Ground, Pos2::new(200.0, 440.0), 1);

        self.components = vec![v1, r1, r2, gnd];
        self.next_comp_id = 5;

        // 2. Wires
        let w1 =
            SchematicWire::manhattan_route_vh(1, Pos2::new(200.0, 260.0), Pos2::new(360.0, 200.0));
        let w2 =
            SchematicWire::manhattan_route(2, Pos2::new(360.0, 280.0), Pos2::new(360.0, 320.0));
        let w3 =
            SchematicWire::manhattan_route(3, Pos2::new(360.0, 400.0), Pos2::new(200.0, 340.0));
        let w4 =
            SchematicWire::manhattan_route(4, Pos2::new(200.0, 340.0), Pos2::new(200.0, 420.0));

        self.wires = vec![w1, w2, w3, w4];
        self.next_wire_id = 5;

        self.sync_canvas_state();
        self.run_erc();
        self.sim_status.clear();
        self.pending_auto_center = true;
        self.project_title = "Voltage Divider".to_string();
        self.current_project_path = None;
        self.modification_epoch = 0;
        self.clean_epoch = 0;
        self.is_modified = false;
        self.top_frame_config.circuit_name = self.project_title.clone();
        self.top_frame_config.is_modified = false;
    }

    /// Loads an interactive Diode Limiter / Clipper demo circuit.
    pub fn load_diode_clipper_demo(&mut self) {
        self.clear_canvas_state();
        self.history.clear();

        let v1 =
            SchematicComponent::new(1, ComponentKind::VoltageSource, Pos2::new(180.0, 300.0), 1);
        let r1 = SchematicComponent::new(2, ComponentKind::Resistor, Pos2::new(320.0, 240.0), 1);
        let d1 = SchematicComponent::new(3, ComponentKind::Diode, Pos2::new(440.0, 320.0), 1);
        let gnd = SchematicComponent::new(4, ComponentKind::Ground, Pos2::new(180.0, 440.0), 1);

        self.components = vec![v1, r1, d1, gnd];
        self.next_comp_id = 5;

        let w1 =
            SchematicWire::manhattan_route(1, Pos2::new(180.0, 260.0), Pos2::new(320.0, 200.0));
        let w2 =
            SchematicWire::manhattan_route(2, Pos2::new(320.0, 280.0), Pos2::new(440.0, 280.0));
        let w3 =
            SchematicWire::manhattan_route(3, Pos2::new(440.0, 360.0), Pos2::new(180.0, 340.0));
        let w4 =
            SchematicWire::manhattan_route(4, Pos2::new(180.0, 340.0), Pos2::new(180.0, 420.0));

        self.wires = vec![w1, w2, w3, w4];
        self.next_wire_id = 5;

        self.sync_canvas_state();
        self.run_erc();
        self.sim_status.clear();
        self.pending_auto_center = true;
        self.project_title = "Diode Clipper".to_string();
        self.current_project_path = None;
        self.modification_epoch = 0;
        self.clean_epoch = 0;
        self.is_modified = false;
        self.top_frame_config.circuit_name = self.project_title.clone();
        self.top_frame_config.is_modified = false;
    }

    /// Loads an interactive BJT Common Emitter Amplifier demo circuit.
    pub fn load_bjt_amplifier_demo(&mut self) {
        self.clear_canvas_state();
        self.history.clear();

        // 1. Components
        // VCC: 12V Supply at (500, 280) -> (+) at (500, 240), (-) at (500, 320)
        let vcc = SchematicComponent::new(1, ComponentKind::VoltageSource, Pos2::new(500.0, 280.0), 1)
            .with_value("12.0");
        // RC: Collector Resistor 2.2k at (420, 280) -> 1=(420, 240), 2=(420, 320)
        let rc = SchematicComponent::new(2, ComponentKind::Resistor, Pos2::new(420.0, 280.0), 1)
            .with_value("2.2k");
        // Q1: BJT NPN Transistor at (400, 380) -> C=(420, 340), B=(380, 380), E=(420, 420)
        let q1 = SchematicComponent::new(3, ComponentKind::BjtNpn, Pos2::new(400.0, 380.0), 1);
        // RE: Emitter Resistor 470 at (420, 480) -> 1=(420, 440), 2=(420, 520)
        let re = SchematicComponent::new(4, ComponentKind::Resistor, Pos2::new(420.0, 480.0), 2)
            .with_value("470");
        // RB1: Base Bias Upper 22k at (300, 280) -> 1=(300, 240), 2=(300, 320)
        let rb1 = SchematicComponent::new(5, ComponentKind::Resistor, Pos2::new(300.0, 280.0), 3)
            .with_value("22k");
        // RB2: Base Bias Lower 4.7k at (300, 480) -> 1=(300, 440), 2=(300, 520)
        let rb2 = SchematicComponent::new(6, ComponentKind::Resistor, Pos2::new(300.0, 480.0), 4)
            .with_value("4.7k");
        // VIN: AC Signal Source at (180, 380) -> (+) at (180, 340), (-) at (180, 420)
        let vin = SchematicComponent::new(7, ComponentKind::AcVoltageSource, Pos2::new(180.0, 380.0), 1)
            .with_value("SIN(0 0.05 1k)");
        // GND: Reference Ground at (300, 560) -> pin at (300, 540)
        let gnd = SchematicComponent::new(8, ComponentKind::Ground, Pos2::new(300.0, 560.0), 1);

        self.components = vec![vcc, rc, q1, re, rb1, rb2, vin, gnd];
        self.next_comp_id = 9;

        // 2. Wires
        // Top VCC rail: VCC (+) (500, 240) -> RC pin 1 (420, 240) -> RB1 pin 1 (300, 240)
        let w1 = SchematicWire::manhattan_route(1, Pos2::new(500.0, 240.0), Pos2::new(420.0, 240.0));
        let w2 = SchematicWire::manhattan_route(2, Pos2::new(420.0, 240.0), Pos2::new(300.0, 240.0));

        // RC pin 2 (420, 320) to Q1 Collector (420, 340)
        let w3 = SchematicWire::manhattan_route(3, Pos2::new(420.0, 320.0), Pos2::new(420.0, 340.0));

        // Q1 Emitter (420, 420) to RE pin 1 (420, 440)
        let w4 = SchematicWire::manhattan_route(4, Pos2::new(420.0, 420.0), Pos2::new(420.0, 440.0));

        // Base network:
        // VIN (+) (180, 340) to Base tie point (300, 340)
        let w5 = SchematicWire::manhattan_route(5, Pos2::new(180.0, 340.0), Pos2::new(300.0, 340.0));
        // RB1 pin 2 (300, 320) to Base tie point (300, 340)
        let w6 = SchematicWire::manhattan_route(6, Pos2::new(300.0, 320.0), Pos2::new(300.0, 340.0));
        // Base tie point (300, 340) down to Q1 Base level (300, 380)
        let w7 = SchematicWire::manhattan_route(7, Pos2::new(300.0, 340.0), Pos2::new(300.0, 380.0));
        // Q1 Base (380, 380) to (300, 380)
        let w8 = SchematicWire::manhattan_route(8, Pos2::new(300.0, 380.0), Pos2::new(380.0, 380.0));
        // (300, 380) down to RB2 pin 1 (300, 440)
        let w9 = SchematicWire::manhattan_route(9, Pos2::new(300.0, 380.0), Pos2::new(300.0, 440.0));

        // Ground rail at y=540:
        // RB2 pin 2 (300, 520) down to GND pin (300, 540)
        let w10 = SchematicWire::manhattan_route(10, Pos2::new(300.0, 520.0), Pos2::new(300.0, 540.0));
        // RE pin 2 (420, 520) via VH to GND pin (300, 540)
        let w11 = SchematicWire::manhattan_route_vh(11, Pos2::new(420.0, 520.0), Pos2::new(300.0, 540.0));
        // VIN (-) (180, 420) via VH to GND pin (300, 540)
        let w12 = SchematicWire::manhattan_route_vh(12, Pos2::new(180.0, 420.0), Pos2::new(300.0, 540.0));
        // VCC (-) (500, 320) via VH to GND pin (300, 540)
        let w13 = SchematicWire::manhattan_route_vh(13, Pos2::new(500.0, 320.0), Pos2::new(300.0, 540.0));

        self.wires = vec![w1, w2, w3, w4, w5, w6, w7, w8, w9, w10, w11, w12, w13];
        self.next_wire_id = 14;

        self.sync_canvas_state();
        self.run_erc();
        self.sim_status.clear();
        self.pending_auto_center = true;
        self.project_title = "BJT CE Amplifier".to_string();
        self.current_project_path = None;
        self.modification_epoch = 0;
        self.clean_epoch = 0;
        self.is_modified = false;
        self.top_frame_config.circuit_name = self.project_title.clone();
        self.top_frame_config.is_modified = false;
    }

    /// Loads an interactive CMOS Inverter Pair demo circuit.
    pub fn load_cmos_inverter_demo(&mut self) {
        self.clear_canvas_state();
        self.history.clear();

        // 1. Components
        // VDD: 3.3V DC Supply
        let vdd = SchematicComponent::new(1, ComponentKind::VoltageSource, Pos2::new(200.0, 240.0), 1)
            .with_value("3.3");
        // M1: PMOS Pull-up at (360, 260) -> D=(380, 220), G=(340, 260), S=(380, 300)
        let mut m1 = SchematicComponent::new(2, ComponentKind::Pmos, Pos2::new(360.0, 260.0), 1);
        m1.name = "MP1".to_string();
        // M2: NMOS Pull-down at (360, 380) -> D=(380, 340), G=(340, 380), S=(380, 420)
        let mut m2 = SchematicComponent::new(3, ComponentKind::Nmos, Pos2::new(360.0, 380.0), 1);
        m2.name = "MN1".to_string();
        // VIN: Pulse Generator Input
        let vin = SchematicComponent::new(4, ComponentKind::PulseGenerator, Pos2::new(180.0, 380.0), 1)
            .with_value("PULSE(0 3.3 0 1n 1n 10u 20u)");
        // GND: Reference Ground
        let gnd = SchematicComponent::new(5, ComponentKind::Ground, Pos2::new(380.0, 480.0), 1);

        self.components = vec![vdd, m1, m2, vin, gnd];
        self.next_comp_id = 6;

        // 2. Wires
        // VDD (+) to PMOS Drain
        let w1 = SchematicWire::manhattan_route(1, Pos2::new(200.0, 200.0), Pos2::new(380.0, 220.0));
        // PMOS Source (380, 300) to NMOS Drain (380, 340) -> Output Node
        let w2 = SchematicWire::manhattan_route(2, Pos2::new(380.0, 300.0), Pos2::new(380.0, 340.0));
        // VIN (+) to Common Gates
        let w3 = SchematicWire::manhattan_route(3, Pos2::new(180.0, 340.0), Pos2::new(340.0, 260.0));
        let w4 = SchematicWire::manhattan_route(4, Pos2::new(340.0, 260.0), Pos2::new(340.0, 380.0));
        // NMOS Source to GND
        let w5 = SchematicWire::manhattan_route(5, Pos2::new(380.0, 420.0), Pos2::new(380.0, 460.0));
        // VDD (-) to GND
        let w6 = SchematicWire::manhattan_route(6, Pos2::new(200.0, 280.0), Pos2::new(380.0, 460.0));
        // VIN (-) to GND
        let w7 = SchematicWire::manhattan_route(7, Pos2::new(180.0, 420.0), Pos2::new(380.0, 460.0));

        self.wires = vec![w1, w2, w3, w4, w5, w6, w7];
        self.next_wire_id = 8;

        self.sync_canvas_state();
        self.run_erc();
        self.sim_status.clear();
        self.pending_auto_center = true;
        self.project_title = "CMOS Inverter".to_string();
        self.current_project_path = None;
        self.modification_epoch = 0;
        self.clean_epoch = 0;
        self.is_modified = false;
        self.top_frame_config.circuit_name = self.project_title.clone();
        self.top_frame_config.is_modified = false;
    }

    /// Loads an interactive NMOS Switch demo circuit.
    pub fn load_nmos_switch_demo(&mut self) {
        self.clear_canvas_state();
        self.history.clear();

        // 1. Components
        // VDD: 5V Supply
        let vdd = SchematicComponent::new(1, ComponentKind::VoltageSource, Pos2::new(180.0, 240.0), 1)
            .with_value("5.0");
        // RLOAD: 1k Load Resistor
        let rload = SchematicComponent::new(2, ComponentKind::Resistor, Pos2::new(380.0, 240.0), 1)
            .with_value("1k");
        // M1: NMOS Switch at (360, 360) -> D=(380, 320), G=(340, 360), S=(380, 400)
        let m1 = SchematicComponent::new(3, ComponentKind::Nmos, Pos2::new(360.0, 360.0), 1);
        // VGATE: Gate Pulse Generator
        let vgate = SchematicComponent::new(4, ComponentKind::PulseGenerator, Pos2::new(220.0, 360.0), 1)
            .with_value("PULSE(0 5 0 1n 1n 50u 100u)");
        // GND: Reference Ground
        let gnd = SchematicComponent::new(5, ComponentKind::Ground, Pos2::new(380.0, 460.0), 1);

        self.components = vec![vdd, rload, m1, vgate, gnd];
        self.next_comp_id = 6;

        // 2. Wires
        // VDD (+) to RLOAD pin 1
        let w1 = SchematicWire::manhattan_route(1, Pos2::new(180.0, 200.0), Pos2::new(380.0, 200.0));
        // RLOAD pin 2 to NMOS Drain (380, 320)
        let w2 = SchematicWire::manhattan_route(2, Pos2::new(380.0, 280.0), Pos2::new(380.0, 320.0));
        // VGATE (+) to NMOS Gate (340, 360)
        let w3 = SchematicWire::manhattan_route(3, Pos2::new(220.0, 320.0), Pos2::new(340.0, 360.0));
        // NMOS Source (380, 400) to GND
        let w4 = SchematicWire::manhattan_route(4, Pos2::new(380.0, 400.0), Pos2::new(380.0, 440.0));
        // VDD (-) to GND
        let w5 = SchematicWire::manhattan_route(5, Pos2::new(180.0, 280.0), Pos2::new(380.0, 440.0));
        // VGATE (-) to GND
        let w6 = SchematicWire::manhattan_route(6, Pos2::new(220.0, 400.0), Pos2::new(380.0, 440.0));

        self.wires = vec![w1, w2, w3, w4, w5, w6];
        self.next_wire_id = 7;

        self.sync_canvas_state();
        self.run_erc();
        self.sim_status.clear();
        self.pending_auto_center = true;
        self.project_title = "NMOS Switch".to_string();
        self.current_project_path = None;
        self.modification_epoch = 0;
        self.clean_epoch = 0;
        self.is_modified = false;
        self.top_frame_config.circuit_name = self.project_title.clone();
        self.top_frame_config.is_modified = false;
    }

    /// Compiles schematic and runs the non-linear DC Operating Point (.OP) solver.
    pub fn run_dc_op(&mut self) {
        match compile_schematic(&self.components, &self.wires) {
            Err(err) => {
                self.sim_status = format!("Compilation Error: {}", err);
            }
            Ok(compiled) => {
                self.spice_netlist_text = compiled.spice_netlist.clone();

                let newton_opts = NewtonOptions::default();
                match solve_dc_non_linear(&compiled.graph, &compiled.model_ctx, &newton_opts) {
                    Err(err) => {
                        self.sim_status = format!("DC Solver Error: {}", err);
                    }
                    Ok(sol) => {
                        self.dc_node_voltages.clear();
                        self.component_temperatures.clear();

                        // Map net voltages
                        for net in &compiled.net_names {
                            if let Some(node_id) = compiled.graph.get_node(net) {
                                if (node_id.0 as usize) < sol.node_voltages.len() {
                                    let v = sol.node_voltages[node_id.0 as usize];
                                    self.dc_node_voltages.insert(net.clone(), v);
                                }
                            }
                        }

                        // Estimate Joule heating and junction temperature for components
                        for comp in &self.components {
                            let pins = comp.all_pins();
                            if pins.len() >= 2 {
                                let n1 = compiled
                                    .pin_to_net
                                    .get(&(comp.name.clone(), pins[0].0.to_string()))
                                    .cloned()
                                    .unwrap_or_else(|| "0".to_string());
                                let n2 = compiled
                                    .pin_to_net
                                    .get(&(comp.name.clone(), pins[1].0.to_string()))
                                    .cloned()
                                    .unwrap_or_else(|| "0".to_string());

                                let v1 = *self.dc_node_voltages.get(&n1).unwrap_or(&0.0);
                                let v2 = *self.dc_node_voltages.get(&n2).unwrap_or(&0.0);
                                let vdrop = (v1 - v2).abs();

                                // Simple power model for thermal feedback visualization
                                let power_watts = match comp.kind {
                                    ComponentKind::Resistor => {
                                        let r = comp
                                            .value_str
                                            .trim_end_matches('k')
                                            .parse::<f64>()
                                            .unwrap_or(1.0)
                                            * 1e3;
                                        (vdrop * vdrop) / r.max(0.1)
                                    }
                                    ComponentKind::Diode => {
                                        // P = Vd * Id, Id ~ 1mA to 50mA
                                        vdrop * 0.015
                                    }
                                    ComponentKind::VoltageSource | ComponentKind::CurrentSource => {
                                        vdrop * 0.005
                                    }
                                    _ => 0.001,
                                };

                                let r_th = 60.0; // K/W
                                let t_ambient = 25.0;
                                let temp_c = t_ambient + power_watts * r_th;
                                self.component_temperatures
                                    .insert(comp.name.clone(), temp_c);
                            }
                        }

                        let (w_volts, w_currs) = compute_wire_telemetry(
                            &self.components,
                            &self.wires,
                            &compiled,
                            &self.dc_node_voltages,
                        );
                        self.wire_voltages = w_volts;
                        self.wire_currents = w_currs;

                        self.sim_status = format!(
                            "DC Solved: {} nodes, cond ratio = {:.2e}",
                            sol.node_voltages.len(),
                            sol.condition_ratio
                        );
                        self.compiled_circuit = Some(compiled);
                    }
                }
            }
        }
    }

    /// Generates multi-trace transient demo waveforms into the virtual oscilloscope.
    pub fn run_transient_demo(&mut self) {
        self.oscilloscope.clear();

        // 1. Input Sine Wave: 5.0 Vpp, 1 kHz
        let mut trace_vin = WaveformTrace::new("VIN (Input 1kHz)", Color32::from_rgb(80, 200, 255));
        let mut trace_vout =
            WaveformTrace::new("VOUT (Attenuated)", Color32::from_rgb(100, 255, 140));
        let mut trace_temp =
            WaveformTrace::new("Temp Junction (°C)", Color32::from_rgb(255, 120, 80));

        let num_points = 1200;
        let t_total = 0.003; // 3 ms
        let dt = t_total / num_points as f64;
        let freq = 1000.0;
        let omega = 2.0 * std::f64::consts::PI * freq;

        for i in 0..num_points {
            let t = i as f64 * dt;
            let v_in = 2.5 + 2.5 * (omega * t).sin();
            let v_out = 1.25 + 1.25 * (omega * t - 0.35).sin();
            let temp = 25.0 + 12.0 * (1.0 - (-t / 0.0008).exp()) + 1.5 * (omega * 2.0 * t).sin();

            trace_vin.push(t, v_in);
            trace_vout.push(t, v_out);
            trace_temp.push(t, temp);
        }

        self.multi_graph
            .route_simulation_traces(&[trace_vin.clone(), trace_vout.clone(), trace_temp.clone()]);
        self.oscilloscope = self.multi_graph.primary_scope.clone();

        self.sim_status = "Transient Solved: 3 traces, 3,600 samples".to_string();
    }

    /// Executes backward adjoint sensitivity analysis and worst-case optimization for the schematic circuit.
    pub fn run_sensitivity_analysis(&mut self) {
        match compile_schematic(&self.components, &self.wires) {
            Err(err) => {
                self.sim_status = format!("Compilation Error: {}", err);
            }
            Ok(compiled) => {
                let opts = phonon_solver::TransientOptions::default();
                self.sensitivity_dialog.run_analysis(
                    &compiled.graph,
                    &compiled.model_ctx,
                    &opts,
                );
                self.sensitivity_dialog.is_open = true;
                self.sim_status = self.sensitivity_dialog.status_msg.clone();
            }
        }
    }

    /// Executes distributed Monte Carlo & Latin Hypercube parameter sweep and yield analysis.
    pub fn run_monte_carlo_sweep(&mut self) {
        match compile_schematic(&self.components, &self.wires) {
            Err(err) => {
                self.sim_status = format!("Compilation Error: {}", err);
            }
            Ok(compiled) => {
                let opts = phonon_solver::TransientOptions::default();
                self.monte_carlo_dialog.run_sweep(
                    &compiled.graph,
                    &compiled.model_ctx,
                    &opts,
                );
                self.monte_carlo_dialog.is_open = true;
                self.sim_status = self.monte_carlo_dialog.status_msg.clone();
            }
        }
    }

    /// Runs dynamic electro-thermal co-simulation with thermal floorplan dialog.
    pub fn run_thermal_cosim(&mut self) {
        match compile_schematic(&self.components, &self.wires) {
            Err(err) => {
                self.sim_status = format!("Compilation Error: {}", err);
            }
            Ok(compiled) => {
                self.thermal_floorplan_dialog.auto_place_components(&compiled.graph);
                self.thermal_floorplan_dialog.run_simulation(&compiled.graph, &compiled.model_ctx);
                self.thermal_floorplan_dialog.is_open = true;
                self.sim_status = self.thermal_floorplan_dialog.status_msg.clone();
            }
        }
    }

    /// Checks whether a component with the given ID is selected.
    pub fn is_component_selected(&self, id: usize) -> bool {
        self.selected_component_ids.contains(&id) || self.selected_component_id == Some(id)
    }

    /// Checks whether a wire with the given ID is selected.
    pub fn is_wire_selected(&self, id: usize) -> bool {
        self.selected_wire_ids.contains(&id) || self.selected_wire_id == Some(id)
    }

    /// Clears all component and wire selections.
    pub fn clear_selection(&mut self) {
        self.selected_component_ids.clear();
        self.selected_wire_ids.clear();
        self.selected_component_id = None;
        self.selected_wire_id = None;
        self.sync_selection_to_canvas();
    }

    /// Synchronizes selection state from PhononApp to SchematicCanvas.
    pub fn sync_selection_to_canvas(&mut self) {
        self.canvas.selected_component_ids = self.selected_component_ids.clone();
        self.canvas.selected_wire_ids = self.selected_wire_ids.clone();
        self.canvas.selected_component_id = self.selected_component_id;
        self.canvas.selected_wire_id = self.selected_wire_id;
        self.canvas.marquee_start = self.marquee_start;
        self.canvas.marquee_current = self.marquee_current;
    }

    /// Selects a single component or adds to selection if multi is true.
    pub fn select_component(&mut self, id: usize, multi: bool) {
        if !multi {
            self.clear_selection();
        }
        self.selected_component_ids.insert(id);
        self.selected_component_id = Some(id);
        self.sync_selection_to_canvas();
    }

    /// Selects a single wire or adds to selection if multi is true.
    pub fn select_wire(&mut self, id: usize, multi: bool) {
        if !multi {
            self.clear_selection();
        }
        self.selected_wire_ids.insert(id);
        self.selected_wire_id = Some(id);
        self.sync_selection_to_canvas();
    }

    /// Toggles a component's selection state (for Shift+Click).
    pub fn toggle_component_selection(&mut self, id: usize) {
        if self.selected_component_ids.contains(&id) {
            self.selected_component_ids.remove(&id);
            if self.selected_component_id == Some(id) {
                self.selected_component_id = self.selected_component_ids.iter().next().copied();
            }
        } else {
            self.selected_component_ids.insert(id);
            self.selected_component_id = Some(id);
        }
        self.sync_selection_to_canvas();
    }

    /// Toggles a wire's selection state (for Shift+Click).
    pub fn toggle_wire_selection(&mut self, id: usize) {
        if self.selected_wire_ids.contains(&id) {
            self.selected_wire_ids.remove(&id);
            if self.selected_wire_id == Some(id) {
                self.selected_wire_id = self.selected_wire_ids.iter().next().copied();
            }
        } else {
            self.selected_wire_ids.insert(id);
            self.selected_wire_id = Some(id);
        }
        self.sync_selection_to_canvas();
    }

    /// Selects all components and wires in the current schematic sheet.
    pub fn select_all(&mut self) {
        self.selected_component_ids = self.components.iter().map(|c| c.id).collect();
        self.selected_wire_ids = self.wires.iter().map(|w| w.id).collect();
        self.selected_component_id = self.selected_component_ids.iter().next().copied();
        self.selected_wire_id = self.selected_wire_ids.iter().next().copied();
        self.sync_selection_to_canvas();
    }

    /// Selects all components and wires intersecting the given rectangle in world coordinates.
    pub fn select_in_rect(&mut self, rect: egui::Rect, add: bool) {
        if !add {
            self.clear_selection();
        }
        for comp in &self.components {
            if comp.intersects_rect(&rect) {
                self.selected_component_ids.insert(comp.id);
                self.selected_component_id = Some(comp.id);
            }
        }
        for wire in &self.wires {
            if wire.intersects_rect(&rect) {
                self.selected_wire_ids.insert(wire.id);
                self.selected_wire_id = Some(wire.id);
            }
        }
        self.sync_selection_to_canvas();
    }

    /// Translates all selected components and wires by delta in world coordinates.
    /// Also updates attached wire endpoints connected to moved components.
    pub fn translate_selection(&mut self, delta: Vec2) {
        if delta == Vec2::ZERO {
            return;
        }

        let mut moving_pins = Vec::new();
        for comp in &self.components {
            if self.is_component_selected(comp.id) {
                for (_, p) in comp.all_pins() {
                    moving_pins.push(p);
                }
            }
        }

        let sel_comp_ids = self.selected_component_ids.clone();
        let sel_comp_id = self.selected_component_id;
        let is_comp_sel = |id: usize| sel_comp_ids.contains(&id) || sel_comp_id == Some(id);

        for comp in &mut self.components {
            if is_comp_sel(comp.id) {
                comp.pos += delta;
            }
        }

        let sel_wire_ids = self.selected_wire_ids.clone();
        let sel_wire_id = self.selected_wire_id;
        let is_wire_sel = |id: usize| sel_wire_ids.contains(&id) || sel_wire_id == Some(id);

        for wire in &mut self.wires {
            if is_wire_sel(wire.id) {
                for seg in &mut wire.segments {
                    seg.start += delta;
                    seg.end += delta;
                }
            } else if !moving_pins.is_empty() {
                let start_attached = wire.segments.first().map_or(false, |s| {
                    moving_pins.iter().any(|&p| (p - s.start).length() <= 4.0)
                });
                let end_attached = wire.segments.last().map_or(false, |s| {
                    moving_pins.iter().any(|&p| (p - s.end).length() <= 4.0)
                });

                if start_attached && end_attached {
                    for seg in &mut wire.segments {
                        seg.start += delta;
                        seg.end += delta;
                    }
                } else if start_attached {
                    let old_end = wire.end_point();
                    let new_start = wire.start_point() + delta;
                    let was_vh = wire.segments.first().map_or(false, |s| {
                        (s.start.x - s.end.x).abs() < 1.0 && (s.start.y - s.end.y).abs() > 1.0
                    });
                    if was_vh {
                        *wire = SchematicWire::manhattan_route_vh_with_net(
                            wire.id,
                            new_start,
                            old_end,
                            wire.net_name.clone(),
                        );
                    } else {
                        *wire = SchematicWire::manhattan_route_hv_with_net(
                            wire.id,
                            new_start,
                            old_end,
                            wire.net_name.clone(),
                        );
                    }
                } else if end_attached {
                    let old_start = wire.start_point();
                    let new_end = wire.end_point() + delta;
                    let was_vh = wire.segments.last().map_or(false, |s| {
                        (s.start.x - s.end.x).abs() < 1.0 && (s.start.y - s.end.y).abs() > 1.0
                    });
                    if was_vh {
                        *wire = SchematicWire::manhattan_route_vh_with_net(
                            wire.id,
                            old_start,
                            new_end,
                            wire.net_name.clone(),
                        );
                    } else {
                        *wire = SchematicWire::manhattan_route_hv_with_net(
                            wire.id,
                            old_start,
                            new_end,
                            wire.net_name.clone(),
                        );
                    }
                }
            }
        }

        self.sync_selection_to_canvas();
    }

    /// Deletes all currently selected components and wires in a unified atomic undo/redo command.
    pub fn delete_selected(&mut self) {
        let mut target_comp_ids = self.selected_component_ids.clone();
        if let Some(cid) = self.selected_component_id {
            target_comp_ids.insert(cid);
        }

        let mut target_wire_ids = self.selected_wire_ids.clone();
        if let Some(wid) = self.selected_wire_id {
            target_wire_ids.insert(wid);
        }

        if target_comp_ids.is_empty() && target_wire_ids.is_empty() {
            return;
        }

        let mut to_delete_comps = Vec::new();
        let mut to_delete_wires = Vec::new();

        self.components.retain(|c| {
            if target_comp_ids.contains(&c.id) {
                to_delete_comps.push(c.clone());
                false
            } else {
                true
            }
        });

        self.wires.retain(|w| {
            if target_wire_ids.contains(&w.id) {
                to_delete_wires.push(w.clone());
                false
            } else {
                true
            }
        });

        let mut batch = Vec::new();
        for comp in to_delete_comps {
            batch.push(CanvasCommand::DeleteComponent(comp));
        }
        for wire in to_delete_wires {
            batch.push(CanvasCommand::DeleteWire(wire));
        }

        if !batch.is_empty() {
            if batch.len() == 1 {
                self.history.record(batch.remove(0));
            } else {
                self.history.record(CanvasCommand::Batch(batch));
            }
            self.mark_dirty();
        }

        self.clear_selection();
        self.sim_status.clear();
    }

    /// Duplicates all selected components and intra-selection wires, offset by (+40.0, +40.0) world coordinates.
    pub fn duplicate_selected(&mut self) {
        let mut target_comp_ids = self.selected_component_ids.clone();
        if let Some(cid) = self.selected_component_id {
            target_comp_ids.insert(cid);
        }

        let mut target_wire_ids = self.selected_wire_ids.clone();
        if let Some(wid) = self.selected_wire_id {
            target_wire_ids.insert(wid);
        }

        if target_comp_ids.is_empty() && target_wire_ids.is_empty() {
            return;
        }

        let offset = Vec2::new(40.0, 40.0);

        // 1. Duplicate components
        let mut new_comps = Vec::new();
        for comp in &self.components {
            if target_comp_ids.contains(&comp.id) {
                let new_id = self.next_comp_id;
                self.next_comp_id += 1;
                let count = self.components.iter().filter(|c| c.kind == comp.kind).count()
                    + 1
                    + new_comps.iter().filter(|c: &&SchematicComponent| c.kind == comp.kind).count();

                let mut cloned = comp.clone();
                cloned.id = new_id;
                cloned.pos += offset;
                cloned.name = format!("{}{}", comp.kind.prefix(), count);
                new_comps.push(cloned);
            }
        }

        // 2. Intra-selection wires
        let mut selected_pins = Vec::new();
        for comp in &self.components {
            if target_comp_ids.contains(&comp.id) {
                for (_, p) in comp.all_pins() {
                    selected_pins.push(p);
                }
            }
        }

        let mut new_wires = Vec::new();
        for wire in &self.wires {
            let explicitly_selected = target_wire_ids.contains(&wire.id);
            let intra_selection = if selected_pins.is_empty() {
                false
            } else {
                let start_connected = wire.segments.first().map_or(false, |s| {
                    selected_pins.iter().any(|&p| (p - s.start).length() <= 4.0)
                });
                let end_connected = wire.segments.last().map_or(false, |s| {
                    selected_pins.iter().any(|&p| (p - s.end).length() <= 4.0)
                });
                start_connected && end_connected
            };

            if explicitly_selected || intra_selection {
                let new_wid = self.next_wire_id;
                self.next_wire_id += 1;

                let mut cloned_wire = wire.clone();
                cloned_wire.id = new_wid;
                for seg in &mut cloned_wire.segments {
                    seg.start += offset;
                    seg.end += offset;
                }
                new_wires.push(cloned_wire);
            }
        }

        // 3. Atomically add items
        let mut batch = Vec::new();
        for comp in &new_comps {
            batch.push(CanvasCommand::AddComponent(comp.clone()));
            self.components.push(comp.clone());
        }
        for wire in &new_wires {
            batch.push(CanvasCommand::AddWire(wire.clone()));
            self.wires.push(wire.clone());
        }

        if !batch.is_empty() {
            if batch.len() == 1 {
                self.history.record(batch.remove(0));
            } else {
                self.history.record(CanvasCommand::Batch(batch));
            }
            self.mark_dirty();
        }

        // 4. Select newly duplicated items
        self.selected_component_ids = new_comps.iter().map(|c| c.id).collect();
        self.selected_wire_ids = new_wires.iter().map(|w| w.id).collect();
        self.selected_component_id = self.selected_component_ids.iter().next().copied();
        self.selected_wire_id = self.selected_wire_ids.iter().next().copied();
        self.sync_selection_to_canvas();

        self.sim_status = format!(
            "Duplicated {} components, {} wires",
            new_comps.len(),
            new_wires.len()
        );
    }

    /// Rotates the active component(s) clockwise by 90 degrees.
    pub fn rotate_active(&mut self) {
        let mut target_comp_ids = self.selected_component_ids.clone();
        if let Some(cid) = self.selected_component_id {
            target_comp_ids.insert(cid);
        }

        if !target_comp_ids.is_empty() {
            let mut batch = Vec::new();
            for comp in &mut self.components {
                if target_comp_ids.contains(&comp.id) {
                    let from_rot = comp.rotation;
                    comp.rotate_clockwise();
                    let to_rot = comp.rotation;
                    batch.push(CanvasCommand::RotateComponent {
                        id: comp.id,
                        from_rot,
                        to_rot,
                    });
                }
            }

            if !batch.is_empty() {
                if batch.len() == 1 {
                    self.history.record(batch.remove(0));
                } else {
                    self.history.record(CanvasCommand::Batch(batch));
                }
                self.mark_dirty();
            }
            self.sync_selection_to_canvas();
            self.sim_status.clear();
            return;
        }

        if self.selected_tool.is_place() {
            self.placement_rotation = (self.placement_rotation + 1) % 4;
            self.sim_status.clear();
        }
    }

    /// Rotates the selected component(s) clockwise by 90 degrees.
    pub fn rotate_selected(&mut self) {
        self.rotate_active();
    }

    /// Executes a registered CAD action by identifier.
    pub fn execute_action(&mut self, action_id: ActionId) {
        match action_id {
            ActionId::NewProject => self.request_action(PendingAction::NewProject),
            ActionId::OpenProject => self.open_open_dialog(),
            ActionId::SaveProject => {
                let _ = self.save_project();
            }
            ActionId::SaveProjectAs => self.open_save_as_dialog(),
            ActionId::ClearCanvas => self.request_action(PendingAction::ClearCanvas),
            ActionId::CloseApp => self.request_action(PendingAction::CloseApp),
            ActionId::ExportNetlist => {
                self.show_netlist_window = true;
                self.spice_netlist_text = self.netlist_sync.sync_from_canvas(&self.canvas).to_string();
            }
            ActionId::Undo => {
                self.undo();
            }
            ActionId::Redo => {
                self.redo();
            }
            ActionId::Delete => self.delete_selected(),
            ActionId::Duplicate => self.duplicate_selected(),
            ActionId::SelectAll => self.select_all(),
            ActionId::ClearSelection => self.clear_selection(),
            ActionId::RotateClockwise => self.rotate_active(),
            ActionId::ToggleFloatingToolbar => {
                self.show_floating_toolbar = !self.show_floating_toolbar;
                self.floating_toolbar_state.is_visible = self.show_floating_toolbar;
            }
            ActionId::ToggleGrid => self.canvas.show_grid = !self.canvas.show_grid,
            ActionId::ZoomFit => self.pending_auto_center = true,
            ActionId::TogglePalette => self.show_palette = !self.show_palette,
            ActionId::ToggleOscilloscope => self.show_oscilloscope = !self.show_oscilloscope,
            ActionId::ToggleThermal => self.show_thermal_overlay = !self.show_thermal_overlay,
            ActionId::ToggleErcOverlay => self.show_erc_overlay = !self.show_erc_overlay,
            ActionId::RunSimulation => self.run_dc_op(),
            ActionId::RunErc => self.run_erc(),
            ActionId::ToolSelect => {
                self.selected_tool = ToolMode::Select;
                self.active_wire_start = None;
            }
            ActionId::ToolWire => {
                self.selected_tool = ToolMode::Wire;
                self.active_wire_start = None;
            }
            ActionId::ToolBus => {
                self.selected_tool = ToolMode::Bus;
                self.active_wire_start = None;
            }
            ActionId::ToolProbe => {
                self.selected_tool = ToolMode::Probe;
                self.active_wire_start = None;
            }
            ActionId::ClearWire => self.active_wire_start = None,
            ActionId::OpenCommandPalette => self.command_palette.open(),
            ActionId::OpenPreferences => self.preferences_dialog.is_open = true,
        }
    }

    /// Handles global hotkeys and keyboard shortcuts using a 3-tier priority hierarchy.
    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let ctrl = ctx.input(|i| i.modifiers.command || i.modifiers.ctrl);
        let shift = ctx.input(|i| i.modifiers.shift);

        if ctrl && ctx.input(|i| i.key_pressed(egui::Key::Comma)) {
            self.preferences_dialog.is_open = !self.preferences_dialog.is_open;
        }

        // =========================================================================
        // TIER 1: System Reserved Keys (Protected - always active)
        // =========================================================================

        // Command Palette: Ctrl+K
        if ctrl && ctx.input(|i| i.key_pressed(Key::K)) {
            self.command_palette.toggle();
            return;
        }

        // New Project: Ctrl+N
        if ctrl && !shift && ctx.input(|i| i.key_pressed(Key::N)) {
            self.request_action(PendingAction::NewProject);
            return;
        }

        // Open Project: Ctrl+O
        if ctrl && !shift && ctx.input(|i| i.key_pressed(Key::O)) {
            self.open_open_dialog();
            return;
        }

        // Save Project: Ctrl+S
        if ctrl && !shift && ctx.input(|i| i.key_pressed(Key::S)) {
            let _ = self.save_project();
            return;
        }

        // Save Project As: Ctrl+Shift+S
        if ctrl && shift && ctx.input(|i| i.key_pressed(Key::S)) {
            self.open_save_as_dialog();
            return;
        }

        // Undo: Ctrl+Z
        if ctrl && !shift && ctx.input(|i| i.key_pressed(Key::Z)) {
            self.undo();
            return;
        }

        // Redo: Ctrl+Y or Ctrl+Shift+Z
        if (ctrl && ctx.input(|i| i.key_pressed(Key::Y)))
            || (ctrl && shift && ctx.input(|i| i.key_pressed(Key::Z)))
        {
            self.redo();
            return;
        }

        // Escape: Cancel active wire, cancel placement, close palette, clear selection
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            if self.command_palette.is_open {
                self.command_palette.close();
            } else {
                self.active_wire_start = None;
                self.selected_tool = ToolMode::Select;
                self.clear_selection();
                self.placement_rotation = 0;
            }
            return;
        }

        // =========================================================================
        // TIER 2: Text Input Focus Masking
        // =========================================================================
        // If user is actively typing in a search bar, rename field, or dialog input,
        // CAD hotkeys MUST be strictly masked to prevent unintended tool switches or rotations.
        if ctx.egui_wants_keyboard_input() {
            return;
        }

        // =========================================================================
        // TIER 3: CAD Hotkeys and Tool Modifiers
        // =========================================================================

        // Duplicate: Ctrl+D
        if ctrl && ctx.input(|i| i.key_pressed(Key::D)) {
            self.duplicate_selected();
            return;
        }

        // Select All: Ctrl+A
        if ctrl && ctx.input(|i| i.key_pressed(Key::A)) {
            self.select_all();
            return;
        }

        // Delete: Delete or Backspace
        if ctx.input(|i| i.key_pressed(Key::Delete) || i.key_pressed(Key::Backspace)) {
            self.delete_selected();
            return;
        }

        // Rotate: R
        if !ctrl && ctx.input(|i| i.key_pressed(Key::R)) {
            self.rotate_active();
            return;
        }

        // Toggle Floating CAD Toolbar: H
        if !ctrl && ctx.input(|i| i.key_pressed(Key::H)) {
            self.show_floating_toolbar = !self.show_floating_toolbar;
            self.floating_toolbar_state.is_visible = self.show_floating_toolbar;
            return;
        }

        // Zoom to Fit: F
        if !ctrl && ctx.input(|i| i.key_pressed(Key::F)) {
            self.pending_auto_center = true;
            return;
        }

        // Toggle Grid: G
        if !ctrl && ctx.input(|i| i.key_pressed(Key::G)) {
            self.canvas.show_grid = !self.canvas.show_grid;
            return;
        }

        // Select Tool: V or S
        if !ctrl && (ctx.input(|i| i.key_pressed(Key::V)) || ctx.input(|i| i.key_pressed(Key::S))) {
            self.selected_tool = ToolMode::Select;
            self.active_wire_start = None;
            self.placement_rotation = 0;
            return;
        }

        // Wire Tool: W
        if !ctrl && ctx.input(|i| i.key_pressed(Key::W)) {
            self.selected_tool = ToolMode::Wire;
            self.active_wire_start = None;
            self.placement_rotation = 0;
            return;
        }

        // Bus Tool: B
        if !ctrl && ctx.input(|i| i.key_pressed(Key::B)) {
            self.selected_tool = ToolMode::Bus;
            self.active_wire_start = None;
            self.placement_rotation = 0;
            return;
        }

        // Probe Tool: P
        if !ctrl && ctx.input(|i| i.key_pressed(Key::P)) {
            self.selected_tool = ToolMode::Probe;
            self.active_wire_start = None;
            self.placement_rotation = 0;
            return;
        }
    }

    /// Interactive canvas response and rendering.
    fn render_canvas(&mut self, ui: &mut egui::Ui) {
        let (response, painter) =
            ui.allocate_painter(ui.available_size_before_wrap(), Sense::click_and_drag());
        let viewport = response.rect;
        self.last_canvas_rect = viewport;

        if self.pending_auto_center {
            self.center_on_bounding_box(viewport);
            self.pending_auto_center = false;
        }

        // 1. Pan & Zoom
        self.canvas.handle_pan_zoom(ui, &response);

        let theme = self.preferences.current_theme();

        // 2. Render background grid
        self.canvas.render_grid_themed(&painter, viewport, &theme);

        // Collect all pin positions for snapping and junction rendering
        let mut all_pin_positions = Vec::new();
        for comp in &self.components {
            for (_, p) in comp.all_pins() {
                all_pin_positions.push(p);
            }
        }

        self.sync_selection_to_canvas();

        // 3. Render wires
        for wire in &self.wires {
            let is_sel = self.is_wire_selected(wire.id);
            wire.render_with_theme(&painter, &self.canvas, is_sel, &theme);
        }

        // 3b. Render buses
        for bus in &self.buses {
            bus.render(&painter, &self.canvas, false);
        }

        // 3c. Render subcircuit instances
        for inst in &self.canvas.subcircuit_instances {
            inst.render(&painter, &self.canvas, self.subcircuits.get(&inst.def_name), false);
        }

        // 4. Render junction dots
        let junctions = compute_junction_dots(&self.wires, &all_pin_positions);
        let junction_stroke = Stroke::new(1.0, Color32::from_rgb(180, 255, 180));
        for j_world in &junctions {
            let j_screen = self.canvas.world_to_screen(*j_world);
            painter.circle_filled(
                j_screen,
                4.0 * self.canvas.zoom.clamp(0.8, 1.6),
                Color32::from_rgb(60, 200, 80),
            );
            painter.circle_stroke(
                j_screen,
                4.0 * self.canvas.zoom.clamp(0.8, 1.6),
                junction_stroke,
            );
        }

        // 4b. Render illuminated selection halos
        self.canvas.render_selection_halos(&painter);

        // 4c. Render rubberband marquee drag box
        self.canvas.render_marquee(&painter);

        // 5. Mouse interactions on canvas
        let mouse_pos = ui.input(|i| i.pointer.hover_pos());
        if let Some(mouse_screen) = mouse_pos {
            let mouse_world = self.canvas.screen_to_world(mouse_screen);
            let snapped_world = self.canvas.snap_to_grid(mouse_world);

            // Wire hover telemetry badge (V and I readouts)
            if self.selected_tool == ToolMode::Select || self.selected_tool == ToolMode::Probe {
                let hover_tol = 8.0 / self.canvas.zoom;
                if let Some(hovered_wire) = self.wires.iter().find(|w| w.contains(mouse_world, hover_tol)) {
                    let v = self.wire_voltages.get(&hovered_wire.id).copied().unwrap_or(0.0);
                    let i = self.wire_currents.get(&hovered_wire.id).copied().unwrap_or(0.0);
                    let v_str = crate::oscilloscope::format_voltage_si(v);
                    let i_str = crate::oscilloscope::format_current_si(i);
                    crate::widgets::render_dual_telemetry_pill(
                        &painter,
                        mouse_screen + Vec2::new(20.0, -20.0),
                        "V",
                        &v_str,
                        "I",
                        &i_str,
                        &crate::widgets::PillBadgeStyle::wire_telemetry(),
                        self.canvas.zoom,
                    );
                }
            }

            // Handle tool actions on click
            if response.clicked_by(PointerButton::Primary) {
                let shift = ui.input(|i| i.modifiers.shift);
                match &self.selected_tool {
                    ToolMode::Select => {
                        // Hit-test components
                        if let Some(comp) = self
                            .components
                            .iter()
                            .rev()
                            .find(|c| c.contains(mouse_world))
                        {
                            let cid = comp.id;
                            if shift {
                                self.toggle_component_selection(cid);
                            } else {
                                self.select_component(cid, false);
                            }
                        } else if let Some(wire) = self
                            .wires
                            .iter()
                            .rev()
                            .find(|w| w.contains(mouse_world, 6.0))
                        {
                            let wid = wire.id;
                            if shift {
                                self.toggle_wire_selection(wid);
                            } else {
                                self.select_wire(wid, false);
                            }
                        } else if !shift {
                            self.clear_selection();
                        }
                    }
                    ToolMode::Place(kind) | ToolMode::PlaceComponent(kind) => {
                        let count = self.components.iter().filter(|c| c.kind == *kind).count() + 1;
                        let mut new_comp = SchematicComponent::new(
                            self.next_comp_id,
                            kind.clone(),
                            snapped_world,
                            count,
                        );
                        new_comp.rotation = self.placement_rotation;
                        self.components.push(new_comp.clone());
                        self.next_comp_id += 1;
                        self.sim_status = format!("Placed {}", kind.prefix());
                        self.history.record(CanvasCommand::AddComponent(new_comp));
                        self.mark_dirty();
                    }
                    ToolMode::Wire => {
                        if let Some(start) = self.active_wire_start {
                            // Finish wire
                            if (start - snapped_world).length() > 5.0 {
                                let wire = SchematicWire::manhattan_route(
                                    self.next_wire_id,
                                    start,
                                    snapped_world,
                                );
                                self.wires.push(wire.clone());
                                self.next_wire_id += 1;
                                self.history.record(CanvasCommand::AddWire(wire));
                                self.mark_dirty();
                                self.active_wire_start = Some(snapped_world); // Continue routing from current point
                            }
                        } else {
                            // Find nearest pin to snap start
                            let nearest_pin = all_pin_positions
                                .iter()
                                .find(|&&p| (p - mouse_world).length() <= 12.0)
                                .copied();
                            self.active_wire_start = Some(nearest_pin.unwrap_or(snapped_world));
                        }
                    }
                    ToolMode::Bus => {
                        if let Some(start) = self.active_wire_start {
                            if (start - snapped_world).length() > 5.0 {
                                let mut segs = Vec::new();
                                segs.push(crate::schematic::wire::WireSegment::new(start, snapped_world));
                                let bus = crate::schematic::bus::SchematicBus::new(
                                    self.buses.len() + 1,
                                    crate::schematic::bus::BusSignal::new("DATA", 7, 0),
                                    segs,
                                );
                                self.buses.push(bus);
                                self.active_wire_start = Some(snapped_world);
                            }
                        } else {
                            self.active_wire_start = Some(snapped_world);
                        }
                    }
                    ToolMode::Probe => {
                        // Check if user clicked a pin
                        let mut probed = false;
                        for comp in &self.components {
                            for (pin_name, p) in comp.all_pins() {
                                if (p - mouse_world).length() <= 15.0 {
                                    if let Some(compiled) = &self.compiled_circuit {
                                        if let Some(net) = compiled
                                            .pin_to_net
                                            .get(&(comp.name.clone(), pin_name.to_string()))
                                        {
                                            if let Some(&volts) = self.dc_node_voltages.get(net) {
                                                self.sim_status =
                                                    format!("Probed Node {}: {:.3} V", net, volts);
                                                probed = true;
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                            if probed {
                                break;
                            }
                        }
                    }
                }
            }

            // Cancel wire on secondary click
            if response.clicked_by(PointerButton::Secondary) {
                self.active_wire_start = None;
            }

            // Drag handling for selection moving or rubberband marquee
            if response.drag_started_by(PointerButton::Primary)
                && self.selected_tool == ToolMode::Select
            {
                let shift = ui.input(|i| i.modifiers.shift);
                if let Some(comp) = self.components.iter().rev().find(|c| c.contains(mouse_world)) {
                    let comp_id = comp.id;
                    if shift {
                        self.toggle_component_selection(comp_id);
                    } else {
                        if !self.is_component_selected(comp_id) {
                            self.select_component(comp_id, false);
                        }
                        self.dragging_selection = true;
                        self.drag_start_positions = self
                            .components
                            .iter()
                            .filter(|c| self.is_component_selected(c.id))
                            .map(|c| (c.id, c.pos))
                            .collect();
                        self.drag_start_wires = self.wires.clone();
                    }
                } else if let Some(wire) = self.wires.iter().rev().find(|w| w.contains(mouse_world, 6.0)) {
                    let wire_id = wire.id;
                    if shift {
                        self.toggle_wire_selection(wire_id);
                    } else {
                        self.select_wire(wire_id, false);
                    }
                } else {
                    // Empty canvas: Start rubberband marquee box
                    self.marquee_start = Some(mouse_world);
                    self.marquee_current = Some(mouse_world);
                    self.sync_selection_to_canvas();
                }
            }

            if response.dragged_by(PointerButton::Primary) && self.selected_tool == ToolMode::Select {
                if self.dragging_selection {
                    let delta = response.drag_delta() / self.canvas.zoom;
                    self.translate_selection(delta);
                } else if self.marquee_start.is_some() {
                    self.marquee_current = Some(mouse_world);
                    self.sync_selection_to_canvas();
                }
            }

            if response.drag_stopped() && self.selected_tool == ToolMode::Select {
                if self.dragging_selection {
                    let mut batch = Vec::new();
                    for (id, start_pos) in self.drag_start_positions.drain(..) {
                        if let Some(comp) = self.components.iter_mut().find(|c| c.id == id) {
                            comp.pos = self.canvas.snap_to_grid(comp.pos);
                            if comp.pos != start_pos {
                                batch.push(CanvasCommand::MoveComponent {
                                    id,
                                    from: start_pos,
                                    to: comp.pos,
                                });
                            }
                        }
                    }

                    // Compare wire states before and after drag to record wire adjustments in history
                    for start_wire in self.drag_start_wires.drain(..) {
                        if let Some(curr_wire) = self.wires.iter().find(|w| w.id == start_wire.id) {
                            if curr_wire.segments != start_wire.segments {
                                batch.push(CanvasCommand::DeleteWire(start_wire));
                                batch.push(CanvasCommand::AddWire(curr_wire.clone()));
                            }
                        }
                    }

                    if !batch.is_empty() {
                        if batch.len() == 1 {
                            self.history.record(batch.remove(0));
                        } else {
                            self.history.record(CanvasCommand::Batch(batch));
                        }
                        self.mark_dirty();
                    }
                    self.dragging_selection = false;
                    self.sync_selection_to_canvas();
                } else if let (Some(m_start), Some(m_curr)) = (self.marquee_start.take(), self.marquee_current.take()) {
                    let marquee_rect = Rect::from_two_pos(m_start, m_curr);
                    if marquee_rect.width() > 3.0 || marquee_rect.height() > 3.0 {
                        let shift = ui.input(|i| i.modifiers.shift);
                        self.select_in_rect(marquee_rect, shift);
                    }
                    self.sync_selection_to_canvas();
                }
            }

            // Render wire routing rubberband preview
            if let Some(start) = self.active_wire_start {
                let preview_wire = SchematicWire::manhattan_route(0, start, snapped_world);
                let preview_stroke = Stroke::new(2.0, Color32::from_rgb(120, 240, 160));
                for seg in &preview_wire.segments {
                    let s_screen = self.canvas.world_to_screen(seg.start);
                    let e_screen = self.canvas.world_to_screen(seg.end);
                    painter.line_segment([s_screen, e_screen], preview_stroke);
                }
            }

            // Render component placement ghost preview
            if let Some(kind) = self.selected_tool.place_kind() {
                let mut ghost = SchematicComponent::new(0, kind.clone(), snapped_world, 0);
                ghost.rotation = self.placement_rotation;
                ghost.render_with_theme(&painter, &self.canvas, true, None, &theme);
            }
        }

        // 6. Render components and thermal overlay
        for comp in &self.components {
            let is_sel = self.is_component_selected(comp.id);

            // Collect pin voltages for this component if available
            let mut pin_voltages = Vec::new();
            if let Some(compiled) = &self.compiled_circuit {
                for (pin_name, _) in comp.all_pins() {
                    if let Some(net) = compiled
                        .pin_to_net
                        .get(&(comp.name.clone(), pin_name.to_string()))
                    {
                        if let Some(&volts) = self.dc_node_voltages.get(net) {
                            pin_voltages.push((pin_name, volts));
                        }
                    }
                }
            }

            comp.render_with_theme(
                &painter,
                &self.canvas,
                is_sel,
                if pin_voltages.is_empty() {
                    None
                } else {
                    Some(&pin_voltages)
                },
                &theme,
            );

            // Thermal badge
            if self.show_thermal_overlay {
                if let Some(&temp_c) = self.component_temperatures.get(&comp.name) {
                    let badge_pos = self
                        .canvas
                        .world_to_screen(comp.pos + Vec2::new(-35.0, 48.0));
                    self.thermal
                        .render_junction_badge_scaled(&painter, badge_pos, temp_c, &comp.name, self.canvas.zoom);
                }
            }

            // Sensitivity pin badges
            if self.sensitivity_dialog.show_badges_on_canvas {
                if let Some((_norm_sens, badge_color)) =
                    self.sensitivity_dialog.get_badge_impact(&comp.name)
                {
                    for (_pin_name, pin_world) in comp.all_pins() {
                        let pin_screen = self.canvas.world_to_screen(pin_world);
                        painter.circle_filled(
                            pin_screen,
                            4.5 * self.canvas.zoom.clamp(0.8, 1.5),
                            badge_color,
                        );
                        painter.circle_stroke(
                            pin_screen,
                            6.5 * self.canvas.zoom.clamp(0.8, 1.5),
                            Stroke::new(1.5, badge_color),
                        );
                    }
                }
            }
        }

        // 7. Visual ERC Diagnostic Overlay
        self.canvas.draw(
            &painter,
            &self.erc_diagnostics,
            self.show_erc_overlay,
            mouse_pos,
        );

        // 8. Floating CAD Tools Island (rendered in Order::Middle)
        let toolbar_action = self.floating_toolbar_state.show(
            ui.ctx(),
            viewport,
            &self.selected_tool,
        );
        if let Some(act) = toolbar_action {
            match act {
                FloatingToolbarAction::SelectTool(tool) => {
                    self.selected_tool = tool;
                    self.active_wire_start = None;
                }
                FloatingToolbarAction::Rotate => self.rotate_active(),
                FloatingToolbarAction::Delete => self.delete_selected(),
                FloatingToolbarAction::Clear => {
                    self.active_wire_start = None;
                    self.clear_selection();
                }
            }
        }

        // 9. Command Palette Modal (rendered in Order::Foreground)
        let palette_action = self.command_palette.show(ui.ctx(), &self.action_registry);
        if let Some(action_id) = palette_action {
            self.execute_action(action_id);
        }
    }

    /// Renders the left component palette and project hierarchy panel.
    fn render_palette(&mut self, ui: &mut egui::Ui) {
        let current_place_kind = self.selected_tool.place_kind();
        let subcircuits = self.subcircuit_registry.list();
        let action = self.palette.render(
            ui,
            &self.top_frame_config.circuit_name,
            &self.components,
            &self.wires,
            &subcircuits,
            &self.dc_node_voltages,
            current_place_kind,
            self.selected_component_id,
        );

        if let Some(act) = action {
            match act {
                PaletteAction::SelectKind(kind) => {
                    self.selected_tool = ToolMode::PlaceComponent(kind);
                    self.placement_rotation = 0;
                    self.active_wire_start = None;
                    self.clear_selection();
                }
                PaletteAction::SelectSubcircuit(pkg_name) => {
                    self.sim_status = format!("Selected subcircuit package: {}", pkg_name);
                }
                PaletteAction::SelectComponent(cid) => {
                    self.select_component(cid, false);
                }
                PaletteAction::FocusComponent(cid) => {
                    if let Some(comp) = self.components.iter().find(|c| c.id == cid) {
                        let vp_center = self.last_canvas_rect.center();
                        self.canvas.pan = vp_center.to_vec2() - comp.pos.to_vec2() * self.canvas.zoom;
                        self.select_component(cid, false);
                    }
                }
                PaletteAction::SelectNet(net) => {
                    self.clear_selection();
                    for wire in &self.wires {
                        if wire.net_name.as_deref() == Some(&net) {
                            self.canvas.selected_wire_ids.insert(wire.id);
                            self.selected_wire_id = Some(wire.id);
                        }
                    }
                }
                PaletteAction::OpenPackageDialog => {
                    let sel_comps: Vec<_> = self
                        .components
                        .iter()
                        .filter(|c| self.is_component_selected(c.id))
                        .cloned()
                        .collect();
                    let sel_wires: Vec<_> = self
                        .wires
                        .iter()
                        .filter(|w| self.canvas.is_wire_selected(w.id))
                        .cloned()
                        .collect();
                    let (comps_to_pkg, wires_to_pkg) = if !sel_comps.is_empty() {
                        (sel_comps, sel_wires)
                    } else {
                        (self.components.clone(), self.wires.clone())
                    };
                    self.subcircuit_dialog
                        .open_with_selection(comps_to_pkg, wires_to_pkg);
                }
            }
        }
    }

    /// Renders the right inspector panel for selected components and simulation parameters.
    fn render_inspector(&mut self, ui: &mut egui::Ui) {
        ui.heading("Inspector");

        if let Some(cid) = self.selected_component_id {
            let mut do_rotate = false;
            let mut do_delete = false;
            let mut record_cmd = None;
            let mut comp_name_for_telemetry = String::new();
            let mut comp_pins_for_telemetry = Vec::new();

            if let Some(comp) = self.components.iter_mut().find(|c| c.id == cid) {
                comp_name_for_telemetry = comp.name.clone();
                comp_pins_for_telemetry = comp.all_pins();

                // 1. Prominent Header: Component Name and Kind
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(&comp.name)
                            .strong()
                            .size(16.0)
                            .color(Color32::from_rgb(100, 200, 255)),
                    );
                    ui.label(
                        RichText::new(format!("({})", comp.kind.display_name()))
                            .size(12.0)
                            .color(Color32::from_rgb(148, 163, 184)),
                    );
                });
                ui.separator();

                // 2. Unboxed Properties: ID, Name, Value, Rotation
                ui.label(format!("Component ID: {}", comp.id));
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut comp.name);
                });
                ui.horizontal(|ui| {
                    ui.label("Value:");
                    if self.editing_comp_value.as_ref().map(|(id, _)| *id) != Some(comp.id) {
                        self.editing_comp_value = Some((comp.id, comp.value_str.clone()));
                    }
                    let val_resp = ui.text_edit_singleline(&mut comp.value_str);
                    if val_resp.lost_focus() {
                        if let Some((_, old_val)) = self.editing_comp_value.take() {
                            if old_val != comp.value_str {
                                record_cmd = Some(CanvasCommand::ModifyComponentValue {
                                    id: comp.id,
                                    old_val,
                                    new_val: comp.value_str.clone(),
                                });
                            }
                        }
                    }
                });
                ui.label(format!("Rotation: {} deg", (comp.rotation % 4) * 90));

                ui.horizontal(|ui| {
                    if ui.button("Rotate 90 deg (R)").clicked() {
                        do_rotate = true;
                    }
                });

                // 3. Label Position Controls (Presets to prevent collision with shapes)
                ui.separator();
                ui.label("Label Position Presets:");
                ui.horizontal(|ui| {
                    if ui.button("Right").clicked() {
                        comp.set_label_offset(Vec2::new(28.0, -10.0));
                        comp.set_value_offset(Vec2::new(28.0, 8.0));
                    }
                    if ui.button("Left").clicked() {
                        comp.set_label_offset(Vec2::new(-60.0, -10.0));
                        comp.set_value_offset(Vec2::new(-60.0, 8.0));
                    }
                    if ui.button("Top").clicked() {
                        comp.set_label_offset(Vec2::new(-16.0, -42.0));
                        comp.set_value_offset(Vec2::new(-16.0, -28.0));
                    }
                    if ui.button("Bottom").clicked() {
                        comp.set_label_offset(Vec2::new(-16.0, 28.0));
                        comp.set_value_offset(Vec2::new(-16.0, 42.0));
                    }
                });

                // 4. Pin Terminals
                ui.separator();
                ui.label("Pin Terminals:");
                for (pin_name, p_world) in comp.all_pins() {
                    ui.monospace(format!(
                        "Pin {}: ({:.0}, {:.0})",
                        pin_name, p_world.x, p_world.y
                    ));
                }

                ui.separator();
                if ui.button("Delete Component").clicked() {
                    do_delete = true;
                }
            }

            // 5. Per-component Operating Telemetry (DC node voltages & temperature)
            if !comp_name_for_telemetry.is_empty() {
                ui.separator();
                ui.heading("Operating Telemetry");

                if let Some(&temp_c) = self.component_temperatures.get(&comp_name_for_telemetry) {
                    ui.monospace(format!("Junction Temp: {:.1} °C", temp_c));
                }

                if let Some(compiled) = &self.compiled_circuit {
                    let mut found_pins = false;
                    for (pin_name, _) in &comp_pins_for_telemetry {
                        if let Some(net) = compiled.pin_to_net.get(&(comp_name_for_telemetry.clone(), pin_name.to_string())) {
                            if let Some(&volts) = self.dc_node_voltages.get(net) {
                                ui.monospace(format!("Pin {} (net {}): {:.4} V", pin_name, net, volts));
                                found_pins = true;
                            }
                        }
                    }
                    if !found_pins {
                        ui.label("No active DC voltages for pins");
                    }
                } else if !self.dc_node_voltages.is_empty() {
                    ui.label("Run DC (.OP) to view terminal voltages");
                }
            }

            if let Some(cmd) = record_cmd {
                self.history.record(cmd);
            }
            if do_rotate {
                self.rotate_active();
            }
            if do_delete {
                self.delete_selected();
            }
        } else if let Some(wid) = self.selected_wire_id {
            let mut do_delete_wire = false;
            if let Some(wire) = self.wires.iter().find(|w| w.id == wid) {
                ui.label(RichText::new(format!("Wire #{} Telemetry", wire.id)).strong());
                ui.separator();
                let net = self
                    .compiled_circuit
                    .as_ref()
                    .and_then(|c| c.wire_to_net.get(&wire.id))
                    .cloned()
                    .or_else(|| wire.net_name.clone())
                    .unwrap_or_else(|| "Unassigned".to_string());
                ui.monospace(format!("Net: {}", net));
                ui.label(format!("Segments: {}", wire.segments.len()));
                let total_len: f32 = wire.segments.iter().map(|s| s.length()).sum();
                ui.label(format!("Total Length: {:.1} px ({:.2} mm)", total_len, total_len * 0.254));

                if let Some(&v) = self.wire_voltages.get(&wire.id) {
                    ui.monospace(format!("DC Voltage: {}", crate::oscilloscope::format_voltage_si(v)));
                }
                if let Some(&i) = self.wire_currents.get(&wire.id) {
                    ui.monospace(format!("Branch Current: {}", crate::oscilloscope::format_current_si(i)));
                    if let Some(&v) = self.wire_voltages.get(&wire.id) {
                        let power = (v * i).abs();
                        ui.label(format!("Carried Power: {:.3} mW", power * 1.0e3));
                    }
                }

                ui.add_space(6.0);
                if ui.button("Delete Wire").clicked() {
                    do_delete_wire = true;
                }
            }
            if do_delete_wire {
                self.delete_selected();
            }
        } else if let Some(prefix) = self.selected_tool.place_kind().map(|k| k.prefix()) {
            let rot_deg = (self.placement_rotation % 4) * 90;
            let mut do_rotate = false;
            ui.label(format!("Placing: {}", prefix));
            ui.label(format!("Rotation: {} deg", rot_deg));
            ui.horizontal(|ui| {
                if ui.button("Rotate 90 deg (R)").clicked() {
                    do_rotate = true;
                }
            });
            if do_rotate {
                self.rotate_active();
            }
        } else {
            ui.label("No component or wire selected.");
            ui.separator();
            ui.label(format!("Total Components: {}", self.components.len()));
            ui.label(format!("Total Wires: {}", self.wires.len()));
            ui.separator();
            ui.label("Click any component to inspect terminal voltages and operating temperature.");
        }
    }

    /// Evaluates one frame of the application UI, custom top frame, action toolbar, canvas, and docked panels.
    pub fn update(&mut self, ui: &mut Ui, _frame: &mut Frame) {
        // Global keyboard hotkeys
        self.handle_shortcuts(ui.ctx());

        // 1. Bespoke Custom Top Frame
        let mut top_config = self.top_frame_config.clone();
        top_config.circuit_name = self.project_title.clone();
        top_config.is_modified = self.is_dirty();

        Panel::top("custom_top_frame").show(ui, |ui| {
            let action = render_top_frame_with_app(ui, &top_config, self);
            match action {
                TopFrameAction::Close => {
                    self.request_action(PendingAction::CloseApp);
                }
                TopFrameAction::SaveProject => {
                    let _ = self.save_project();
                }
                TopFrameAction::OpenProject => {
                    self.open_open_dialog();
                }
                _ => {}
            }
        });

        // Action Toolbar
        Panel::top("action_toolbar").show(ui, |ui| {
            ui.horizontal(|ui| {
                if self.preferences.show_toolbar_run_dc && ui.button("Run DC (.OP)").clicked() {
                    self.run_dc_op();
                }
                if self.preferences.show_toolbar_run_transient && ui.button("Run Transient (.TRAN)").clicked() {
                    self.run_transient_demo();
                }
                if self.preferences.show_toolbar_export_netlist && ui.button("Export Netlist").clicked() {
                    self.sync_canvas_state();
                    self.spice_netlist_text = self.netlist_sync.sync_from_canvas(&self.canvas).to_string();
                    self.show_netlist_window = true;
                }
                if self.preferences.show_toolbar_clear_canvas && ui.button("Clear Canvas").clicked() {
                    self.clear_canvas_user();
                }
            });
        });

        // Sheet Tabs Bar
        Panel::top("sheet_tabs_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(4.0, 0.0);
                let mut switch_to = None;
                let mut remove_idx = None;
                let active_idx = self.sheets.active_sheet_idx;
                let sheet_count = self.sheets.sheets.len();

                for (idx, sheet) in self.sheets.sheets.iter().enumerate() {
                    let is_active = idx == active_idx;
                    let (bg_color, text_color) = if is_active {
                        (Color32::from_rgb(45, 60, 85), Color32::from_rgb(100, 200, 255))
                    } else {
                        (Color32::from_rgb(25, 30, 40), Color32::from_rgb(160, 175, 195))
                    };

                    egui::Frame::new()
                        .fill(bg_color)
                        .corner_radius(3.0)
                        .inner_margin(egui::Margin::symmetric(8, 3))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let btn = ui.add(
                                    egui::Button::new(
                                        egui::RichText::new(&sheet.name)
                                            .color(text_color)
                                            .size(11.5),
                                    )
                                    .frame(false),
                                );
                                if btn.clicked() {
                                    switch_to = Some(idx);
                                }

                                if sheet_count > 1 {
                                    let close_btn = ui.add(
                                        egui::Button::new(
                                            egui::RichText::new("x")
                                                .color(Color32::from_rgb(140, 150, 165))
                                                .size(10.0),
                                        )
                                        .frame(false),
                                    );
                                    if close_btn.clicked() {
                                        remove_idx = Some(idx);
                                    }
                                }
                            });
                        });
                }

                // Add Sheet button
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("+")
                                .color(Color32::from_rgb(120, 210, 140))
                                .size(13.0)
                                .strong(),
                        )
                        .frame(true),
                    )
                    .clicked()
                {
                    let count = self.sheets.sheets.len() + 1;
                    let new_name = format!("Sheet {}", count);
                    self.add_sheet(&new_name);
                }

                if let Some(idx) = switch_to {
                    self.switch_to_sheet(idx);
                }
                if let Some(idx) = remove_idx {
                    let _ = self.remove_sheet(idx);
                }
            });
        });

        // 2. Unobtrusive Bottom Status Bar
        Panel::bottom("status_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("Components: {}", self.components.len()));
                ui.separator();
                ui.label(format!("Wires: {}", self.wires.len()));
                ui.separator();
                ui.label(format!("Zoom: {:.1}x", self.canvas.zoom));

                if !self.sim_status.is_empty() {
                    ui.separator();
                    ui.label(&self.sim_status);
                }

                let telemetry = self.dynamics_backend.current_telemetry();
                if telemetry.step_count > 0 {
                    ui.separator();
                    ui.label(format!(
                        "Dynamics: t={:.3}s, steps={}, alt={:.1}m, spd={:.1}m/s",
                        telemetry.sim_time_s,
                        telemetry.step_count,
                        telemetry.altitude_m(),
                        telemetry.speed_m_per_s()
                    ));
                }
            });
        });

        // 3. Docked Oscilloscope Panel
        if self.show_oscilloscope {
            Panel::bottom("oscilloscope_panel")
                .resizable(true)
                .default_size(240.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("+ Detach Floating Window").clicked() {
                            self.multi_graph.detach_primary_to_floating();
                        }
                    });
                    self.multi_graph.primary_scope.show(ui);
                    self.oscilloscope = self.multi_graph.primary_scope.clone();
                });
        }

        // 4. Left Palette Panel
        if self.show_palette {
            Panel::left("palette_panel")
                .resizable(true)
                .default_size(180.0)
                .show(ui, |ui| {
                    self.render_palette(ui);
                });
        }

        // 5. Right Inspector Panel
        Panel::right("inspector_panel")
            .resizable(true)
            .default_size(240.0)
            .show(ui, |ui| {
                self.render_inspector(ui);
            });

        // 6. Center Canvas
        CentralPanel::default().show(ui, |ui| {
            self.render_canvas(ui);
        });

        // 7. Optional SPICE Netlist Window
        if self.show_netlist_window {
            let mut is_open = self.show_netlist_window;
            let mut sync_requested = false;
            egui::Window::new("Exported SPICE Netlist")
                .open(&mut is_open)
                .resizable(true)
                .default_size([450.0, 360.0])
                .show(ui.ctx(), |ui| {
                    ui.label("SPICE 3f5 Netlist representation of current schematic:");
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.spice_netlist_text)
                                .font(FontId::monospace(12.0))
                                .desired_rows(14)
                                .lock_focus(true),
                        );
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Sync to Canvas").clicked() {
                            sync_requested = true;
                        }
                    });
                });
            self.show_netlist_window = is_open;

            if sync_requested {
                if let Ok(delta) = self
                    .netlist_sync
                    .sync_to_canvas(&self.spice_netlist_text, &mut self.canvas)
                {
                    self.components = self.canvas.components.clone();
                    self.wires = self.canvas.wires.clone();
                    self.run_erc();
                    self.sim_status = format!(
                        "Synced to Canvas: +{} updated {}, -{}",
                        delta.added_count, delta.updated_count, delta.removed_count
                    );
                }
            }
        }

        // 8. SPICE Model Parameter Extraction Wizard Dialog
        self.extraction_wizard.ui(ui.ctx());

        // 9. Non-Linear Transient Sensitivity Dialog
        if self.sensitivity_dialog.run_requested {
            self.sensitivity_dialog.run_requested = false;
            self.run_sensitivity_analysis();
        }
        self.sensitivity_dialog.ui(ui.ctx());

        // 10. Monte Carlo Yield & Latin Hypercube Sampling Dialog
        if self.monte_carlo_dialog.run_requested {
            self.monte_carlo_dialog.run_requested = false;
            self.run_monte_carlo_sweep();
        }
        self.monte_carlo_dialog.ui(ui.ctx());

        // 11. Interactive Component Symbol & Shape Editor Dialog
        let mut saved_symbol = None;
        self.symbol_editor.show(ui.ctx(), |sym| {
            saved_symbol = Some(sym);
        });
        if let Some(sym) = saved_symbol {
            self.symbol_library.register(sym);
            self.sim_status = "Custom component symbol registered into library.".to_string();
        }

        // 12. Interactive RF S-Parameters, Smith Chart & Harmonic Balance Dialog
        if self.smith_chart_dialog.run_requested {
            self.smith_chart_dialog.run_requested = false;
            self.smith_chart_dialog.run_simulation();
        }
        self.smith_chart_dialog.ui(ui.ctx());

        // 13. Interactive Thermal Floorplan & Co-Simulation Dialog
        if self.thermal_floorplan_dialog.run_requested {
            self.thermal_floorplan_dialog.run_requested = false;
            self.run_thermal_cosim();
        }
        self.thermal_floorplan_dialog.ui(ui.ctx());

        // 14. Interactive Polariton Waveguide & Topological Photonic Cavity Dialog
        if self.polariton_cavity_dialog.run_requested {
            self.polariton_cavity_dialog.run_requested = false;
            self.polariton_cavity_dialog.run_simulation();
        }
        self.polariton_cavity_dialog.ui(ui.ctx());

        // 15. Interactive Distributed Cloud Parameter Sweep Cluster Dashboard Dialog
        self.cluster_dashboard_dialog.ui(ui.ctx());

        // 16. Interactive Neuromorphic Studio Canvas & Synaptic Weight Visualizer Dialog
        self.neuromorphic_snn_dialog.ui(ui.ctx());

        // 17. Interactive Exceptional Point Sensor & PT-Symmetric Circuit Simulator Dialog
        self.exceptional_point_dialog.ui(ui.ctx());

        // 18. Interactive Universal Topological Dirac & Weyl Semimetal Metamaterial Studio Dialog
        self.weyl_semimetal_dialog.ui(ui.ctx());

        // 19. Interactive Fractional Quantum Hall Anyon Braiding & Non-Abelian Topological Circuit Dialog
        self.fqh_braiding_dialog.ui(ui.ctx());

        // 20. Interactive Superconducting Josephson Traveling-Wave Parametric Amplifier (JTWPA) Studio Dialog
        self.jtwpa_dialog.ui(ui.ctx());

        // 21. Interactive Floquet Engineered Spatio-Temporal Acoustic Metasurface Studio Dialog
        self.floquet_metasurface_dialog.ui(ui.ctx());

        // 22. Interactive Non-Abelian Holonomic Geometric Phase Quantum Acoustic Processor Dialog
        self.holonomic_processor_dialog.ui(ui.ctx());

        // 23. Interactive Twisted Bilayer Moiré Phonon Polariton Magic-Angle Superlattice Studio Dialog
        self.twisted_moire_dialog.ui(ui.ctx());

        // 24. Interactive Acoustic Chern Insulator Chiral Circulator & Non-Reciprocal Router Studio Dialog
        self.chern_circulator_dialog.ui(ui.ctx());

        // 25. Interactive Non-Linear Soliton Kerr Microcomb Phononic Frequency Comb Studio Dialog
        self.kerr_microcomb_dialog.ui(ui.ctx());

        // 26. Interactive Non-Hermitian Chiral Exceptional Surface Acoustic Sensing Array Studio Dialog
        self.exceptional_surface_dialog.ui(ui.ctx());

        // 27. Interactive Topological Higher-Order Corner State Acoustic Resonator Dialog
        self.soti_corner_dialog.ui(ui.ctx());

        // 28. Interactive Topological Acoustic Flat-Band Lieb-Lattice Gauge Simulator Dialog
        self.lieb_lattice_dialog.ui(ui.ctx());

        // 29. Interactive Quantum Metamaterial Higher-Order Axion Insulator Dialog
        self.axion_insulator_dialog.ui(ui.ctx());

        // 30. Interactive Floquet-Bloch Quantum Acoustic Discrete Time Crystal Simulator Dialog
        self.floquet_time_crystal_dialog.ui(ui.ctx());

        // 31. Interactive Quantum Acoustic Protected Braiding Lattice Studio Dialog
        self.quantum_braiding_lattice_dialog.ui(ui.ctx());

        // 32. Interactive Cavity Optomechanical Squeezing & Phonon Counting Studio Dialog
        self.optomechanical_squeezing_dialog.ui(ui.ctx());

        // 33. Interactive Topological Acoustic Skyrmion Vortex Lattice & Domain Wall Router Dialog
        self.skyrmion_router_dialog.ui(ui.ctx());

        // 34. Interactive Non-Linear Acoustic Domain Wall Kink & Soliton Waveguide Dialog
        self.acoustic_soliton_dialog.ui(ui.ctx());

        // 35. Interactive Valley-Polarized Topological Acoustic Multiplexer & Beam Splitter Dialog
        self.valley_multiplexer_dialog.ui(ui.ctx());

        // 36. Interactive Non-Hermitian Skin Effect Acoustic Sensor & Directional Funnel Dialog
        self.non_hermitian_skin_dialog.ui(ui.ctx());

        // 37. Interactive Topological Acoustic Quadrupole Second-Harmonic Generation Dialog
        self.quadrupole_shg_dialog.ui(ui.ctx());

        // 38. Interactive Topological Acoustic Synthetic Dimension & 4D QHE Dialog
        self.synthetic_4d_dialog.ui(ui.ctx());

        // 39. Interactive Parity-Time (PT) Symmetric Acoustic Metamaterial & Unidirectional Invisibility Dialog
        self.pt_symmetric_dialog.ui(ui.ctx());

        // 40. Interactive Topological Acoustic Bound States in the Continuum (BIC) & Vortex Cavity Dialog
        self.acoustic_bic_dialog.ui(ui.ctx());

        // 41. Interactive Non-Abelian Euler Class Topological Acoustic Dialog
        self.euler_acoustic_dialog.ui(ui.ctx());

        // 42. Interactive Higher-Order Topological Acoustic Octupole Insulator Dialog
        self.octupole_insulator_dialog.ui(ui.ctx());

        // 43. Interactive Topological Acoustic Moire Quasicrystal & AAH Mobility Edge Dialog
        self.aah_quasicrystal_dialog.ui(ui.ctx());

        // 44. Interactive Acoustic Valley-Hall Vortex Pumping & Synthetic Chiral Gauge Field Dialog
        self.valley_hall_vortex_dialog.ui(ui.ctx());

        // 45. Interactive Acoustic Higher-Order Skyrmion Beam Deflector & Chiral Router Dialog
        self.skyrmion_deflector_dialog.ui(ui.ctx());

        // 46. Interactive Floquet Synthetic Frequency Dimension & Frequency Soliton Dialog
        self.floquet_frequency_dialog.ui(ui.ctx());

        // 47. Interactive Non-Hermitian Higher-Order Topological Corner Laser Dialog
        self.corner_laser_dialog.ui(ui.ctx());

        // 47b. Interactive Multi-Octave Acoustic Metasurface Wavefront Hologram & Tractor Beam Dialog
        self.metasurface_hologram_dialog.ui(ui.ctx());

        // 47c. Interactive Quantum Metamaterial Non-Abelian Majorana Braid Interconnect & Surface Code Dialog
        self.majorana_surface_code_dialog.ui(ui.ctx());

        // 47d. Interactive Cryogenic Quantum Optomechanical Transducer Dialog
        self.optomechanical_transducer_dialog.ui(ui.ctx());

        // 47e. Interactive Topological Corner-Polariton Micro-Comb Soliton & Frequency Synthesizer Dialog
        self.corner_polariton_microcomb_dialog.ui(ui.ctx());

        // 48. Interactive Directional Cosmic Heavy Ion Radiation Track & 3D Anisotropic Shielding Dialog
        self.directional_radiation_dialog.ui(ui.ctx());

        // 49. Interactive Atmospheric Secondary Neutron Spallation Cascade & DO-254 DAL-A SER Dialog
        self.atmospheric_neutron_dialog.ui(ui.ctx());

        // 50. Interactive Aerospace Thermal-Vacuum Radiation Dissipation & Orbital Cycling Dialog
        self.thermal_vacuum_dialog.ui(ui.ctx());

        // 51. Interactive SpaceWire/SpaceFibre & Avionics AFDX Bus Dialog
        self.space_avionics_bus_dialog.ui(ui.ctx());

        // 52. Interactive RHBD DRC & Autonomous Self-Healing Co-Simulator Dialog
        self.rhbd_self_healing_dialog.ui(ui.ctx());

        // 53. Interactive Production Economics & Hierarchical BOM Cost Estimator Dialog
        self.production_economics_dialog.ui(ui.ctx());

        // 54. Interactive 2.5D/3D Multi-Die & Chiplet Packaging Studio Dialog
        self.chiplet_packaging_dialog.ui(ui.ctx());

        // 55. Interactive Closed-Loop Dynamic Electro-Thermal & Power Throttling Studio Dialog
        self.electrothermal_throttling_dialog.ui(ui.ctx());

        // 56. Interactive Power Delivery Network (PDN) & Dynamic Droop Co-Simulator Dialog
        self.pdn_droop_dialog.ui(ui.ctx());

        // 57. Interactive Physics-Based Silicon Aging, Reliability & Electromigration (EM) Dialog
        self.silicon_aging_dialog.ui(ui.ctx());

        // 58. Interactive Wafer-Scale Yield, DFM & Harvesting Economics Dialog
        self.wafer_yield_dialog.ui(ui.ctx());

        // 59. Interactive Automated Multi-Objective PPA-C Design Space Exploration (DSE) Dialog
        self.dse_dialog.ui(ui.ctx());

        // 60. Interactive Silicon Lifecycle Management & On-Die Telemetry Digital Twin Dialog
        self.slm_dialog.ui(ui.ctx());

        // 61. Interactive Web Studio WASM Binary Size Optimization & Cache Invalidation Dialog
        self.wasm_optimization_dialog.ui(ui.ctx());

        // 62. Interactive Progressive Web App (PWA) Offline ServiceWorker & Asset Cache Dialog
        self.pwa_dialog.ui(ui.ctx());

        // 63. Interactive Native Desktop Studio, Tauri v2 Shell & Zero-Copy Binary IPC Co-Processor Dialog
        self.desktop_ipc_dialog.ui(ui.ctx());

        // 64. Interactive Real-Time Collaborative WebRTC Peer-to-Peer Multi-User CAD Mesh Dialog
        self.webrtc_mesh_dialog.ui(ui.ctx());

        // 65. Interactive WebGL2 / WebGPU Compute Shader Hardware-Accelerated SPICE Co-Processor Dialog
        self.webgpu_spice_dialog.ui(ui.ctx());

        // 66. Interactive Microscopic Electron & Phonon Wavepacket Scattering Simulator Dialog
        self.wavepacket_scattering_dialog.ui(ui.ctx());

        // 67. Interactive Quantum Spin-Torque Oscillator & Magnetic Skyrmion Reservoir Computing Dialog
        self.skyrmion_reservoir_dialog.ui(ui.ctx());

        // 68. Interactive Coherent Phonon-Magnon Polariton Transducer Dialog
        self.phonon_magnon_dialog.ui(ui.ctx());

        // 69. Interactive Topological Higher-Order Acoustic Quadrupole Parametric Waveguide Dialog
        self.quadrupole_parametric_dialog.ui(ui.ctx());

        // 70. Interactive Non-Hermitian Floquet Skin-Effect Sensor & EP Magnetometer Dialog
        self.non_hermitian_sensor_dialog.ui(ui.ctx());

        // 71. Interactive Topological Corner-Induced Second-Harmonic Waveguide Interconnect & Doubler Dialog
        self.corner_doubler_dialog.ui(ui.ctx());

        // 72. Interactive Universal Non-Abelian Braiding & Topological Co-Processor Dialog
        self.universal_braiding_dialog.ui(ui.ctx());

        // 73. Interactive Topological Floquet Chiral Polariton Circulator Dialog
        self.chiral_circulator_dialog.ui(ui.ctx());

        // 74. Interactive Superconducting Josephson Parametric Acoustic Waveguide Dialog
        self.josephson_parametric_dialog.ui(ui.ctx());

        // 75. Interactive Cavity Optomagnonic Polariton Frequency Comb Dialog
        self.optomagnonic_comb_dialog.ui(ui.ctx());

        // 76. Interactive Floquet Time-Crystal Magnetometer Sensor Dialog
        self.floquet_sensor_dialog.ui(ui.ctx());

        // 29b. Interactive Subcircuit Packaging Dialog (.phnc)
        if let Some(action) = self.subcircuit_dialog.show(ui.ctx()) {
            match action {
                SubcircuitDialogAction::SavePackage(pkg) => {
                    self.sim_status = format!("Subcircuit package '{}' registered", pkg.name);
                    self.subcircuit_registry.register(pkg);
                }
                SubcircuitDialogAction::Cancel => {}
            }
        }

        // 29c. Multi-Graph Floating Oscilloscope Windows
        self.multi_graph.render_floating_windows(ui.ctx());

        // 29d. Interactive Lua Testbench Console Dialog & Expression Waveform Routing
        if self.lua_console_dialog.is_open {
            for (net, &v) in &self.dc_node_voltages {
                self.lua_console_dialog.engine.set_voltage(net, v);
            }
            for (idx, &i) in &self.wire_currents {
                self.lua_console_dialog
                    .engine
                    .set_current(&format!("wire_{}", idx), i);
            }
        }
        self.lua_console_dialog.ui(ui.ctx());
        if !self.lua_console_dialog.engine.generated_traces.is_empty() {
            let traces = self.lua_console_dialog.engine.generated_traces.clone();
            self.lua_console_dialog.engine.generated_traces.clear();
            for trace in &traces {
                self.multi_graph.primary_scope.add_trace(trace.clone());
                self.oscilloscope.add_trace(trace.clone());
            }
        }

        // 30. Unsaved Changes Confirmation Modal
        if let Some(pending) = self.pending_confirmation_action.clone() {
            let decision = ConfirmationModal::show(ui.ctx(), &self.project_title, &pending);
            match decision {
                ConfirmationDecision::SaveAndProceed => {
                    let _ = self.save_project();
                    self.pending_confirmation_action = None;
                    self.execute_confirmation_action(pending);
                }
                ConfirmationDecision::DiscardAndProceed => {
                    self.pending_confirmation_action = None;
                    self.execute_confirmation_action(pending);
                }
                ConfirmationDecision::Cancel => {
                    self.pending_confirmation_action = None;
                }
                ConfirmationDecision::None => {}
            }
        }

        // 31. Project Manager Modal Dialog
        let project_list = self.storage_manager.list_projects().unwrap_or_default();
        let dialog_action = self.project_dialog.show(ui.ctx(), &project_list);
        match dialog_action {
            ProjectDialogAction::Open(name) => {
                self.project_dialog.close();
                self.request_action(PendingAction::OpenProject(Some(std::path::PathBuf::from(name))));
            }
            ProjectDialogAction::SaveAs(name) => {
                let _ = self.save_project_as(&name);
                self.project_dialog.close();
            }
            ProjectDialogAction::Delete(name) => {
                let _ = self.storage_manager.delete_project(&name);
            }
            ProjectDialogAction::Export(name) => {
                self.export_project(&name);
            }
            ProjectDialogAction::Close => {
                self.project_dialog.close();
            }
            ProjectDialogAction::None => {}
        }

        // 31b. Preferences Modal Settings Dialog
        let mut cmap = self.thermal.colormap;
        if self.preferences_dialog.show(
            ui.ctx(),
            &mut self.preferences,
            &self.action_registry,
            &mut self.history,
            &mut cmap,
        ) {
            self.thermal.colormap = cmap;
            self.canvas.show_grid = self.preferences.show_grid;
            self.canvas.grid_size = self.preferences.grid_size;
            self.history.max_depth = self.preferences.in_memory_history_limit;
            self.show_thermal_overlay = self.preferences.thermal_overlay_enabled;
            let _ = self.preferences.save_to_storage(&mut self.storage_manager);
        }

        // 32. Application Close Command handling
        if self.should_close {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

impl App for PhononApp {
    fn ui(&mut self, ui: &mut Ui, frame: &mut Frame) {
        self.update(ui, frame);
    }
}
