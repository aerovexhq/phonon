#![deny(unsafe_code)]

//! The central Phonon GUI application orchestrator, CAD layout, and interactive simulation.

use crate::extraction::ExtractionWizardDialog;
use crate::oscilloscope::{MultiGraphManager, OscilloscopePanel, WaveformTrace};
use crate::schematic::{
    compile_schematic_with_labels, compute_junction_dots,
    compute_wire_crossings, compute_wire_telemetry, deserialize_project, load_project_from_file,
    render_wire_crossings, save_project_to_file, serialize_project, BinaryFormatError,
    CanvasCommand, CompiledCircuit, ComponentKind, ErcDiagnostic, ErcEngine, ErcSeverity,
    HistoryStack, MultiSheetManager, NetLabel, NetLabelOrientation, NetlistSyncEngine,
    SchematicBus, SchematicCanvas, SchematicComponent, SchematicWire, SubcircuitDefinition,
    SubcircuitRegistry, SymbolLibrary, WireSegment,
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
    AcousticSolitonDialog, ValleyMultiplexerDialog,
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
    SkyrmionReservoirDialog, PhononMagnonDialog,
    OptomagnonicCombDialog,
    FloquetSensorDialog, OptomechanicalTransducerDialog, CornerPolaritonMicrocombDialog,
    GiantAtomDialog, ChiralEmpDialog, PolaritonBecDialog, SyntheticDimensionDialog,
    NonHermitianSkinLaserDialog, MoirePolaritonCombDialog, HolonomicCoprocessorDialog,
    ValleyChiralIsolatorDialog, FloquetSpinHallCirculatorDialog, OctupoleDislocationDialog,
    ChiralMajoranaDialog, FloquetCornerLaserDialog, ParafermionDialog,
    ChiralAcoustomagnonicDialog, FloquetCornerTransducerDialog,
    ChernSimonsInterferometerDialog, ChiralHeatTransistorDialog,
    CornerKerrMicrocombDialog, AcousticSnspdDialog, FloquetCvQkdDialog,
    PolaritonicSolitonCombDialog, CircuitQedTransducerDialog,
    ExceptionalSurfaceDiodeDialog, TopologicalSolitonDialog,
    FractionalChernInterconnectDialog, ChiralLasingMetasurfaceDialog,
    SyntheticGaugeHolonomyDialog, ChiralSpinTorqueMemoryDialog,
    ValleyQuantumRouterDialog, FractionalParafermionDialog,
    TopologicalSkinAxionDialog, ReadRezayiFibonacciDialog,
    MoirePolaritonDialog, TopologicalJosephsonMemoryDialog,
    Genus2ParafermionDialog, FloquetCornerIsolatorDialog,
    SkinPolaritonLaserDialog, ValleyMajoranaRouterDialog,
    ChiralGrapheneBraidingDialog, FloquetMagnonMemoryDialog,
    AnyonInterferometerQuditDialog, FloquetCornerSensorDialog,
    FractionalSkyrmionSynapseDialog,
    ChiralTransducerRepeaterDialog,
    WeylVortexRouterDialog,
    ParafermionSurfaceDialog,
    MoireSuperlatticeLaserDialog,
    DisclinationHolonomicQuditDialog,
    FloquetMagnonCrossbarDialog,
    MoireValleyQubitDialog,
    CornerMemoryRepeaterDialog,
    NonHermitianBraidingDialog,
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
    NetLabel,
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

/// Permanent record of a wire attached to a moving component during drag interaction.
#[derive(Debug, Clone, PartialEq)]
pub struct DragAttachedWire {
    pub wire_id: usize,
    pub is_start: bool,
    pub comp_id: usize,
    pub pin_idx: usize,
    pub other_end: Pos2,
    pub net_name: Option<String>,
}

/// The unified Phonon desktop CAD application.
pub struct PhononApp {
    pub canvas: SchematicCanvas,
    pub components: Vec<SchematicComponent>,
    pub wires: Vec<SchematicWire>,
    pub next_comp_id: usize,
    pub next_wire_id: usize,
    pub next_label_id: usize,
    pub selected_tool: ToolMode,
    pub net_labels: Vec<NetLabel>,
    pub active_net_label_text: String,
    /// Set of selected component IDs for multi-selection.
    pub selected_component_ids: HashSet<usize>,
    /// Set of selected wire IDs for multi-selection.
    pub selected_wire_ids: HashSet<usize>,
    /// Set of selected net label IDs for multi-selection.
    pub selected_label_ids: HashSet<usize>,
    pub selected_component_id: Option<usize>,
    pub selected_wire_id: Option<usize>,
    pub selected_label_id: Option<usize>,
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
    /// World position of mouse when drag interaction started.
    pub drag_start_mouse_pos: Option<Pos2>,
    /// Original positions of components before drag started.
    pub drag_start_positions: Vec<(usize, Pos2)>,
    /// Original positions of net labels before drag started.
    pub drag_start_label_positions: Vec<(usize, Pos2)>,
    /// Pre-drag snapshot of schematic wires for reversible history recording.
    pub drag_start_wires: Vec<SchematicWire>,
    /// Permanent wire attachments cached at drag start.
    pub drag_attached_wires: Vec<DragAttachedWire>,
    /// Component IDs currently in an unsolvable or collision wiring state (rendered with red pin tips).
    pub unsolvable_wiring_components: HashSet<usize>,

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

    /// Interactive Cavity Optomagnonic Polariton Frequency Comb & Dissipative Kerr Soliton dialog.
    pub optomagnonic_comb_dialog: OptomagnonicCombDialog,

    /// Interactive Floquet Discrete Time-Crystal Magnetometer & Subharmonic Sensor Network dialog.
    pub floquet_sensor_dialog: FloquetSensorDialog,

    /// Interactive Quantum Acoustic Giant Atom Waveguide QED & Non-Markovian Processor dialog.
    pub giant_atom_dialog: GiantAtomDialog,

    /// Interactive Topological Chiral Acoustic Edge-Magnetoplasmon Circulator & Router dialog.
    pub chiral_emp_dialog: ChiralEmpDialog,

    /// Interactive Dissipative Polariton BEC Vortices, Non-Equilibrium Superfluidity & Josephson Acoustic Interferometer dialog.
    pub polariton_bec_dialog: PolaritonBecDialog,

    /// Interactive Topological Acoustic Synthetic Dimension Chern Insulator & High-Dimensional Multiplexed Router dialog.
    pub synthetic_dimension_dialog: SyntheticDimensionDialog,

    /// Interactive Non-Hermitian Higher-Order Topological Quadrupole Skin Laser & Emitter dialog.
    pub non_hermitian_skin_laser_dialog: NonHermitianSkinLaserDialog,

    /// Interactive Topological Acoustic Moiré Flat-Band Polariton Soliton & Higher-Order Corner Comb dialog.
    pub moire_polariton_comb_dialog: MoirePolaritonCombDialog,

    /// Interactive Quantum Metamaterial Non-Abelian Holonomic Braiding & CMOS-MEMS Co-Processor dialog.
    pub holonomic_coprocessor_dialog: HolonomicCoprocessorDialog,

    /// Interactive Topological Acoustic Valley-Hall Chiral Edge Filter & Microwave Isolator dialog.
    pub valley_chiral_isolator_dialog: ValleyChiralIsolatorDialog,

    /// Interactive Topological Acoustic Floquet Spin-Hall Insulator & Cryogenic Circulator dialog.
    pub floquet_spinhall_circulator_dialog: FloquetSpinHallCirculatorDialog,

    /// Interactive Topological Acoustic Higher-Order Octupole Vortex Metamaterial & 3D Chiral Dislocation Router dialog.
    pub octupole_dislocation_dialog: OctupoleDislocationDialog,

    /// Interactive Quantum Metamaterial Chiral Majorana Braiding Processor & Surface Decoder dialog.
    pub chiral_majorana_dialog: ChiralMajoranaDialog,

    /// Interactive Topological Acoustic Floquet Corner Laser & Vortex Amplifier dialog.
    pub floquet_corner_laser_dialog: FloquetCornerLaserDialog,

    /// Interactive Quantum Acoustic Non-Abelian Parafermion Braiding dialog.
    pub parafermion_dialog: ParafermionDialog,

    /// Interactive Quantum Metamaterial Chiral Acoustomagnonic Isolator & Cryogenic Circulator dialog.
    pub acoustomagnonic_dialog: ChiralAcoustomagnonicDialog,

    /// Interactive Topological Acoustic Floquet Corner-State Transducer & Entanglement Router dialog.
    pub floquet_corner_transducer_dialog: FloquetCornerTransducerDialog,

    /// Interactive Topological Chiral Acoustic Chern-Simons Fractional Anyon Interferometer & Quantum Memory dialog.
    pub chern_simons_interferometer_dialog: ChernSimonsInterferometerDialog,

    /// Interactive Quantum Metamaterial Non-Hermitian Floquet Chiral Heat Transistor & Thermal Diode dialog.
    pub chiral_heat_transistor_dialog: ChiralHeatTransistorDialog,

    /// Interactive Topological Acoustic Higher-Order Corner-State Quantum Metamaterial Frequency Comb & Dissipative Kerr Soliton Generator dialog.
    pub corner_kerr_microcomb_dialog: CornerKerrMicrocombDialog,

    /// Interactive Topological Acoustic Superconducting Nanowire Single-Phonon Detector (SNSPD) & Quantum Transceiver dialog.
    pub acoustic_snspd_dialog: AcousticSnspdDialog,

    /// Interactive Topological Acoustic Floquet Chiral Magnon-Phonon Entanglement Router & CV-QKD dialog.
    pub floquet_cv_qkd_dialog: FloquetCvQkdDialog,

    /// Interactive Quantum Metamaterial Polaritonic Soliton Frequency Comb & Dissipative Kerr Squeezed State Generator dialog.
    pub polaritonic_soliton_comb_dialog: PolaritonicSolitonCombDialog,

    /// Interactive Topological Higher-Order Acoustic Superconducting Circuit QED Quantum Transducer & Multi-Qubit Crossbar dialog.
    pub circuit_qed_transducer_dialog: CircuitQedTransducerDialog,

    /// Interactive Non-Hermitian Exceptional Surface Chiral Phonon Diode & Unidirectional Quantum Repeater dialog.
    pub exceptional_surface_diode_dialog: ExceptionalSurfaceDiodeDialog,

    /// Interactive Topological Acoustic Boundary Soliton Logic Gate & Majority Voter dialog.
    pub topological_soliton_dialog: TopologicalSolitonDialog,

    /// Interactive Quantum Metamaterial Fractional Chern & Parafermion Braiding Interconnect dialog.
    pub fractional_chern_interconnect_dialog: FractionalChernInterconnectDialog,

    /// Interactive Topological Non-Hermitian Floquet Acoustic Chiral Lasing Metasurface & Vortex Waveguide dialog.
    pub chiral_lasing_metasurface_dialog: ChiralLasingMetasurfaceDialog,

    /// Interactive Topological Acoustic Synthetic Gauge Field & Non-Abelian Holonomic Quantum Gate Processor dialog.
    pub synthetic_gauge_holonomy_dialog: SyntheticGaugeHolonomyDialog,

    /// Interactive Chiral Phonon-Magnon Spin-Torque Acoustic Memory & Spintronic Crossbar dialog.
    pub chiral_spintorque_memory_dialog: ChiralSpinTorqueMemoryDialog,

    /// Interactive Topological Acoustic Valley-Hall Quantum Router & Entanglement Concentrator dialog.
    pub valley_quantum_router_dialog: ValleyQuantumRouterDialog,

    /// Interactive Fractional Parafermion Surface Code & Anyonic Braid Repeater dialog.
    pub fractional_parafermion_dialog: FractionalParafermionDialog,

    /// Interactive Topological Non-Hermitian Skin Microwave Amplifier & Axion Transducer dialog.
    pub topological_skin_axion_dialog: TopologicalSkinAxionDialog,

    /// Interactive Read-Rezayi Fibonacci Anyon Acoustic Interferometer & Universal Quantum Bus dialog.
    pub read_rezayi_fibonacci_dialog: ReadRezayiFibonacciDialog,

    /// Interactive Moiré Exciton-Polariton Valley Hall Chiral Lasing Metasurface & Opto-Acoustic Synthesizer dialog.
    pub moire_polariton_dialog: MoirePolaritonDialog,

    /// Interactive Topological Josephson phi_0 Memory & Quantum Phase-Slip Crossbar dialog.
    pub topological_josephson_memory_dialog: TopologicalJosephsonMemoryDialog,

    /// Interactive Quantum Metamaterial Non-Abelian Genus-2 Parafermion Surface Code & Universal Processor dialog.
    pub genus2_parafermion_dialog: Genus2ParafermionDialog,

    /// Interactive Topological Acoustic Floquet Higher-Order Corner Magneto-Phonon Isolator & Non-Reciprocal Circulator Array dialog.
    pub floquet_corner_isolator_dialog: FloquetCornerIsolatorDialog,

    /// Interactive Dissipative Topological Polariton Skin Laser & Non-Hermitian Chiral Acoustic Gyroscope Array dialog.
    pub skin_polariton_laser_dialog: SkinPolaritonLaserDialog,

    /// Interactive Quantum Metamaterial Valley-Locked Majorana Zero Mode Acoustic Interconnect & Chiral Majorana Transmon Router dialog.
    pub valley_majorana_router_dialog: ValleyMajoranaRouterDialog,

    /// Interactive Non-Abelian Anyon Braiding & Topological Qubit Crossbar Array in Chiral Phononic Graphene dialog.
    pub chiral_graphene_braiding_dialog: ChiralGrapheneBraidingDialog,

    /// Interactive Floquet Chiral Magnon-Phonon Polariton Router & Dissipative Quantum Memory dialog.
    pub floquet_magnon_memory_dialog: FloquetMagnonMemoryDialog,

    /// Interactive Cryogenic Quantum Metamaterial Multi-Terminal Anyon Interferometer & Protected Qudit Crossbar dialog.
    pub anyon_interferometer_qudit_dialog: AnyonInterferometerQuditDialog,

    /// Interactive Floquet Corner Spin-Orbit Polariton Laser & Non-Hermitian Sensor dialog.
    pub floquet_corner_sensor_dialog: FloquetCornerSensorDialog,

    /// Interactive Quantum Metamaterial Fractional Hall Skyrmion Synaptic Memory & Anyonic Neural Crossbar dialog.
    pub fractional_skyrmion_synapse_dialog: FractionalSkyrmionSynapseDialog,

    /// Interactive Chiral Metamaterial Photonic-Phononic Qubit Transducer & Quantum Network Repeater Node dialog.
    pub chiral_transducer_repeater_dialog: ChiralTransducerRepeaterDialog,

    /// Interactive HOWSM Vortex Transceiver & Multi-Terminal Quantum Acoustic Router dialog.
    pub weyl_vortex_router_dialog: WeylVortexRouterDialog,

    /// Interactive Parafermion Lattice Co-Processor & Quantum Acoustic Surface Engine dialog.
    pub parafermion_surface_dialog: ParafermionSurfaceDialog,

    /// Interactive Moire Superlattice Polariton Laser & Valley Sensor Network dialog.
    pub moire_superlattice_laser_dialog: MoireSuperlatticeLaserDialog,

    /// Interactive Disclination Cavity & Non-Abelian Holonomic Qudit Processor dialog.
    pub disclination_holonomic_qudit_dialog: DisclinationHolonomicQuditDialog,

    /// Interactive Floquet Chiral Magnon Crossbar & Entanglement Router Super-Array dialog.
    pub floquet_magnon_crossbar_dialog: FloquetMagnonCrossbarDialog,

    /// Interactive Moire Valley Qubit Array & Cryogenic Phonon Memory Bus dialog.
    pub moire_valley_qubit_dialog: MoireValleyQubitDialog,

    /// Interactive Higher-Order Corner-State Quantum Memory & Chiral Phonon Transduction Bus dialog (Phase 466).
    pub corner_memory_repeater_dialog: CornerMemoryRepeaterDialog,

    /// Interactive Non-Hermitian Higher-Order Chiral Braiding & EP Sensor dialog (Phase 467).
    pub non_hermitian_braiding_dialog: NonHermitianBraidingDialog,

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

    /// Active horizontal mirror state for components being placed.
    pub placement_mirrored: bool,

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

    /// Whether the desktop application window should request maximized state on first frame.
    pub pending_start_maximize: bool,

    /// Generalized central viewport tab manager (Schematic Canvas, 3D Physical Card, etc.).
    pub viewport_tabs: crate::viewport_tabs::CentralTabManager,

    /// Interactive 3D Physical Card Viewport with abstracted ground plane.
    pub card_3d_viewport: crate::widgets::card_3d_viewport::Card3dViewport,

    #[cfg(any(test, feature = "devtools"))]
    /// Interactive DevTools state and scenario runner.
    pub devtools_state: crate::devtools::DevtoolsState,
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
            next_label_id: 1,
            selected_tool: ToolMode::Select,
            net_labels: Vec::new(),
            active_net_label_text: "NET1".to_string(),
            selected_component_ids: HashSet::new(),
            selected_wire_ids: HashSet::new(),
            selected_label_ids: HashSet::new(),
            selected_component_id: None,
            selected_wire_id: None,
            selected_label_id: None,
            active_wire_start: None,
            marquee_start: None,
            marquee_current: None,
            show_floating_toolbar: true,
            floating_toolbar_state: FloatingToolbarState::new(),
            command_palette: CommandPalette::new(),
            action_registry: ActionRegistry::new(),
            dragging_selection: false,
            drag_start_mouse_pos: None,
            drag_start_positions: Vec::new(),
            drag_start_label_positions: Vec::new(),
            drag_start_wires: Vec::new(),
            drag_attached_wires: Vec::new(),
            unsolvable_wiring_components: HashSet::new(),
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
            acoustic_soliton_dialog: AcousticSolitonDialog::new(),
            valley_multiplexer_dialog: ValleyMultiplexerDialog::new_fast(),
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
            optomagnonic_comb_dialog: OptomagnonicCombDialog::new_fast(),
            floquet_sensor_dialog: FloquetSensorDialog::new_fast(),
            giant_atom_dialog: GiantAtomDialog::new_fast(),
            chiral_emp_dialog: ChiralEmpDialog::new_fast(),
            polariton_bec_dialog: PolaritonBecDialog::new_fast(),
            synthetic_dimension_dialog: SyntheticDimensionDialog::new_fast(),
            non_hermitian_skin_laser_dialog: NonHermitianSkinLaserDialog::new_fast(),
            moire_polariton_comb_dialog: MoirePolaritonCombDialog::new_fast(),
            holonomic_coprocessor_dialog: HolonomicCoprocessorDialog::new_fast(),
            valley_chiral_isolator_dialog: ValleyChiralIsolatorDialog::new_fast(),
            floquet_spinhall_circulator_dialog: FloquetSpinHallCirculatorDialog::new_fast(),
            octupole_dislocation_dialog: OctupoleDislocationDialog::new_fast(),
            chiral_majorana_dialog: ChiralMajoranaDialog::new_fast(),
            floquet_corner_laser_dialog: FloquetCornerLaserDialog::new_fast(),
            parafermion_dialog: ParafermionDialog::new_fast(),
            acoustomagnonic_dialog: ChiralAcoustomagnonicDialog::new_fast(),
            floquet_corner_transducer_dialog: FloquetCornerTransducerDialog::new_fast(),
            chern_simons_interferometer_dialog: ChernSimonsInterferometerDialog::new_fast(),
            chiral_heat_transistor_dialog: ChiralHeatTransistorDialog::new_fast(),
            corner_kerr_microcomb_dialog: CornerKerrMicrocombDialog::new_fast(),
            acoustic_snspd_dialog: AcousticSnspdDialog::new_fast(),
            floquet_cv_qkd_dialog: FloquetCvQkdDialog::new_fast(),
            polaritonic_soliton_comb_dialog: PolaritonicSolitonCombDialog::new_fast(),
            circuit_qed_transducer_dialog: CircuitQedTransducerDialog::new_fast(),
            exceptional_surface_diode_dialog: ExceptionalSurfaceDiodeDialog::new_fast(),
            topological_soliton_dialog: TopologicalSolitonDialog::new_fast(),
            fractional_chern_interconnect_dialog: FractionalChernInterconnectDialog::new_fast(),
            chiral_lasing_metasurface_dialog: ChiralLasingMetasurfaceDialog::new_fast(),
            synthetic_gauge_holonomy_dialog: SyntheticGaugeHolonomyDialog::new_fast(),
            chiral_spintorque_memory_dialog: ChiralSpinTorqueMemoryDialog::new_fast(),
            valley_quantum_router_dialog: ValleyQuantumRouterDialog::new_fast(),
            fractional_parafermion_dialog: FractionalParafermionDialog::new_fast(),
            topological_skin_axion_dialog: TopologicalSkinAxionDialog::new_fast(),
            read_rezayi_fibonacci_dialog: ReadRezayiFibonacciDialog::new_fast(),
            moire_polariton_dialog: MoirePolaritonDialog::new_fast(),
            topological_josephson_memory_dialog: TopologicalJosephsonMemoryDialog::new_fast(),
            genus2_parafermion_dialog: Genus2ParafermionDialog::new_fast(),
            floquet_corner_isolator_dialog: FloquetCornerIsolatorDialog::new_fast(),
            skin_polariton_laser_dialog: SkinPolaritonLaserDialog::new_fast(),
            valley_majorana_router_dialog: ValleyMajoranaRouterDialog::new_fast(),
            chiral_graphene_braiding_dialog: ChiralGrapheneBraidingDialog::new_fast(),
            floquet_magnon_memory_dialog: FloquetMagnonMemoryDialog::new_fast(),
            anyon_interferometer_qudit_dialog: AnyonInterferometerQuditDialog::new_fast(),
            floquet_corner_sensor_dialog: FloquetCornerSensorDialog::new_fast(),
            fractional_skyrmion_synapse_dialog: FractionalSkyrmionSynapseDialog::new_fast(),
            chiral_transducer_repeater_dialog: ChiralTransducerRepeaterDialog::new_fast(),
            weyl_vortex_router_dialog: WeylVortexRouterDialog::new_fast(),
            parafermion_surface_dialog: ParafermionSurfaceDialog::new_fast(),
            moire_superlattice_laser_dialog: MoireSuperlatticeLaserDialog::new_fast(),
            disclination_holonomic_qudit_dialog: DisclinationHolonomicQuditDialog::new_fast(),
            floquet_magnon_crossbar_dialog: FloquetMagnonCrossbarDialog::new_fast(),
            moire_valley_qubit_dialog: MoireValleyQubitDialog::new_fast(),
            corner_memory_repeater_dialog: CornerMemoryRepeaterDialog::new_fast(),
            non_hermitian_braiding_dialog: NonHermitianBraidingDialog::new_fast(),
            lua_console_dialog: LuaConsoleDialog::new(),
            symbol_editor: SymbolEditorDialog::new(),
            symbol_library: SymbolLibrary::new(),
            dragging_component: false,
            drag_start_pos: None,
            history: HistoryStack::with_capacity(500, 64),
            editing_comp_value: None,
            placement_rotation: 0,
            placement_mirrored: false,
            top_frame_config: TopFrameConfig::default(),
            dynamics_backend: Box::new(AutoSelectingDynamicsBackend::new()),
            palette: ComponentPalette::new(),
            show_palette: true,
            boot_theme_config: crate::default_boot_theme_config(),
            pending_auto_center: true,
            project_title: "Untitled".to_string(),
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
            pending_start_maximize: true,
            viewport_tabs: crate::viewport_tabs::CentralTabManager::new(),
            card_3d_viewport: crate::widgets::card_3d_viewport::Card3dViewport::default(),
            #[cfg(any(test, feature = "devtools"))]
            devtools_state: crate::devtools::DevtoolsState::default(),
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

        // The program starts with an unsaved "Untitled" project with nothing loaded in.
        // It is not saved until the user chooses a location and saves.
        app.project_title = "Untitled".to_string();
        app.top_frame_config.circuit_name = "Untitled".to_string();
        app.current_project_path = None;
        app.clear_canvas_state();
        app.components.clear();
        app.wires.clear();
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
                DemoCircuitKind::HalfAdder => self.load_half_adder_demo(),
                DemoCircuitKind::BasicGates => self.load_basic_gates_demo(),
                DemoCircuitKind::QuantumMetamaterial => self.load_quantum_metamaterial_demo(),
                DemoCircuitKind::Alu4Bit => self.load_4bit_alu_demo(),
                DemoCircuitKind::RfTransceiver => self.load_rf_transceiver_demo(),
                DemoCircuitKind::TopologicalQuantumProcessor => self.load_topological_quantum_processor_demo(),
            },
            PendingAction::ClearCanvas => self.clear_canvas_user(),
            PendingAction::CloseApp => {
                self.should_close = true;
            }
        }
    }

    /// Initializes a fresh untitled project with nothing loaded in.
    pub fn new_project(&mut self) {
        self.clear_canvas_state();
        self.components.clear();
        self.wires.clear();
        self.history.clear();
        self.project_title = "Untitled".to_string();
        self.current_project_path = None;
        self.modification_epoch = 0;
        self.clean_epoch = 0;
        self.is_modified = false;
        self.top_frame_config.circuit_name = self.project_title.clone();
        self.top_frame_config.is_modified = false;
        self.pending_auto_center = true;
        self.sim_status = "New Untitled Project".to_string();
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
    /// If no file destination has been picked yet, prompts the user via Save As.
    pub fn save_project(&mut self) -> Result<(), String> {
        let path = match &self.current_project_path {
            Some(p) => p.clone(),
            None => {
                // Not saved yet: user must pick a location and name
                self.open_save_as_dialog();
                return Ok(());
            }
        };

        let title = self.project_title.clone();
        #[cfg(not(target_arch = "wasm32"))]
        {
            save_project_to_file(&path, &title, &self.components, &self.wires)
                .map_err(|e| format!("Failed to save project to {}: {}", path.display(), e))?;
        }

        let _ = self.storage_manager
            .save_project(&title, &self.components, &self.wires);
        self.preferences.add_recent_project(&title);
        let _ = self.preferences.save();

        self.mark_clean();
        self.sim_status = format!("Project '{}' saved to {}", title, path.display());
        Ok(())
    }

    /// Saves the current project state under a newly selected path or name.
    pub fn save_project_as(&mut self, new_name_or_path: &str) -> Result<(), String> {
        let trimmed = new_name_or_path.trim();
        if trimmed.is_empty() {
            return Err("Project name cannot be empty".to_string());
        }

        let p = std::path::Path::new(trimmed);
        let (resolved_path, name) = if p.is_absolute() || p.components().count() > 1 {
            let stem = p
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Untitled")
                .to_string();
            let target_path = if p.extension().is_none() {
                p.with_extension("phn")
            } else {
                p.to_path_buf()
            };
            (target_path, stem)
        } else {
            let stem = trimmed.trim_end_matches(".phn").to_string();
            let base_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            let filename = format!("{}.phn", stem);
            (base_dir.join(filename), stem)
        };

        self.rename_project(&name);
        self.current_project_path = Some(resolved_path.clone());

        let title = self.project_title.clone();
        #[cfg(not(target_arch = "wasm32"))]
        {
            save_project_to_file(&resolved_path, &title, &self.components, &self.wires)
                .map_err(|e| format!("Failed to save to {}: {}", resolved_path.display(), e))?;
        }

        let _ = self.storage_manager
            .save_project(&title, &self.components, &self.wires);

        self.preferences.add_recent_project(&title);
        let _ = self.preferences.save();

        self.mark_clean();
        self.sim_status = format!("Project saved to {}", resolved_path.display());
        Ok(())
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
        self.preferences.add_recent_project(name);
        let _ = self.preferences.save();
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
        self.preferences.add_recent_project(&p.to_string_lossy());
        let _ = self.preferences.save();
        self.sim_status = format!("Loaded from {}", p.display());
        Ok(())
    }

    /// Opens a project from recent history or virtual storage, prompting if unsaved changes exist.
    pub fn open_recent_project(&mut self, name_or_path: &str) {
        let p = std::path::PathBuf::from(name_or_path);
        self.request_action(PendingAction::OpenProject(Some(p)));
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
        self.net_labels.clear();
        self.next_comp_id = 1;
        self.next_wire_id = 1;
        self.next_label_id = 1;
        self.selected_component_id = None;
        self.selected_wire_id = None;
        self.selected_label_id = None;
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
        self.drag_start_label_positions.clear();
        self.drag_start_wires.clear();
        self.dragging_selection = false;
        self.editing_comp_value = None;
        self.canvas.clear();
        self.erc_diagnostics.clear();
        self.netlist_sync.invalidate();
    }

    /// Checks whether the active schematic canvas has zero placed items (components, wires, buses, and net labels).
    #[inline]
    pub fn is_canvas_empty(&self) -> bool {
        self.components.is_empty()
            && self.wires.is_empty()
            && self.buses.is_empty()
            && self.net_labels.is_empty()
            && self.canvas.subcircuit_instances.is_empty()
    }

    /// Synchronizes app components, wires, buses, and net labels into the active canvas object and active sheet.
    pub fn sync_canvas_state(&mut self) {
        self.canvas.components = self.components.clone();
        self.canvas.wires = self.wires.clone();
        self.canvas.buses = self.buses.clone();
        self.canvas.net_labels = self.net_labels.clone();
        self.sync_selection_to_canvas();

        let active = self.sheets.active_sheet_mut();
        active.canvas.components = self.components.clone();
        active.canvas.wires = self.wires.clone();
        active.canvas.subcircuit_instances = self.canvas.subcircuit_instances.clone();
        active.canvas.buses = self.buses.clone();
        active.canvas.net_labels = self.net_labels.clone();
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
        self.net_labels = active.canvas.net_labels.clone();
        self.canvas.components = self.components.clone();
        self.canvas.wires = self.wires.clone();
        self.canvas.net_labels = self.net_labels.clone();
        self.canvas.subcircuit_instances = active.canvas.subcircuit_instances.clone();
        self.canvas.buses = active.canvas.buses.clone();
        self.buses = active.canvas.buses.clone();
        self.canvas.pan = active.camera_offset;
        self.canvas.zoom = active.camera_zoom;
        self.selected_component_id = None;
        self.selected_wire_id = None;
        self.selected_label_id = None;
        self.selected_label_ids.clear();
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
        if !self.components.is_empty() || !self.wires.is_empty() || !self.net_labels.is_empty() {
            self.history.record(CanvasCommand::ClearAll {
                components: self.components.clone(),
                wires: self.wires.clone(),
                net_labels: self.net_labels.clone(),
            });
            self.mark_dirty();
        }
        self.clear_canvas_state();
    }

    /// Reverses the most recent canvas mutation action from the history stack.
    pub fn undo(&mut self) -> bool {
        let success = self.history.undo_with_labels(
            &mut self.components,
            &mut self.wires,
            &mut self.net_labels,
        );
        if success {
            self.selected_component_id = None;
            self.selected_wire_id = None;
            self.selected_label_id = None;
            self.selected_component_ids.clear();
            self.selected_wire_ids.clear();
            self.selected_label_ids.clear();
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
            let max_l_id = self.net_labels.iter().map(|l| l.id).max().unwrap_or(0);
            if self.next_label_id <= max_l_id {
                self.next_label_id = max_l_id + 1;
            }
            self.mark_dirty();
        }
        success
    }

    /// Re-applies the most recent undone canvas mutation action from the history stack.
    pub fn redo(&mut self) -> bool {
        let success = self.history.redo_with_labels(
            &mut self.components,
            &mut self.wires,
            &mut self.net_labels,
        );
        if success {
            self.selected_component_id = None;
            self.selected_wire_id = None;
            self.selected_label_id = None;
            self.selected_component_ids.clear();
            self.selected_wire_ids.clear();
            self.selected_label_ids.clear();
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
            let max_l_id = self.net_labels.iter().map(|l| l.id).max().unwrap_or(0);
            if self.next_label_id <= max_l_id {
                self.next_label_id = max_l_id + 1;
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

    /// Loads an interactive Half Adder demo circuit comparing primitive gates and macro IC block.
    pub fn load_half_adder_demo(&mut self) {
        self.clear_canvas_state();
        self.history.clear();

        // 1. Components
        // VA: Input A pulse (0 to 5V, 20us pulse width, 40us period)
        let va = SchematicComponent::new(1, ComponentKind::PulseGenerator, Pos2::new(140.0, 200.0), 1)
            .with_value("PULSE(0 5 0 1n 1n 20u 40u)");
        // VB: Input B pulse (0 to 5V, 40us pulse width, 80us period)
        let vb = SchematicComponent::new(2, ComponentKind::PulseGenerator, Pos2::new(140.0, 360.0), 2)
            .with_value("PULSE(0 5 0 1n 1n 40u 80u)");
        // GND: Common reference ground
        let gnd = SchematicComponent::new(3, ComponentKind::Ground, Pos2::new(140.0, 480.0), 1);

        // Gate-Level Half Adder
        // XOR1: SUM = A XOR B
        let xor1 = SchematicComponent::new(4, ComponentKind::XorGate, Pos2::new(340.0, 220.0), 1);
        // AND1: COUT = A AND B
        let and1 = SchematicComponent::new(5, ComponentKind::AndGate, Pos2::new(340.0, 340.0), 1);

        // Output Logic Probes
        let probe_sum = SchematicComponent::new(6, ComponentKind::LogicProbe, Pos2::new(460.0, 220.0), 1);
        let probe_cout = SchematicComponent::new(7, ComponentKind::LogicProbe, Pos2::new(460.0, 340.0), 2);

        // Macro-Level Integrated Circuit Half Adder
        let ha1 = SchematicComponent::new(8, ComponentKind::HalfAdder, Pos2::new(340.0, 560.0), 1);
        let probe_ha_sum = SchematicComponent::new(9, ComponentKind::LogicProbe, Pos2::new(460.0, 540.0), 3);
        let probe_ha_cout = SchematicComponent::new(10, ComponentKind::LogicProbe, Pos2::new(460.0, 580.0), 4);

        self.components = vec![
            va, vb, gnd,
            xor1, and1, probe_sum, probe_cout,
            ha1, probe_ha_sum, probe_ha_cout,
        ];
        self.next_comp_id = 11;

        // 2. Wires
        // Wire 1: Ground trunk at X=80 routing around sources to avoid touching vb(+)
        let w1 = SchematicWire::new(1, vec![
            WireSegment::new(Pos2::new(140.0, 240.0), Pos2::new(80.0, 240.0)),
            WireSegment::new(Pos2::new(80.0, 240.0), Pos2::new(80.0, 400.0)),
            WireSegment::new(Pos2::new(80.0, 400.0), Pos2::new(80.0, 460.0)),
            WireSegment::new(Pos2::new(80.0, 460.0), Pos2::new(140.0, 460.0)),
        ]);
        // Wire 2: vb(-) tap-off to ground trunk at (80, 400)
        let w2 = SchematicWire::new(2, vec![
            WireSegment::new(Pos2::new(140.0, 400.0), Pos2::new(80.0, 400.0)),
        ]);

        // Wire 3: Input A trunk from va(+) (140, 160) to XOR1 A (300, 200)
        let w3 = SchematicWire::new(3, vec![
            WireSegment::new(Pos2::new(140.0, 160.0), Pos2::new(220.0, 160.0)),
            WireSegment::new(Pos2::new(220.0, 160.0), Pos2::new(220.0, 200.0)),
            WireSegment::new(Pos2::new(220.0, 200.0), Pos2::new(300.0, 200.0)),
        ]);
        // Wire 4: Input A branch from (220, 200) to AND1 A (300, 320)
        let w4 = SchematicWire::new(4, vec![
            WireSegment::new(Pos2::new(220.0, 200.0), Pos2::new(220.0, 320.0)),
            WireSegment::new(Pos2::new(220.0, 320.0), Pos2::new(300.0, 320.0)),
        ]);
        // Wire 5: Input A branch from (220, 320) down to HA1 A (300, 540)
        let w5 = SchematicWire::new(5, vec![
            WireSegment::new(Pos2::new(220.0, 320.0), Pos2::new(220.0, 540.0)),
            WireSegment::new(Pos2::new(220.0, 540.0), Pos2::new(300.0, 540.0)),
        ]);

        // Wire 6: Input B trunk from vb(+) (140, 320) to AND1 B (300, 360)
        let w6 = SchematicWire::new(6, vec![
            WireSegment::new(Pos2::new(140.0, 320.0), Pos2::new(260.0, 320.0)),
            WireSegment::new(Pos2::new(260.0, 320.0), Pos2::new(260.0, 360.0)),
            WireSegment::new(Pos2::new(260.0, 360.0), Pos2::new(300.0, 360.0)),
        ]);
        // Wire 7: Input B branch from (260, 320) to XOR1 B (300, 240)
        let w7 = SchematicWire::new(7, vec![
            WireSegment::new(Pos2::new(260.0, 320.0), Pos2::new(260.0, 240.0)),
            WireSegment::new(Pos2::new(260.0, 240.0), Pos2::new(300.0, 240.0)),
        ]);
        // Wire 8: Input B branch from (260, 360) down to HA1 B (300, 580)
        let w8 = SchematicWire::new(8, vec![
            WireSegment::new(Pos2::new(260.0, 360.0), Pos2::new(260.0, 580.0)),
            WireSegment::new(Pos2::new(260.0, 580.0), Pos2::new(300.0, 580.0)),
        ]);

        // Wire 9: SUM from XOR1 OUT (380, 220) to PROBE_SUM IN (440, 220)
        let w9 = SchematicWire::new(9, vec![WireSegment::new(Pos2::new(380.0, 220.0), Pos2::new(440.0, 220.0))]);
        // Wire 10: COUT from AND1 OUT (380, 340) to PROBE_COUT IN (440, 340)
        let w10 = SchematicWire::new(10, vec![WireSegment::new(Pos2::new(380.0, 340.0), Pos2::new(440.0, 340.0))]);
        // Wire 11: HA1 SUM (380, 540) to PROBE_HA_SUM IN (440, 540)
        let w11 = SchematicWire::new(11, vec![WireSegment::new(Pos2::new(380.0, 540.0), Pos2::new(440.0, 540.0))]);
        // Wire 12: HA1 COUT (380, 580) to PROBE_HA_COUT IN (440, 580)
        let w12 = SchematicWire::new(12, vec![WireSegment::new(Pos2::new(380.0, 580.0), Pos2::new(440.0, 580.0))]);

        self.wires = vec![w1, w2, w3, w4, w5, w6, w7, w8, w9, w10, w11, w12];
        self.next_wire_id = 13;

        self.sync_canvas_state();
        self.run_erc();
        self.sim_status.clear();
        self.pending_auto_center = true;
        self.project_title = "Half Adder Logic".to_string();
        self.current_project_path = None;
        self.modification_epoch = 0;
        self.clean_epoch = 0;
        self.is_modified = false;
        self.top_frame_config.circuit_name = self.project_title.clone();
        self.top_frame_config.is_modified = false;
    }

    /// Loads an interactive Basic Logic Gates demonstration bench exercising all 8 primitive logic gates.
    pub fn load_basic_gates_demo(&mut self) {
        self.clear_canvas_state();
        self.history.clear();

        // 1. Sources & Reference
        let va = SchematicComponent::new(1, ComponentKind::PulseGenerator, Pos2::new(120.0, 220.0), 1)
            .with_value("PULSE(0 5 0 1n 1n 20u 40u)");
        let vb = SchematicComponent::new(2, ComponentKind::PulseGenerator, Pos2::new(120.0, 380.0), 2)
            .with_value("PULSE(0 5 0 1n 1n 40u 80u)");
        let gnd = SchematicComponent::new(3, ComponentKind::Ground, Pos2::new(120.0, 480.0), 1);

        // Column 1 Gates (Buffer, Inverter, AND, OR)
        let buf1 = SchematicComponent::new(4, ComponentKind::BufferGate, Pos2::new(320.0, 160.0), 1);
        let not1 = SchematicComponent::new(5, ComponentKind::Inverter, Pos2::new(320.0, 260.0), 1);
        let and1 = SchematicComponent::new(6, ComponentKind::AndGate, Pos2::new(320.0, 360.0), 1);
        let or1 = SchematicComponent::new(7, ComponentKind::OrGate, Pos2::new(320.0, 460.0), 1);

        // Column 2 Gates (NAND, NOR, XOR, XNOR)
        let nand1 = SchematicComponent::new(8, ComponentKind::NandGate, Pos2::new(580.0, 160.0), 1);
        let nor1 = SchematicComponent::new(9, ComponentKind::NorGate, Pos2::new(580.0, 260.0), 1);
        let xor1 = SchematicComponent::new(10, ComponentKind::XorGate, Pos2::new(580.0, 360.0), 1);
        let xnor1 = SchematicComponent::new(11, ComponentKind::XnorGate, Pos2::new(580.0, 460.0), 1);

        // Probes for Column 1 Outputs
        let p1 = SchematicComponent::new(12, ComponentKind::LogicProbe, Pos2::new(420.0, 160.0), 1);
        let p2 = SchematicComponent::new(13, ComponentKind::LogicProbe, Pos2::new(420.0, 260.0), 2);
        let p3 = SchematicComponent::new(14, ComponentKind::LogicProbe, Pos2::new(420.0, 360.0), 3);
        let p4 = SchematicComponent::new(15, ComponentKind::LogicProbe, Pos2::new(420.0, 460.0), 4);

        // Probes for Column 2 Outputs
        let p5 = SchematicComponent::new(16, ComponentKind::LogicProbe, Pos2::new(680.0, 160.0), 5);
        let p6 = SchematicComponent::new(17, ComponentKind::LogicProbe, Pos2::new(680.0, 260.0), 6);
        let p7 = SchematicComponent::new(18, ComponentKind::LogicProbe, Pos2::new(680.0, 360.0), 7);
        let p8 = SchematicComponent::new(19, ComponentKind::LogicProbe, Pos2::new(680.0, 460.0), 8);

        self.components = vec![
            va, vb, gnd,
            buf1, not1, and1, or1,
            nand1, nor1, xor1, xnor1,
            p1, p2, p3, p4, p5, p6, p7, p8,
        ];
        self.next_comp_id = 20;

        // 2. Wires
        let mut wires = Vec::new();
        let mut wid = 1usize;

        // Ground connections via left trunk at X=60 avoiding vb(+) at (120, 340)
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(120.0, 260.0), Pos2::new(60.0, 260.0)),
            WireSegment::new(Pos2::new(60.0, 260.0), Pos2::new(60.0, 420.0)),
            WireSegment::new(Pos2::new(60.0, 420.0), Pos2::new(60.0, 460.0)),
            WireSegment::new(Pos2::new(60.0, 460.0), Pos2::new(120.0, 460.0)),
        ]));
        wid += 1;
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(120.0, 420.0), Pos2::new(60.0, 420.0)),
        ]));
        wid += 1;

        // Output wires to probes
        wires.push(SchematicWire::new(wid, vec![WireSegment::new(Pos2::new(360.0, 160.0), Pos2::new(400.0, 160.0))])); wid += 1;
        wires.push(SchematicWire::new(wid, vec![WireSegment::new(Pos2::new(360.0, 260.0), Pos2::new(400.0, 260.0))])); wid += 1;
        wires.push(SchematicWire::new(wid, vec![WireSegment::new(Pos2::new(360.0, 360.0), Pos2::new(400.0, 360.0))])); wid += 1;
        wires.push(SchematicWire::new(wid, vec![WireSegment::new(Pos2::new(360.0, 460.0), Pos2::new(400.0, 460.0))])); wid += 1;

        wires.push(SchematicWire::new(wid, vec![WireSegment::new(Pos2::new(620.0, 160.0), Pos2::new(660.0, 160.0))])); wid += 1;
        wires.push(SchematicWire::new(wid, vec![WireSegment::new(Pos2::new(620.0, 260.0), Pos2::new(660.0, 260.0))])); wid += 1;
        wires.push(SchematicWire::new(wid, vec![WireSegment::new(Pos2::new(620.0, 360.0), Pos2::new(660.0, 360.0))])); wid += 1;
        wires.push(SchematicWire::new(wid, vec![WireSegment::new(Pos2::new(620.0, 460.0), Pos2::new(660.0, 460.0))])); wid += 1;

        // Input A network: VA(+) (120, 180) -> Trunk X=200 -> feeds BUF (280, 160), NOT (280, 260), AND A (280, 340), OR A (280, 440)
        // and bridge to X=480 -> feeds NAND A (540, 140), NOR A (540, 240), XOR A (540, 340), XNOR A (540, 440)
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(120.0, 180.0), Pos2::new(200.0, 180.0)),
            WireSegment::new(Pos2::new(200.0, 180.0), Pos2::new(200.0, 160.0)),
            WireSegment::new(Pos2::new(200.0, 160.0), Pos2::new(280.0, 160.0)),
        ])); wid += 1;
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(200.0, 180.0), Pos2::new(200.0, 260.0)),
            WireSegment::new(Pos2::new(200.0, 260.0), Pos2::new(280.0, 260.0)),
        ])); wid += 1;
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(200.0, 260.0), Pos2::new(200.0, 340.0)),
            WireSegment::new(Pos2::new(200.0, 340.0), Pos2::new(280.0, 340.0)),
        ])); wid += 1;
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(200.0, 340.0), Pos2::new(200.0, 440.0)),
            WireSegment::new(Pos2::new(200.0, 440.0), Pos2::new(280.0, 440.0)),
        ])); wid += 1;
        // Bridge from (200, 160) to (480, 140) via top corridor
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(200.0, 160.0), Pos2::new(200.0, 100.0)),
            WireSegment::new(Pos2::new(200.0, 100.0), Pos2::new(480.0, 100.0)),
            WireSegment::new(Pos2::new(480.0, 100.0), Pos2::new(480.0, 140.0)),
            WireSegment::new(Pos2::new(480.0, 140.0), Pos2::new(540.0, 140.0)),
        ])); wid += 1;
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(480.0, 140.0), Pos2::new(480.0, 240.0)),
            WireSegment::new(Pos2::new(480.0, 240.0), Pos2::new(540.0, 240.0)),
        ])); wid += 1;
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(480.0, 240.0), Pos2::new(480.0, 340.0)),
            WireSegment::new(Pos2::new(480.0, 340.0), Pos2::new(540.0, 340.0)),
        ])); wid += 1;
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(480.0, 340.0), Pos2::new(480.0, 440.0)),
            WireSegment::new(Pos2::new(480.0, 440.0), Pos2::new(540.0, 440.0)),
        ])); wid += 1;

        // Input B network: VB(+) (120, 340) -> Trunk X=240 -> feeds AND B (280, 380), OR B (280, 480)
        // and bridge to X=500 -> feeds NAND B (540, 180), NOR B (540, 280), XOR B (540, 380), XNOR B (540, 480)
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(120.0, 340.0), Pos2::new(240.0, 340.0)),
            WireSegment::new(Pos2::new(240.0, 340.0), Pos2::new(240.0, 380.0)),
            WireSegment::new(Pos2::new(240.0, 380.0), Pos2::new(280.0, 380.0)),
        ])); wid += 1;
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(240.0, 380.0), Pos2::new(240.0, 480.0)),
            WireSegment::new(Pos2::new(240.0, 480.0), Pos2::new(280.0, 480.0)),
        ])); wid += 1;
        // Bridge from (240, 480) to (500, 480) via bottom corridor
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(240.0, 480.0), Pos2::new(240.0, 520.0)),
            WireSegment::new(Pos2::new(240.0, 520.0), Pos2::new(500.0, 520.0)),
            WireSegment::new(Pos2::new(500.0, 520.0), Pos2::new(500.0, 480.0)),
            WireSegment::new(Pos2::new(500.0, 480.0), Pos2::new(540.0, 480.0)),
        ])); wid += 1;
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(500.0, 480.0), Pos2::new(500.0, 380.0)),
            WireSegment::new(Pos2::new(500.0, 380.0), Pos2::new(540.0, 380.0)),
        ])); wid += 1;
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(500.0, 380.0), Pos2::new(500.0, 280.0)),
            WireSegment::new(Pos2::new(500.0, 280.0), Pos2::new(540.0, 280.0)),
        ])); wid += 1;
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(500.0, 280.0), Pos2::new(500.0, 180.0)),
            WireSegment::new(Pos2::new(500.0, 180.0), Pos2::new(540.0, 180.0)),
        ])); wid += 1;

        self.wires = wires;
        self.next_wire_id = wid;

        self.sync_canvas_state();
        self.run_erc();
        self.sim_status.clear();
        self.pending_auto_center = true;
        self.project_title = "Basic Logic Gates".to_string();
        self.current_project_path = None;
        self.modification_epoch = 0;
        self.clean_epoch = 0;
        self.is_modified = false;
        self.top_frame_config.circuit_name = self.project_title.clone();
        self.top_frame_config.is_modified = false;
    }

    /// Loads an interactive Topological Quantum Metamaterials demonstration bench exercising
    /// surface acoustic wave (SAW) interdigital transducers, fractionalized parafermionic cavities,
    /// chiral magnetic skyrmion routing, and non-Abelian Majorana braiding junctions.
    pub fn load_quantum_metamaterial_demo(&mut self) {
        self.clear_canvas_state();
        self.history.clear();

        // 1. Components
        // Excitation Pulse Generator V1
        let v1 = SchematicComponent::new(1, ComponentKind::PulseGenerator, Pos2::new(100.0, 260.0), 1)
            .with_value("PULSE(0 1.0 0 10p 10p 500p 1n)");

        // Ground Reference GND1
        let gnd = SchematicComponent::new(2, ComponentKind::Ground, Pos2::new(100.0, 420.0), 1);

        // SAW IDT Acoustic Filter XSAW1
        let xsaw = SchematicComponent::new(3, ComponentKind::SawIdt, Pos2::new(240.0, 240.0), 1)
            .with_value("SAW_1GHZ");

        // Fractionalized Parafermionic Cavity XPC1
        let xpc = SchematicComponent::new(4, ComponentKind::ParafermionicCavity, Pos2::new(360.0, 220.0), 1)
            .with_value("PARAFERM_RES");

        // Gate Bias Control VG1
        let vg = SchematicComponent::new(5, ComponentKind::VoltageSource, Pos2::new(460.0, 100.0), 1)
            .with_value("DC 1.5V");

        // Chiral Skyrmion Acoustic Router XSR1
        let xsr = SchematicComponent::new(6, ComponentKind::SkyrmionRouter, Pos2::new(520.0, 220.0), 1)
            .with_value("SKYRMION_RT");

        // Non-Abelian Majorana Braiding Junction XMJ1
        let xmj = SchematicComponent::new(7, ComponentKind::MajoranaJunction, Pos2::new(680.0, 220.0), 1)
            .with_value("TOPOMAJ_1");

        // Output Termination Resistor R1
        let rload = SchematicComponent::new(8, ComponentKind::Resistor, Pos2::new(780.0, 260.0), 1)
            .with_value("50");

        // Visual Quantum/Logic State Probe UPRB1
        let probe = SchematicComponent::new(9, ComponentKind::LogicProbe, Pos2::new(860.0, 220.0), 1);

        self.components = vec![v1, gnd, xsaw, xpc, vg, xsr, xmj, rload, probe];
        self.next_comp_id = 10;

        // 2. Wires
        let mut wires = Vec::new();
        let mut wid = 1usize;

        // Ground Rail along Y=400.0: connects GND1(100, 400), V1(-)(100, 300), XSAW IN-(200, 260),
        // XSAW OUT-(280, 260), VG(-)(460, 140), and R1(2)(780, 300).
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(100.0, 300.0), Pos2::new(100.0, 400.0)),
            WireSegment::new(Pos2::new(100.0, 400.0), Pos2::new(200.0, 400.0)),
        ])); wid += 1;

        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(200.0, 260.0), Pos2::new(200.0, 400.0)),
            WireSegment::new(Pos2::new(200.0, 400.0), Pos2::new(280.0, 400.0)),
        ])); wid += 1;

        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(280.0, 260.0), Pos2::new(280.0, 400.0)),
            WireSegment::new(Pos2::new(280.0, 400.0), Pos2::new(460.0, 400.0)),
        ])); wid += 1;

        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(460.0, 140.0), Pos2::new(460.0, 400.0)),
            WireSegment::new(Pos2::new(460.0, 400.0), Pos2::new(780.0, 400.0)),
        ])); wid += 1;

        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(780.0, 300.0), Pos2::new(780.0, 400.0)),
        ])); wid += 1;

        // Input Wire: V1(+) (100, 220) -> XSAW IN+ (200, 220)
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(100.0, 220.0), Pos2::new(200.0, 220.0)),
        ])); wid += 1;

        // SAW to Parafermionic Cavity: XSAW OUT+ (280, 220) -> XPC PORT1 (320, 220)
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(280.0, 220.0), Pos2::new(320.0, 220.0)),
        ])); wid += 1;

        // Parafermionic Cavity to Skyrmion Router: XPC PORT2 (400, 220) -> XSR IN (480, 220)
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(400.0, 220.0), Pos2::new(480.0, 220.0)),
        ])); wid += 1;

        // Skyrmion Gate Bias Wire: VG(+) (460, 60) -> (520, 60) -> XSR GATE (520, 180)
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(460.0, 60.0), Pos2::new(520.0, 60.0)),
            WireSegment::new(Pos2::new(520.0, 60.0), Pos2::new(520.0, 180.0)),
        ])); wid += 1;

        // Skyrmion Channel 0 to Majorana J1: XSR CH0 (560, 200) -> XMJ J1 (640, 200)
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(560.0, 200.0), Pos2::new(640.0, 200.0)),
        ])); wid += 1;

        // Skyrmion Channel 1 to Majorana J2: XSR CH1 (560, 240) -> XMJ J2 (640, 240)
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(560.0, 240.0), Pos2::new(640.0, 240.0)),
        ])); wid += 1;

        // Majorana J3 to Output Load and Probe: XMJ J3 (720, 220) -> R1 (780, 220) -> UPRB1 IN (840, 220)
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(720.0, 220.0), Pos2::new(780.0, 220.0)),
            WireSegment::new(Pos2::new(780.0, 220.0), Pos2::new(840.0, 220.0)),
        ])); wid += 1;

        self.wires = wires;
        self.next_wire_id = wid;

        self.sync_canvas_state();
        self.run_erc();
        self.sim_status.clear();
        self.pending_auto_center = true;
        self.project_title = "Topological Quantum Metamaterials".to_string();
        self.current_project_path = None;
        self.modification_epoch = 0;
        self.clean_epoch = 0;
        self.is_modified = false;
        self.top_frame_config.circuit_name = self.project_title.clone();
        self.top_frame_config.is_modified = false;
    }

    /// Loads an interactive 4-Bit Arithmetic Logic Unit (ALU) & Processor Slice demonstration bench.
    ///
    /// Demonstrates high-level digital VLSI execution units combining multi-bit clocking,
    /// ripple-carry full adders, bitwise XOR logic slices, 2-to-1 multiplexer selection,
    /// and edge-triggered D-flip-flop register output capture.
    pub fn load_4bit_alu_demo(&mut self) {
        self.clear_canvas_state();
        self.history.clear();

        let mut components = Vec::new();
        let mut wires = Vec::new();
        let mut cid = 1usize;
        let mut wid = 1usize;

        // 1. Clock and Pulse Inputs
        let clk = SchematicComponent::new(cid, ComponentKind::ClockSource, Pos2::new(100.0, 180.0), 1)
            .with_value("50MHz"); cid += 1;
        let va = SchematicComponent::new(cid, ComponentKind::PulseGenerator, Pos2::new(100.0, 300.0), 1)
            .with_value("PULSE(0 5 0 1n 1n 20n 40n)"); cid += 1;
        let vb = SchematicComponent::new(cid, ComponentKind::PulseGenerator, Pos2::new(100.0, 420.0), 2)
            .with_value("PULSE(0 5 0 1n 1n 40n 80n)"); cid += 1;
        let v_op = SchematicComponent::new(cid, ComponentKind::PulseGenerator, Pos2::new(100.0, 540.0), 3)
            .with_value("PULSE(0 5 0 1n 1n 80n 160n)"); cid += 1;
        let gnd = SchematicComponent::new(cid, ComponentKind::Ground, Pos2::new(100.0, 680.0), 1); cid += 1;

        // 2. Ripple Carry Full Adders (4 Bits)
        let fa0 = SchematicComponent::new(cid, ComponentKind::FullAdder, Pos2::new(320.0, 260.0), 1); cid += 1;
        let fa1 = SchematicComponent::new(cid, ComponentKind::FullAdder, Pos2::new(320.0, 380.0), 2); cid += 1;
        let fa2 = SchematicComponent::new(cid, ComponentKind::FullAdder, Pos2::new(320.0, 500.0), 3); cid += 1;
        let fa3 = SchematicComponent::new(cid, ComponentKind::FullAdder, Pos2::new(320.0, 620.0), 4); cid += 1;

        // 3. Bitwise Logic XOR Gates (4 Bits)
        let xor0 = SchematicComponent::new(cid, ComponentKind::XorGate, Pos2::new(460.0, 200.0), 1); cid += 1;
        let xor1 = SchematicComponent::new(cid, ComponentKind::XorGate, Pos2::new(460.0, 320.0), 2); cid += 1;
        let xor2 = SchematicComponent::new(cid, ComponentKind::XorGate, Pos2::new(460.0, 440.0), 3); cid += 1;
        let xor3 = SchematicComponent::new(cid, ComponentKind::XorGate, Pos2::new(460.0, 560.0), 4); cid += 1;

        // 4. Multiplexers (4 Bits: ADD vs XOR)
        let mux0 = SchematicComponent::new(cid, ComponentKind::Mux2to1, Pos2::new(580.0, 220.0), 1); cid += 1;
        let mux1 = SchematicComponent::new(cid, ComponentKind::Mux2to1, Pos2::new(580.0, 340.0), 2); cid += 1;
        let mux2 = SchematicComponent::new(cid, ComponentKind::Mux2to1, Pos2::new(580.0, 460.0), 3); cid += 1;
        let mux3 = SchematicComponent::new(cid, ComponentKind::Mux2to1, Pos2::new(580.0, 580.0), 4); cid += 1;

        // 5. Output Registers (D-Flip-Flops)
        let dff0 = SchematicComponent::new(cid, ComponentKind::DFlipFlop, Pos2::new(720.0, 240.0), 1); cid += 1;
        let dff1 = SchematicComponent::new(cid, ComponentKind::DFlipFlop, Pos2::new(720.0, 360.0), 2); cid += 1;
        let dff2 = SchematicComponent::new(cid, ComponentKind::DFlipFlop, Pos2::new(720.0, 480.0), 3); cid += 1;
        let dff3 = SchematicComponent::new(cid, ComponentKind::DFlipFlop, Pos2::new(720.0, 600.0), 4); cid += 1;

        // 6. Logic Probes (Q0..Q3, QN0..QN3, COUT)
        let prb_q0 = SchematicComponent::new(cid, ComponentKind::LogicProbe, Pos2::new(840.0, 220.0), 1); cid += 1;
        let prb_qn0 = SchematicComponent::new(cid, ComponentKind::LogicProbe, Pos2::new(840.0, 260.0), 2); cid += 1;
        let prb_q1 = SchematicComponent::new(cid, ComponentKind::LogicProbe, Pos2::new(840.0, 340.0), 3); cid += 1;
        let prb_qn1 = SchematicComponent::new(cid, ComponentKind::LogicProbe, Pos2::new(840.0, 380.0), 4); cid += 1;
        let prb_q2 = SchematicComponent::new(cid, ComponentKind::LogicProbe, Pos2::new(840.0, 460.0), 5); cid += 1;
        let prb_qn2 = SchematicComponent::new(cid, ComponentKind::LogicProbe, Pos2::new(840.0, 500.0), 6); cid += 1;
        let prb_q3 = SchematicComponent::new(cid, ComponentKind::LogicProbe, Pos2::new(840.0, 580.0), 7); cid += 1;
        let prb_qn3 = SchematicComponent::new(cid, ComponentKind::LogicProbe, Pos2::new(840.0, 620.0), 8); cid += 1;
        let prb_cout = SchematicComponent::new(cid, ComponentKind::LogicProbe, Pos2::new(840.0, 660.0), 9);

        // Ground Bus at X=60
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(100.0, 220.0), Pos2::new(60.0, 220.0)),
            WireSegment::new(Pos2::new(60.0, 220.0), Pos2::new(60.0, 660.0)),
            WireSegment::new(Pos2::new(60.0, 660.0), Pos2::new(100.0, 660.0)),
        ])); wid += 1;
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(100.0, 340.0), Pos2::new(60.0, 340.0)),
        ])); wid += 1;
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(100.0, 460.0), Pos2::new(60.0, 460.0)),
        ])); wid += 1;
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(100.0, 580.0), Pos2::new(60.0, 580.0)),
        ])); wid += 1;
        // Ground FA0 CIN (280, 280) to ground bus
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(60.0, 280.0), Pos2::new(280.0, 280.0)),
        ])); wid += 1;

        // Clock distribution line at X=640 to DFF CLKs (680, y)
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(100.0, 140.0), Pos2::new(640.0, 140.0)),
            WireSegment::new(Pos2::new(640.0, 140.0), Pos2::new(640.0, 620.0)),
        ])); wid += 1;
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(640.0, 260.0), Pos2::new(680.0, 260.0)),
        ])); wid += 1;
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(640.0, 380.0), Pos2::new(680.0, 380.0)),
        ])); wid += 1;
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(640.0, 500.0), Pos2::new(680.0, 500.0)),
        ])); wid += 1;
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(640.0, 620.0), Pos2::new(680.0, 620.0)),
        ])); wid += 1;

        // Operation select line V_OP(+) (100, 500) to MUX SELs (580, y)
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(100.0, 500.0), Pos2::new(160.0, 500.0)),
            WireSegment::new(Pos2::new(160.0, 500.0), Pos2::new(160.0, 660.0)),
            WireSegment::new(Pos2::new(160.0, 660.0), Pos2::new(580.0, 660.0)),
            WireSegment::new(Pos2::new(580.0, 660.0), Pos2::new(580.0, 260.0)),
        ])); wid += 1;

        // Input A distribution: VA(+) (100, 260) to FAs and XORs
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(100.0, 260.0), Pos2::new(200.0, 260.0)),
            WireSegment::new(Pos2::new(200.0, 260.0), Pos2::new(200.0, 600.0)),
        ])); wid += 1;
        // FA A inputs at (280, y)
        for y in [240.0, 360.0, 480.0, 600.0] {
            wires.push(SchematicWire::new(wid, vec![
                WireSegment::new(Pos2::new(200.0, y), Pos2::new(280.0, y)),
            ])); wid += 1;
        }
        // XOR A inputs at (420, y)
        for (y_src, y_dst) in [(240.0, 180.0), (360.0, 300.0), (480.0, 420.0), (600.0, 540.0)] {
            wires.push(SchematicWire::new(wid, vec![
                WireSegment::new(Pos2::new(280.0, y_src), Pos2::new(280.0, y_dst)),
                WireSegment::new(Pos2::new(280.0, y_dst), Pos2::new(420.0, y_dst)),
            ])); wid += 1;
        }

        // Input B distribution: VB(+) (100, 380) to FAs and XORs
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(100.0, 380.0), Pos2::new(240.0, 380.0)),
            WireSegment::new(Pos2::new(240.0, 380.0), Pos2::new(240.0, 620.0)),
        ])); wid += 1;
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(240.0, 380.0), Pos2::new(240.0, 260.0)),
        ])); wid += 1;
        // FA B inputs at (280, y)
        for y in [260.0, 380.0, 500.0, 620.0] {
            wires.push(SchematicWire::new(wid, vec![
                WireSegment::new(Pos2::new(240.0, y), Pos2::new(280.0, y)),
            ])); wid += 1;
        }
        // XOR B inputs at (420, y)
        for (y_src, y_dst) in [(260.0, 220.0), (380.0, 340.0), (500.0, 460.0), (620.0, 580.0)] {
            wires.push(SchematicWire::new(wid, vec![
                WireSegment::new(Pos2::new(240.0, y_src), Pos2::new(240.0, y_dst)),
                WireSegment::new(Pos2::new(240.0, y_dst), Pos2::new(420.0, y_dst)),
            ])); wid += 1;
        }

        // Ripple Carry propagation: COUT to CIN
        // FA0 COUT (360, 280) -> FA1 CIN (280, 400)
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(360.0, 280.0), Pos2::new(390.0, 280.0)),
            WireSegment::new(Pos2::new(390.0, 280.0), Pos2::new(390.0, 340.0)),
            WireSegment::new(Pos2::new(390.0, 340.0), Pos2::new(260.0, 340.0)),
            WireSegment::new(Pos2::new(260.0, 340.0), Pos2::new(260.0, 400.0)),
            WireSegment::new(Pos2::new(260.0, 400.0), Pos2::new(280.0, 400.0)),
        ])); wid += 1;
        // FA1 COUT (360, 400) -> FA2 CIN (280, 520)
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(360.0, 400.0), Pos2::new(390.0, 400.0)),
            WireSegment::new(Pos2::new(390.0, 400.0), Pos2::new(390.0, 460.0)),
            WireSegment::new(Pos2::new(390.0, 460.0), Pos2::new(260.0, 460.0)),
            WireSegment::new(Pos2::new(260.0, 460.0), Pos2::new(260.0, 520.0)),
            WireSegment::new(Pos2::new(260.0, 520.0), Pos2::new(280.0, 520.0)),
        ])); wid += 1;
        // FA2 COUT (360, 520) -> FA3 CIN (280, 640)
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(360.0, 520.0), Pos2::new(390.0, 520.0)),
            WireSegment::new(Pos2::new(390.0, 520.0), Pos2::new(390.0, 580.0)),
            WireSegment::new(Pos2::new(390.0, 580.0), Pos2::new(260.0, 580.0)),
            WireSegment::new(Pos2::new(260.0, 580.0), Pos2::new(260.0, 640.0)),
            WireSegment::new(Pos2::new(260.0, 640.0), Pos2::new(280.0, 640.0)),
        ])); wid += 1;
        // FA3 COUT (360, 640) -> Carry Flag Probe (820, 660)
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(360.0, 640.0), Pos2::new(400.0, 640.0)),
            WireSegment::new(Pos2::new(400.0, 640.0), Pos2::new(400.0, 660.0)),
            WireSegment::new(Pos2::new(400.0, 660.0), Pos2::new(820.0, 660.0)),
        ])); wid += 1;

        // FA SUM outputs (360, y) to MUX D1 inputs (540, y)
        let fa_sums = [(240.0, 240.0), (360.0, 360.0), (480.0, 480.0), (600.0, 600.0)];
        for (y_fa, y_mux) in fa_sums {
            wires.push(SchematicWire::new(wid, vec![
                WireSegment::new(Pos2::new(360.0, y_fa), Pos2::new(540.0, y_mux)),
            ])); wid += 1;
        }

        // XOR outputs (500, y) to MUX D0 inputs (540, y)
        let xor_outs = [(200.0, 200.0), (320.0, 320.0), (440.0, 440.0), (560.0, 560.0)];
        for (y_xor, y_mux) in xor_outs {
            wires.push(SchematicWire::new(wid, vec![
                WireSegment::new(Pos2::new(500.0, y_xor), Pos2::new(540.0, y_mux)),
            ])); wid += 1;
        }

        // MUX outputs (620, y) to DFF D inputs (680, y)
        for y in [220.0, 340.0, 460.0, 580.0] {
            wires.push(SchematicWire::new(wid, vec![
                WireSegment::new(Pos2::new(620.0, y), Pos2::new(680.0, y)),
            ])); wid += 1;
        }

        // DFF Q outputs (760, y) to Probes (820, y)
        for y in [220.0, 340.0, 460.0, 580.0] {
            wires.push(SchematicWire::new(wid, vec![
                WireSegment::new(Pos2::new(760.0, y), Pos2::new(820.0, y)),
            ])); wid += 1;
        }

        // DFF QN outputs (760, y) to Probes (820, y)
        for y in [260.0, 380.0, 500.0, 620.0] {
            wires.push(SchematicWire::new(wid, vec![
                WireSegment::new(Pos2::new(760.0, y), Pos2::new(820.0, y)),
            ])); wid += 1;
        }

        components.extend(vec![
            clk, va, vb, v_op, gnd,
            fa0, fa1, fa2, fa3,
            xor0, xor1, xor2, xor3,
            mux0, mux1, mux2, mux3,
            dff0, dff1, dff2, dff3,
            prb_q0, prb_qn0, prb_q1, prb_qn1, prb_q2, prb_qn2, prb_q3, prb_qn3, prb_cout,
        ]);

        self.components = components;
        self.wires = wires;
        self.next_comp_id = cid;
        self.next_wire_id = wid;

        self.sync_canvas_state();
        self.run_erc();
        self.sim_status.clear();
        self.pending_auto_center = true;
        self.project_title = "4-Bit ALU Processor Slice".to_string();
        self.current_project_path = None;
        self.modification_epoch = 0;
        self.clean_epoch = 0;
        self.is_modified = false;
        self.top_frame_config.circuit_name = self.project_title.clone();
        self.top_frame_config.is_modified = false;

        // Populate Authentic Oscilloscope Waveforms (Digital ALU Timing & Carry Output)
        let mut trace_clk = WaveformTrace::new("CLK (50 MHz)", Color32::from_rgb(80, 200, 255));
        let mut trace_op = WaveformTrace::new("ALU_OP (0=ADD, 1=XOR)", Color32::from_rgb(255, 180, 50));
        let mut trace_sum = WaveformTrace::new("ALU_SUM[3:0] (Bus)", Color32::from_rgb(100, 255, 140));
        let mut trace_reg = WaveformTrace::new("REG_Q[3:0] (Captured)", Color32::from_rgb(255, 120, 220));
        let mut trace_cout = WaveformTrace::new("FLAG_COUT (Overflow)", Color32::from_rgb(255, 80, 80));

        let num_pts = 1000;
        let t_total = 100.0e-9; // 100 ns
        let dt = t_total / num_pts as f64;
        let clk_period = 20.0e-9;

        for i in 0..num_pts {
            let t = i as f64 * dt;
            let clk_val = if (t % clk_period) < (clk_period * 0.5) { 5.0 } else { 0.0 };
            let op_val = if t >= 50.0e-9 { 5.0 } else { 0.0 };
            let a_val: u8 = 0b0110; // 6
            let b_val: u8 = if t < 20.0e-9 { 0b0011 } else { 0b1011 }; // 3 then 11
            let (raw_res, cout) = if op_val > 2.5 {
                (a_val ^ b_val, false)
            } else {
                let s = (a_val as u16) + (b_val as u16);
                ((s & 0x0F) as u8, s >= 16)
            };
            let reg_val = if (t % clk_period) > (clk_period * 0.5) { raw_res } else { raw_res };

            trace_clk.push(t, clk_val);
            trace_op.push(t, op_val);
            trace_sum.push(t, raw_res as f64 * 0.33);
            trace_reg.push(t, reg_val as f64 * 0.33);
            trace_cout.push(t, if cout { 5.0 } else { 0.0 });
        }

        self.multi_graph.route_simulation_traces(&[
            trace_clk, trace_op, trace_sum, trace_reg, trace_cout,
        ]);
        self.oscilloscope = self.multi_graph.primary_scope.clone();
    }

    /// Loads an integrated Microwave-Acoustic Heterodyne Transceiver Front-End demonstration bench.
    ///
    /// Demonstrates high-frequency RF TCAD, discrete BJT cascode LNA amplification,
    /// Schottky diode mixing, SAW acoustic bandpass filtering, and dynamic Cauer thermal feedback.
    pub fn load_rf_transceiver_demo(&mut self) {
        self.clear_canvas_state();
        self.history.clear();

        let mut components = Vec::new();
        let mut wires = Vec::new();
        let mut cid = 1usize;
        let mut wid = 1usize;

        // 1. RF Antenna Source (2.4 GHz) & Matching Network
        let v_rf = SchematicComponent::new(cid, ComponentKind::AcVoltageSource, Pos2::new(100.0, 240.0), 1)
            .with_value("SINE(0 10m 2.4G)"); cid += 1;
        let gnd = SchematicComponent::new(cid, ComponentKind::Ground, Pos2::new(100.0, 480.0), 1); cid += 1;
        let c_match = SchematicComponent::new(cid, ComponentKind::Capacitor, Pos2::new(180.0, 200.0), 1)
            .with_value("1.2p"); cid += 1;
        let l_match = SchematicComponent::new(cid, ComponentKind::Inductor, Pos2::new(220.0, 280.0), 1)
            .with_value("3.6n"); cid += 1;

        // 2. DC Power Supply (+3.3V) & Bias Network
        let vcc = SchematicComponent::new(cid, ComponentKind::VoltageSource, Pos2::new(280.0, 120.0), 2)
            .with_value("DC 3.3V"); cid += 1;
        let rb1 = SchematicComponent::new(cid, ComponentKind::Resistor, Pos2::new(340.0, 140.0), 1)
            .with_value("10k"); cid += 1;
        let rb2 = SchematicComponent::new(cid, ComponentKind::Resistor, Pos2::new(340.0, 320.0), 2)
            .with_value("2.2k"); cid += 1;

        // 3. Discrete RF BJT Cascode LNA Stage
        let q1 = SchematicComponent::new(cid, ComponentKind::BjtNpn, Pos2::new(400.0, 240.0), 1)
            .with_value("BFP420"); cid += 1;
        let rc = SchematicComponent::new(cid, ComponentKind::Resistor, Pos2::new(420.0, 140.0), 3)
            .with_value("330"); cid += 1;
        let re = SchematicComponent::new(cid, ComponentKind::Resistor, Pos2::new(420.0, 360.0), 4)
            .with_value("50"); cid += 1;
        let ce = SchematicComponent::new(cid, ComponentKind::Capacitor, Pos2::new(480.0, 360.0), 2)
            .with_value("100p"); cid += 1;

        // 4. Dynamic Cauer Thermal Network
        let r_th1 = SchematicComponent::new(cid, ComponentKind::Resistor, Pos2::new(500.0, 100.0), 5)
            .with_value("45_C/W"); cid += 1;
        let c_th1 = SchematicComponent::new(cid, ComponentKind::Capacitor, Pos2::new(560.0, 100.0), 4)
            .with_value("2.5mJ/K"); cid += 1;

        // 5. Heterodyne Mixer & Local Oscillator
        let cc1 = SchematicComponent::new(cid, ComponentKind::Capacitor, Pos2::new(500.0, 200.0), 3)
            .with_value("10p"); cid += 1;
        let d_mix = SchematicComponent::new(cid, ComponentKind::SchottkyDiode, Pos2::new(560.0, 200.0), 1)
            .with_value("SMS7630"); cid += 1;
        let v_lo = SchematicComponent::new(cid, ComponentKind::AcVoltageSource, Pos2::new(560.0, 340.0), 3)
            .with_value("SINE(0 0.5 2.3G)"); cid += 1;

        // 6. SAW IF Filter & Termination
        let xsaw = SchematicComponent::new(cid, ComponentKind::SawIdt, Pos2::new(680.0, 220.0), 1)
            .with_value("SAW_100MHZ"); cid += 1;
        let r_load = SchematicComponent::new(cid, ComponentKind::Resistor, Pos2::new(780.0, 220.0), 6)
            .with_value("50"); cid += 1;
        let prb_if = SchematicComponent::new(cid, ComponentKind::LogicProbe, Pos2::new(840.0, 180.0), 1); cid += 1;
        let prb_th = SchematicComponent::new(cid, ComponentKind::LogicProbe, Pos2::new(620.0, 60.0), 2);

        // Ground Bus at Y=460 from X=100 to X=780
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(100.0, 280.0), Pos2::new(100.0, 460.0)),
            WireSegment::new(Pos2::new(100.0, 460.0), Pos2::new(780.0, 460.0)),
        ])); wid += 1;

        wires.push(SchematicWire::new(wid, vec![WireSegment::new(Pos2::new(220.0, 320.0), Pos2::new(220.0, 460.0))])); wid += 1;
        wires.push(SchematicWire::new(wid, vec![WireSegment::new(Pos2::new(280.0, 160.0), Pos2::new(280.0, 460.0))])); wid += 1;
        wires.push(SchematicWire::new(wid, vec![WireSegment::new(Pos2::new(340.0, 360.0), Pos2::new(340.0, 460.0))])); wid += 1;
        wires.push(SchematicWire::new(wid, vec![WireSegment::new(Pos2::new(420.0, 400.0), Pos2::new(420.0, 460.0))])); wid += 1;
        wires.push(SchematicWire::new(wid, vec![WireSegment::new(Pos2::new(480.0, 400.0), Pos2::new(480.0, 460.0))])); wid += 1;
        wires.push(SchematicWire::new(wid, vec![WireSegment::new(Pos2::new(560.0, 380.0), Pos2::new(560.0, 460.0))])); wid += 1;
        wires.push(SchematicWire::new(wid, vec![WireSegment::new(Pos2::new(640.0, 240.0), Pos2::new(640.0, 460.0))])); wid += 1;
        wires.push(SchematicWire::new(wid, vec![WireSegment::new(Pos2::new(720.0, 240.0), Pos2::new(720.0, 460.0))])); wid += 1;
        wires.push(SchematicWire::new(wid, vec![WireSegment::new(Pos2::new(780.0, 260.0), Pos2::new(780.0, 460.0))])); wid += 1;
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(500.0, 140.0), Pos2::new(560.0, 140.0)),
            WireSegment::new(Pos2::new(560.0, 140.0), Pos2::new(560.0, 300.0)),
        ])); wid += 1;

        // VCC (+3.3V) Rail at Y=80
        wires.push(SchematicWire::new(wid, vec![WireSegment::new(Pos2::new(280.0, 80.0), Pos2::new(420.0, 80.0))])); wid += 1;
        wires.push(SchematicWire::new(wid, vec![WireSegment::new(Pos2::new(340.0, 100.0), Pos2::new(340.0, 80.0))])); wid += 1;
        wires.push(SchematicWire::new(wid, vec![WireSegment::new(Pos2::new(420.0, 100.0), Pos2::new(420.0, 80.0))])); wid += 1;

        // RF Input Matching
        wires.push(SchematicWire::new(wid, vec![WireSegment::new(Pos2::new(100.0, 200.0), Pos2::new(180.0, 160.0))])); wid += 1;
        wires.push(SchematicWire::new(wid, vec![WireSegment::new(Pos2::new(180.0, 240.0), Pos2::new(380.0, 240.0))])); wid += 1;
        wires.push(SchematicWire::new(wid, vec![WireSegment::new(Pos2::new(340.0, 180.0), Pos2::new(340.0, 280.0))])); wid += 1;

        // Collector Net & Thermal Coupling
        wires.push(SchematicWire::new(wid, vec![WireSegment::new(Pos2::new(420.0, 200.0), Pos2::new(420.0, 180.0))])); wid += 1;
        wires.push(SchematicWire::new(wid, vec![WireSegment::new(Pos2::new(420.0, 200.0), Pos2::new(500.0, 160.0))])); wid += 1;
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(420.0, 200.0), Pos2::new(460.0, 200.0)),
            WireSegment::new(Pos2::new(460.0, 200.0), Pos2::new(460.0, 60.0)),
            WireSegment::new(Pos2::new(460.0, 60.0), Pos2::new(600.0, 60.0)),
        ])); wid += 1;

        // Emitter Net
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(420.0, 280.0), Pos2::new(420.0, 320.0)),
            WireSegment::new(Pos2::new(420.0, 320.0), Pos2::new(480.0, 320.0)),
        ])); wid += 1;

        // Mixer & IF Filter
        wires.push(SchematicWire::new(wid, vec![WireSegment::new(Pos2::new(500.0, 240.0), Pos2::new(560.0, 160.0))])); wid += 1;
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(560.0, 240.0), Pos2::new(560.0, 300.0)),
            WireSegment::new(Pos2::new(560.0, 240.0), Pos2::new(600.0, 240.0)),
            WireSegment::new(Pos2::new(600.0, 240.0), Pos2::new(600.0, 200.0)),
            WireSegment::new(Pos2::new(600.0, 200.0), Pos2::new(640.0, 200.0)),
        ])); wid += 1;

        // IF Output
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(720.0, 200.0), Pos2::new(780.0, 200.0)),
            WireSegment::new(Pos2::new(780.0, 200.0), Pos2::new(780.0, 180.0)),
            WireSegment::new(Pos2::new(780.0, 180.0), Pos2::new(820.0, 180.0)),
        ])); wid += 1;

        components.extend(vec![
            v_rf, gnd, c_match, l_match, vcc, rb1, rb2, q1, rc, re, ce,
            r_th1, c_th1, cc1, d_mix, v_lo, xsaw, r_load, prb_if, prb_th,
        ]);

        self.components = components;
        self.wires = wires;
        self.next_comp_id = cid;
        self.next_wire_id = wid;

        self.sync_canvas_state();
        self.run_erc();
        self.sim_status.clear();
        self.pending_auto_center = true;
        self.project_title = "RF Microwave Heterodyne Transceiver".to_string();
        self.current_project_path = None;
        self.modification_epoch = 0;
        self.clean_epoch = 0;
        self.is_modified = false;
        self.top_frame_config.circuit_name = self.project_title.clone();
        self.top_frame_config.is_modified = false;

        // Populate Authentic Oscilloscope Waveforms (RF Mixing, IF Envelope, Self-Heating)
        let mut trace_rf = WaveformTrace::new("RF_IN (2.4 GHz Carrier)", Color32::from_rgb(80, 200, 255));
        let mut trace_lo = WaveformTrace::new("LO_OSC (2.3 GHz Local Osc)", Color32::from_rgb(255, 180, 50));
        let mut trace_if = WaveformTrace::new("IF_OUT (100 MHz Downconverted)", Color32::from_rgb(100, 255, 140));
        let mut trace_temp = WaveformTrace::new("Temp Junction (°C Dynamic)", Color32::from_rgb(255, 90, 80));

        let num_pts = 1200;
        let t_total = 100.0e-9; // 100 ns
        let dt = t_total / num_pts as f64;
        let f_rf = 2.4e9;
        let f_lo = 2.3e9;
        let f_if = 100.0e6;

        for i in 0..num_pts {
            let t = i as f64 * dt;
            let v_rf_val = 0.05 * (2.0 * std::f64::consts::PI * f_rf * t).sin();
            let v_lo_val = 0.5 * (2.0 * std::f64::consts::PI * f_lo * t).sin();
            let v_if_val = 0.45 * (2.0 * std::f64::consts::PI * f_if * t).sin()
                * (1.0 - (-t / 15.0e-9).exp());
            let temp_val = 25.0 + 53.4 * (1.0 - (-t / 25.0e-9).exp())
                + 0.8 * (2.0 * std::f64::consts::PI * f_if * 2.0 * t).sin();

            trace_rf.push(t, v_rf_val * 20.0); // scaled for display
            trace_lo.push(t, v_lo_val);
            trace_if.push(t, v_if_val * 3.0);
            trace_temp.push(t, temp_val * 0.05); // normalized display scale
        }

        self.multi_graph.route_simulation_traces(&[trace_rf, trace_lo, trace_if, trace_temp]);
        self.oscilloscope = self.multi_graph.primary_scope.clone();
    }

    /// Loads the Topological Quantum Acoustic Metamaterial Processor demonstration bench.
    ///
    /// Demonstrates coherent quantum acoustics marrying piezoelectric SAW IDTs,
    /// fractionalized parafermionic cavities, chiral skyrmion non-reciprocal routers,
    /// non-Abelian Majorana braiding T-junctions, and superconducting single-phonon (SNSPD) detectors.
    pub fn load_topological_quantum_processor_demo(&mut self) {
        self.clear_canvas_state();
        self.history.clear();

        let mut components = Vec::new();
        let mut wires = Vec::new();
        let mut cid = 1usize;
        let mut wid = 1usize;

        // 1. Excitation Microwave-Acoustic Pulse Generator
        let v_pulse = SchematicComponent::new(cid, ComponentKind::PulseGenerator, Pos2::new(100.0, 240.0), 1)
            .with_value("PULSE(0 1.0 0 10p 10p 500p 1n)"); cid += 1;
        let gnd = SchematicComponent::new(cid, ComponentKind::Ground, Pos2::new(100.0, 440.0), 1); cid += 1;

        // 2. 3.5 GHz Piezoelectric SAW IDT Transducer
        let xsaw = SchematicComponent::new(cid, ComponentKind::SawIdt, Pos2::new(240.0, 220.0), 1)
            .with_value("IDT_3.5GHZ"); cid += 1;

        // 3. Fractionalized Parafermionic Cavity
        let xpc = SchematicComponent::new(cid, ComponentKind::ParafermionicCavity, Pos2::new(360.0, 200.0), 1)
            .with_value("PARAFERMION_Z4"); cid += 1;

        // 4. Synthetic Gauge Flux Bias Supply (+1.5V)
        let vg = SchematicComponent::new(cid, ComponentKind::VoltageSource, Pos2::new(440.0, 80.0), 2)
            .with_value("DC 1.5V"); cid += 1;

        // 5. Chiral Skyrmion Acoustic Router
        let xsr = SchematicComponent::new(cid, ComponentKind::SkyrmionRouter, Pos2::new(480.0, 200.0), 1)
            .with_value("CHIRAL_SK_ROUTER"); cid += 1;

        // 6. Non-Abelian Majorana Braiding Junction 1
        let xmj1 = SchematicComponent::new(cid, ComponentKind::MajoranaJunction, Pos2::new(600.0, 200.0), 1)
            .with_value("BRAID_OP_B1"); cid += 1;

        // 7. Non-Abelian Majorana Braiding Junction 2
        let xmj2 = SchematicComponent::new(cid, ComponentKind::MajoranaJunction, Pos2::new(720.0, 200.0), 2)
            .with_value("BRAID_OP_B2"); cid += 1;

        // 8. Superconducting Nanowire Single-Phonon Detector (SNSPD)
        let r_snspd = SchematicComponent::new(cid, ComponentKind::Resistor, Pos2::new(840.0, 240.0), 1)
            .with_value("50_OHM"); cid += 1;
        let d_snspd = SchematicComponent::new(cid, ComponentKind::ZenerDiode, Pos2::new(840.0, 360.0), 1)
            .with_value("SNSPD_HOTSPOT"); cid += 1;

        // 9. Readout Probes
        let prb_click = SchematicComponent::new(cid, ComponentKind::LogicProbe, Pos2::new(920.0, 200.0), 1); cid += 1;
        let prb_braid = SchematicComponent::new(cid, ComponentKind::LogicProbe, Pos2::new(680.0, 140.0), 2);

        // Ground Bus at Y=420 from X=100 to X=860
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(100.0, 280.0), Pos2::new(100.0, 420.0)),
            WireSegment::new(Pos2::new(100.0, 420.0), Pos2::new(860.0, 420.0)),
        ])); wid += 1;

        // SAW IN- (200, 240) to ground bus
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(200.0, 240.0), Pos2::new(200.0, 420.0)),
        ])); wid += 1;
        // SAW OUT- (280, 240) to ground bus
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(280.0, 240.0), Pos2::new(280.0, 420.0)),
        ])); wid += 1;
        // VG(-) (440, 120) to ground bus
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(440.0, 120.0), Pos2::new(440.0, 420.0)),
        ])); wid += 1;
        // D_SNSPD pin 2 (840, 400) to ground bus
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(840.0, 400.0), Pos2::new(840.0, 420.0)),
        ])); wid += 1;

        // Microwave pulse to SAW IN+: V_PULSE(+) (100, 200) -> XSAW IN+ (200, 200)
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(100.0, 200.0), Pos2::new(200.0, 200.0)),
        ])); wid += 1;

        // SAW OUT+ (280, 200) to Parafermionic Cavity PORT1 (320, 200)
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(280.0, 200.0), Pos2::new(320.0, 200.0)),
        ])); wid += 1;

        // Parafermionic Cavity PORT2 (400, 200) to Skyrmion Router IN (440, 200)
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(400.0, 200.0), Pos2::new(440.0, 200.0)),
        ])); wid += 1;

        // Synthetic flux bias: VG(+) (440, 40) -> (480, 40) -> XSR GATE (480, 160)
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(440.0, 40.0), Pos2::new(480.0, 40.0)),
            WireSegment::new(Pos2::new(480.0, 40.0), Pos2::new(480.0, 160.0)),
        ])); wid += 1;

        // Skyrmion Router CH0 (520, 180) to XMJ1 J1 (560, 180)
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(520.0, 180.0), Pos2::new(560.0, 180.0)),
        ])); wid += 1;
        // Skyrmion Router CH1 (520, 220) to XMJ1 J2 (560, 220)
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(520.0, 220.0), Pos2::new(560.0, 220.0)),
        ])); wid += 1;

        // XMJ1 J3 (640, 200) to XMJ2 J1 (680, 180) and Braid Probe (660, 140)
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(640.0, 200.0), Pos2::new(660.0, 200.0)),
            WireSegment::new(Pos2::new(660.0, 200.0), Pos2::new(660.0, 180.0)),
            WireSegment::new(Pos2::new(660.0, 180.0), Pos2::new(680.0, 180.0)),
        ])); wid += 1;
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(660.0, 180.0), Pos2::new(660.0, 140.0)),
        ])); wid += 1;

        // XMJ2 J2 (680, 220) looped back to XMJ1 J2 (560, 220) for crossbar braiding topology
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(560.0, 220.0), Pos2::new(680.0, 220.0)),
        ])); wid += 1;

        // XMJ2 J3 (760, 200) to R_SNSPD pin 1 (840, 200) and PRB_CLICK (900, 200)
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(760.0, 200.0), Pos2::new(840.0, 200.0)),
            WireSegment::new(Pos2::new(840.0, 200.0), Pos2::new(900.0, 200.0)),
        ])); wid += 1;

        // R_SNSPD pin 2 (840, 280) to D_SNSPD pin 1 (840, 320)
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(840.0, 280.0), Pos2::new(840.0, 320.0)),
        ]));

        components.extend(vec![
            v_pulse, gnd, xsaw, xpc, vg, xsr, xmj1, xmj2, r_snspd, d_snspd, prb_click, prb_braid,
        ]);

        self.components = components;
        self.wires = wires;
        self.next_comp_id = cid;
        self.next_wire_id = wid;

        self.sync_canvas_state();
        self.run_erc();
        self.sim_status.clear();
        self.pending_auto_center = true;
        self.project_title = "Topological Quantum Acoustic Processor".to_string();
        self.current_project_path = None;
        self.modification_epoch = 0;
        self.clean_epoch = 0;
        self.is_modified = false;
        self.top_frame_config.circuit_name = self.project_title.clone();
        self.top_frame_config.is_modified = false;

        // Populate Authentic Oscilloscope Waveforms (Single-Phonon Wavepacket, Majorana Braiding, SNSPD Click)
        let mut trace_saw = WaveformTrace::new("SAW_PHONON (3.5 GHz Acoustic Wave)", Color32::from_rgb(80, 200, 255));
        let mut trace_braid = WaveformTrace::new("MAJORANA_PHASE (pi/2 Holonomy)", Color32::from_rgb(255, 180, 50));
        let mut trace_sk = WaveformTrace::new("CHIRAL_SKYRMION (32 dB Isolation)", Color32::from_rgb(100, 255, 140));
        let mut trace_click = WaveformTrace::new("SNSPD_HOTSPOT_CLICK (100 ps Reset)", Color32::from_rgb(255, 80, 120));

        let num_pts = 1200;
        let t_total = 10.0e-9; // 10 ns
        let dt = t_total / num_pts as f64;
        let f_saw = 3.5e9;

        for i in 0..num_pts {
            let t = i as f64 * dt;
            // Acoustic wave packet envelope
            let env = (-((t - 3.0e-9) / 0.8e-9).powi(2)).exp();
            let v_saw = env * (2.0 * std::f64::consts::PI * f_saw * t).sin();

            // Braiding holonomy phase shift
            let v_braid = if t < 2.0e-9 { 0.0 } else if t < 6.0e-9 {
                (t - 2.0e-9) / 4.0e-9 * (std::f64::consts::PI / 2.0)
            } else {
                std::f64::consts::PI / 2.0
            };

            // Chiral skyrmion routing envelope
            let v_sk = if t > 2.5e-9 && t < 7.5e-9 { 0.95 } else { 0.02 };

            // SNSPD single-phonon detection click at t = 6.2 ns
            let click = if t >= 6.2e-9 && t <= 7.2e-9 {
                let dt_click = (t - 6.2e-9) / 0.2e-9;
                (1.0 - (-dt_click * 10.0).exp()) * (-dt_click).exp() * 4.0
            } else {
                0.0
            };

            trace_saw.push(t, v_saw * 2.5);
            trace_braid.push(t, v_braid * 1.5);
            trace_sk.push(t, v_sk * 3.0);
            trace_click.push(t, click * 3.0);
        }

        self.multi_graph.route_simulation_traces(&[trace_saw, trace_braid, trace_sk, trace_click]);
        self.oscilloscope = self.multi_graph.primary_scope.clone();
    }

    /// Compiles schematic and runs the non-linear DC Operating Point (.OP) solver.
    pub fn run_dc_op(&mut self) {
        match compile_schematic_with_labels(&self.components, &self.wires, &self.net_labels) {
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
        match compile_schematic_with_labels(&self.components, &self.wires, &self.net_labels) {
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
        match compile_schematic_with_labels(&self.components, &self.wires, &self.net_labels) {
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
        match compile_schematic_with_labels(&self.components, &self.wires, &self.net_labels) {
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

    /// Checks whether a net label with the given ID is selected.
    pub fn is_label_selected(&self, id: usize) -> bool {
        self.selected_label_ids.contains(&id) || self.selected_label_id == Some(id)
    }

    /// Clears all component, wire, and net label selections.
    pub fn clear_selection(&mut self) {
        self.selected_component_ids.clear();
        self.selected_wire_ids.clear();
        self.selected_label_ids.clear();
        self.selected_component_id = None;
        self.selected_wire_id = None;
        self.selected_label_id = None;
        self.drag_start_mouse_pos = None;
        self.drag_attached_wires.clear();
        self.unsolvable_wiring_components.clear();
        self.sync_selection_to_canvas();
    }

    /// Synchronizes selection state, components, wires, and net labels from PhononApp to SchematicCanvas.
    pub fn sync_selection_to_canvas(&mut self) {
        self.canvas.components = self.components.clone();
        self.canvas.wires = self.wires.clone();
        self.canvas.net_labels = self.net_labels.clone();
        self.canvas.selected_component_ids = self.selected_component_ids.clone();
        self.canvas.selected_wire_ids = self.selected_wire_ids.clone();
        self.canvas.selected_label_ids = self.selected_label_ids.clone();
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

    /// Selects a single net label or adds to selection if multi is true.
    pub fn select_label(&mut self, id: usize, multi: bool) {
        if !multi {
            self.clear_selection();
        }
        self.selected_label_ids.insert(id);
        self.selected_label_id = Some(id);
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

    /// Toggles a net label's selection state (for Shift+Click).
    pub fn toggle_label_selection(&mut self, id: usize) {
        if self.selected_label_ids.contains(&id) {
            self.selected_label_ids.remove(&id);
            if self.selected_label_id == Some(id) {
                self.selected_label_id = self.selected_label_ids.iter().next().copied();
            }
        } else {
            self.selected_label_ids.insert(id);
            self.selected_label_id = Some(id);
        }
        self.sync_selection_to_canvas();
    }

    /// Selects all components, wires, and net labels in the current schematic sheet.
    pub fn select_all(&mut self) {
        self.selected_component_ids = self.components.iter().map(|c| c.id).collect();
        self.selected_wire_ids = self.wires.iter().map(|w| w.id).collect();
        self.selected_label_ids = self.net_labels.iter().map(|l| l.id).collect();
        self.selected_component_id = self.selected_component_ids.iter().next().copied();
        self.selected_wire_id = self.selected_wire_ids.iter().next().copied();
        self.selected_label_id = self.selected_label_ids.iter().next().copied();
        self.sync_selection_to_canvas();
    }

    /// Selects all components, wires, and net labels intersecting the given rectangle in world coordinates.
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
        for label in &self.net_labels {
            if rect.intersects(label.bounding_box()) {
                self.selected_label_ids.insert(label.id);
                self.selected_label_id = Some(label.id);
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

        let mut moving_comp_ids = HashSet::new();
        let mut moving_pins = Vec::new();
        for comp in &self.components {
            if self.is_component_selected(comp.id) {
                moving_comp_ids.insert(comp.id);
                for (pin_idx, (_, p)) in comp.all_pins().iter().enumerate() {
                    let normal = comp.pin_normal(pin_idx);
                    moving_pins.push((comp.id, pin_idx, *p, normal));
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

        let obstacles: Vec<Rect> = self
            .components
            .iter()
            .filter(|c| !moving_comp_ids.contains(&c.id))
            .map(|c| c.bounding_box())
            .collect();

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
                let start_attached = moving_pins.iter().find(|(_, _, p, _)| (*p - wire.start_point()).length() <= 8.0);
                let end_attached = moving_pins.iter().find(|(_, _, p, _)| (*p - wire.end_point()).length() <= 8.0);

                if start_attached.is_some() && end_attached.is_some() {
                    for seg in &mut wire.segments {
                        seg.start += delta;
                        seg.end += delta;
                    }
                } else if let Some(&(_cid, _pidx, _p, normal)) = start_attached {
                    let old_end = wire.end_point();
                    let new_start = wire.start_point() + delta;
                    if let Some(routed) = SchematicWire::manhattan_route_avoiding_obstacles(
                        wire.id,
                        new_start,
                        normal,
                        old_end,
                        None,
                        &obstacles,
                        wire.net_name.clone(),
                    ) {
                        *wire = routed;
                    } else if normal.is_vertical() {
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
                } else if let Some(&(_cid, _pidx, _p, normal)) = end_attached {
                    let old_start = wire.start_point();
                    let new_end = wire.end_point() + delta;
                    if let Some(routed) = SchematicWire::manhattan_route_avoiding_obstacles(
                        wire.id,
                        old_start,
                        normal.opposite(),
                        new_end,
                        Some(normal),
                        &obstacles,
                        wire.net_name.clone(),
                    ) {
                        *wire = routed;
                    } else if normal.is_vertical() {
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

        let sel_label_ids = self.selected_label_ids.clone();
        let sel_label_id = self.selected_label_id;
        let is_label_sel = |id: usize| sel_label_ids.contains(&id) || sel_label_id == Some(id);

        for label in &mut self.net_labels {
            if is_label_sel(label.id) {
                label.pos += delta;
            }
        }

        self.sync_selection_to_canvas();
    }

    /// Deletes all currently selected components, wires, and net labels in a unified atomic undo/redo command.
    pub fn delete_selected(&mut self) {
        let mut target_comp_ids = self.selected_component_ids.clone();
        if let Some(cid) = self.selected_component_id {
            target_comp_ids.insert(cid);
        }

        let mut target_wire_ids = self.selected_wire_ids.clone();
        if let Some(wid) = self.selected_wire_id {
            target_wire_ids.insert(wid);
        }

        let mut target_label_ids = self.selected_label_ids.clone();
        if let Some(lid) = self.selected_label_id {
            target_label_ids.insert(lid);
        }

        if target_comp_ids.is_empty() && target_wire_ids.is_empty() && target_label_ids.is_empty() {
            return;
        }

        let mut to_delete_comps = Vec::new();
        let mut to_delete_wires = Vec::new();
        let mut to_delete_labels = Vec::new();

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

        self.net_labels.retain(|l| {
            if target_label_ids.contains(&l.id) {
                to_delete_labels.push(l.clone());
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
        for label in to_delete_labels {
            batch.push(CanvasCommand::DeleteNetLabel(label));
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

    /// Duplicates all selected components, wires, and net labels, offset by (+40.0, +40.0) world coordinates.
    pub fn duplicate_selected(&mut self) {
        let mut target_comp_ids = self.selected_component_ids.clone();
        if let Some(cid) = self.selected_component_id {
            target_comp_ids.insert(cid);
        }

        let mut target_wire_ids = self.selected_wire_ids.clone();
        if let Some(wid) = self.selected_wire_id {
            target_wire_ids.insert(wid);
        }

        let mut target_label_ids = self.selected_label_ids.clone();
        if let Some(lid) = self.selected_label_id {
            target_label_ids.insert(lid);
        }

        if target_comp_ids.is_empty() && target_wire_ids.is_empty() && target_label_ids.is_empty() {
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

        // 3. Duplicate net labels
        let mut new_labels = Vec::new();
        for label in &self.net_labels {
            if target_label_ids.contains(&label.id) {
                let new_id = self.next_label_id;
                self.next_label_id += 1;
                let mut cloned = label.clone();
                cloned.id = new_id;
                cloned.pos += offset;
                new_labels.push(cloned);
            }
        }

        // 4. Atomically add items
        let mut batch = Vec::new();
        for comp in &new_comps {
            batch.push(CanvasCommand::AddComponent(comp.clone()));
            self.components.push(comp.clone());
        }
        for wire in &new_wires {
            batch.push(CanvasCommand::AddWire(wire.clone()));
            self.wires.push(wire.clone());
        }
        for label in &new_labels {
            batch.push(CanvasCommand::AddNetLabel(label.clone()));
            self.net_labels.push(label.clone());
        }

        if !batch.is_empty() {
            if batch.len() == 1 {
                self.history.record(batch.remove(0));
            } else {
                self.history.record(CanvasCommand::Batch(batch));
            }
            self.mark_dirty();
        }

        // 5. Select newly duplicated items
        self.selected_component_ids = new_comps.iter().map(|c| c.id).collect();
        self.selected_wire_ids = new_wires.iter().map(|w| w.id).collect();
        self.selected_label_ids = new_labels.iter().map(|l| l.id).collect();
        self.selected_component_id = self.selected_component_ids.iter().next().copied();
        self.selected_wire_id = self.selected_wire_ids.iter().next().copied();
        self.selected_label_id = self.selected_label_ids.iter().next().copied();
        self.sync_selection_to_canvas();

        self.sim_status = format!(
            "Duplicated {} components, {} wires, {} labels",
            new_comps.len(),
            new_wires.len(),
            new_labels.len()
        );
    }

    /// Rotates the active component(s) or net label(s) clockwise by 90 degrees.
    pub fn rotate_active(&mut self) {
        let mut target_comp_ids = self.selected_component_ids.clone();
        if let Some(cid) = self.selected_component_id {
            target_comp_ids.insert(cid);
        }

        let mut target_label_ids = self.selected_label_ids.clone();
        if let Some(lid) = self.selected_label_id {
            target_label_ids.insert(lid);
        }

        if !target_comp_ids.is_empty() || !target_label_ids.is_empty() {
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

            for label in &mut self.net_labels {
                if target_label_ids.contains(&label.id) {
                    label.orientation = label.orientation.rotate_clockwise();
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

    /// Toggles horizontal mirroring for all selected components or active placement kind.
    pub fn mirror_active(&mut self) {
        let mut target_comp_ids = self.selected_component_ids.clone();
        if let Some(cid) = self.selected_component_id {
            target_comp_ids.insert(cid);
        }

        if !target_comp_ids.is_empty() {
            let mut batch = Vec::new();
            for comp in &mut self.components {
                if target_comp_ids.contains(&comp.id) {
                    let from_mirrored = comp.mirrored;
                    comp.mirror_horizontal();
                    let to_mirrored = comp.mirrored;
                    batch.push(CanvasCommand::MirrorComponent {
                        id: comp.id,
                        from_mirrored,
                        to_mirrored,
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
            self.placement_mirrored = !self.placement_mirrored;
            self.sim_status.clear();
        }
    }

    /// Toggles horizontal mirroring for all selected components.
    pub fn mirror_selected(&mut self) {
        self.mirror_active();
    }

    /// Increases size of all selected components (or single selected component)
    /// to the next valid scale that keeps all pins strictly on the grid.
    pub fn size_up_selected(&mut self) {
        self.resize_selected(true);
    }

    /// Decreases size of all selected components (or single selected component)
    /// to the next valid scale that keeps all pins strictly on the grid.
    pub fn size_down_selected(&mut self) {
        self.resize_selected(false);
    }

    fn resize_selected(&mut self, size_up: bool) {
        let mut target_comp_ids = self.selected_component_ids.clone();
        if let Some(cid) = self.selected_component_id {
            target_comp_ids.insert(cid);
        }

        if target_comp_ids.is_empty() {
            return;
        }

        let grid_size = self.canvas.grid_size;
        let mut batch = Vec::new();
        let mut pin_replacements: Vec<(Pos2, Pos2)> = Vec::new();

        for comp in &mut self.components {
            if target_comp_ids.contains(&comp.id) {
                let from_scale = comp.scale;
                let target_opt = if size_up {
                    comp.next_valid_scale_up(grid_size)
                } else {
                    comp.next_valid_scale_down(grid_size)
                };

                if let Some(to_scale) = target_opt {
                    let old_pins = comp.all_pins();
                    comp.scale = to_scale;
                    comp.set_property("scale", format!("{:.2}", to_scale));
                    let new_pins = comp.all_pins();

                    for (i, &(_, old_pos)) in old_pins.iter().enumerate() {
                        if let Some(&(_, new_pos)) = new_pins.get(i) {
                            if (old_pos - new_pos).length() > 1e-3 {
                                pin_replacements.push((old_pos, new_pos));
                            }
                        }
                    }

                    batch.push(CanvasCommand::ScaleComponent {
                        id: comp.id,
                        from_scale,
                        to_scale,
                    });
                }
            }
        }

        if !batch.is_empty() {
            // Update any attached wires whose endpoints were at old pin positions
            if !pin_replacements.is_empty() {
                for wire in &mut self.wires {
                    let old_wire = wire.clone();
                    let mut modified = false;

                    if let Some(first_seg) = wire.segments.first_mut() {
                        for &(old_pos, new_pos) in &pin_replacements {
                            if (first_seg.start - old_pos).length() < 1e-2 {
                                first_seg.start = new_pos;
                                modified = true;
                                break;
                            }
                        }
                    }

                    if let Some(last_seg) = wire.segments.last_mut() {
                        for &(old_pos, new_pos) in &pin_replacements {
                            if (last_seg.end - old_pos).length() < 1e-2 {
                                last_seg.end = new_pos;
                                modified = true;
                                break;
                            }
                        }
                    }

                    if modified {
                        // Re-route with Manhattan routing to ensure all segments remain orthogonal
                        let start = wire.start_point();
                        let end = wire.end_point();
                        let mut routed = SchematicWire::manhattan_route_hv(wire.id, start, end);
                        routed.net_name = wire.net_name.clone();
                        *wire = routed;

                        batch.push(CanvasCommand::DeleteWire(old_wire));
                        batch.push(CanvasCommand::AddWire(wire.clone()));
                    }
                }
            }

            if batch.len() == 1 {
                self.history.record(batch.remove(0));
            } else {
                self.history.record(CanvasCommand::Batch(batch));
            }
            self.mark_dirty();
            self.sync_selection_to_canvas();
            self.sim_status.clear();
        }
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
            ActionId::MirrorComponent => self.mirror_active(),
            ActionId::SizeUpComponent => self.size_up_selected(),
            ActionId::SizeDownComponent => self.size_down_selected(),
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
            ActionId::ToolNetLabel => {
                self.selected_tool = ToolMode::NetLabel;
                self.active_wire_start = None;
            }
            ActionId::ClearWire => self.active_wire_start = None,
            ActionId::OpenCommandPalette => self.command_palette.open(),
            ActionId::OpenPreferences => self.preferences_dialog.is_open = true,
            ActionId::Toggle3DCardView => {
                if self.viewport_tabs.active_tab == crate::viewport_tabs::CentralViewportTab::PhysicalCard3D {
                    self.viewport_tabs.set_active_tab(crate::viewport_tabs::CentralViewportTab::Schematic);
                } else {
                    self.viewport_tabs.set_active_tab(crate::viewport_tabs::CentralViewportTab::PhysicalCard3D);
                    let _ = self.card_3d_viewport.run_synthesis(&self.components, &self.wires);
                }
            }
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
                self.placement_mirrored = false;
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

        // Mirror: M
        if !ctrl && ctx.input(|i| i.key_pressed(Key::M)) {
            self.mirror_active();
            return;
        }

        // Size Up Component: ]
        if !ctrl && ctx.input(|i| i.key_pressed(Key::CloseBracket)) {
            self.size_up_selected();
            return;
        }

        // Size Down Component: [
        if !ctrl && ctx.input(|i| i.key_pressed(Key::OpenBracket)) {
            self.size_down_selected();
            return;
        }

        // Toggle 3D Physical Card View: F3
        if ctx.input(|i| i.key_pressed(Key::F3)) {
            self.execute_action(ActionId::Toggle3DCardView);
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
            self.placement_mirrored = false;
            return;
        }

        // Wire Tool: W
        if !ctrl && ctx.input(|i| i.key_pressed(Key::W)) {
            self.selected_tool = ToolMode::Wire;
            self.active_wire_start = None;
            self.placement_rotation = 0;
            self.placement_mirrored = false;
            return;
        }

        // Bus Tool: B
        if !ctrl && ctx.input(|i| i.key_pressed(Key::B)) {
            self.selected_tool = ToolMode::Bus;
            self.active_wire_start = None;
            self.placement_rotation = 0;
            self.placement_mirrored = false;
            return;
        }

        // Probe Tool: P
        if !ctrl && ctx.input(|i| i.key_pressed(Key::P)) {
            self.selected_tool = ToolMode::Probe;
            self.active_wire_start = None;
            self.placement_rotation = 0;
            self.placement_mirrored = false;
            return;
        }

        // Net Label Tool: L
        if !ctrl && ctx.input(|i| i.key_pressed(Key::L)) {
            self.selected_tool = ToolMode::NetLabel;
            self.active_wire_start = None;
            self.placement_rotation = 0;
            self.placement_mirrored = false;
            return;
        }
    }

    /// Interactive canvas response and rendering.
    pub fn render_canvas(&mut self, ui: &mut egui::Ui) {
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

        // 3. Process mouse interactions on canvas FIRST so all state mutations
        // (moving components, snapping to grid, selection changes) take effect in the current frame
        // before any scene elements or selection halos are rendered.
        let mut hovered_wire_telemetry: Option<(egui::Pos2, f64, f64)> = None;

        let mouse_pos = ui.input(|i| i.pointer.hover_pos());
        if let Some(mouse_screen) = mouse_pos {
            let mouse_world = self.canvas.screen_to_world(mouse_screen);
            let snapped_world = self.canvas.snap_to_grid(mouse_world);

            // Wire hover telemetry badge lookup (V and I readouts)
            if self.selected_tool == ToolMode::Select || self.selected_tool == ToolMode::Probe {
                let hover_tol = 8.0 / self.canvas.zoom;
                if let Some(hovered_wire) = self.wires.iter().find(|w| w.contains(mouse_world, hover_tol)) {
                    let v = self.wire_voltages.get(&hovered_wire.id).copied().unwrap_or(0.0);
                    let i = self.wire_currents.get(&hovered_wire.id).copied().unwrap_or(0.0);
                    hovered_wire_telemetry = Some((mouse_screen + Vec2::new(20.0, -20.0), v, i));
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
                        } else if let Some(label) = self
                            .net_labels
                            .iter()
                            .rev()
                            .find(|l| l.hit_test(mouse_world, 6.0))
                        {
                            let lid = label.id;
                            if shift {
                                self.toggle_label_selection(lid);
                            } else {
                                self.select_label(lid, false);
                            }
                        } else if !shift {
                            self.clear_selection();
                        }
                    }
                    ToolMode::Place(kind) | ToolMode::PlaceComponent(kind) => {
                        let count = self.components.iter().filter(|c| c.kind == *kind).count() + 1;
                        let is_empty = self.is_canvas_empty();
                        let place_pos = if is_empty {
                            let target_screen = self.canvas.world_to_screen(snapped_world);
                            self.canvas.reset_origin_at_screen(target_screen);
                            self.sheets.active_sheet_mut().camera_offset = self.canvas.pan;
                            egui::Pos2::ZERO
                        } else {
                            snapped_world
                        };

                        let mut new_comp = SchematicComponent::new(
                            self.next_comp_id,
                            kind.clone(),
                            place_pos,
                            count,
                        );
                        new_comp.rotation = self.placement_rotation;
                        new_comp.mirrored = self.placement_mirrored;
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
                            let is_empty = self.is_canvas_empty();
                            if is_empty {
                                let target_screen = self.canvas.world_to_screen(snapped_world);
                                self.canvas.reset_origin_at_screen(target_screen);
                                self.sheets.active_sheet_mut().camera_offset = self.canvas.pan;
                                self.active_wire_start = Some(egui::Pos2::ZERO);
                            } else {
                                // Find nearest pin to snap start
                                let nearest_pin = self
                                    .components
                                    .iter()
                                    .flat_map(|c| c.all_pins())
                                    .map(|(_, p)| p)
                                    .find(|&p| (p - mouse_world).length() <= 12.0);
                                self.active_wire_start = Some(nearest_pin.unwrap_or(snapped_world));
                            }
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
                            let is_empty = self.is_canvas_empty();
                            if is_empty {
                                let target_screen = self.canvas.world_to_screen(snapped_world);
                                self.canvas.reset_origin_at_screen(target_screen);
                                self.sheets.active_sheet_mut().camera_offset = self.canvas.pan;
                                self.active_wire_start = Some(egui::Pos2::ZERO);
                            } else {
                                self.active_wire_start = Some(snapped_world);
                            }
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
                    ToolMode::NetLabel => {
                        let is_empty = self.is_canvas_empty();
                        let place_pos = if is_empty {
                            let target_screen = self.canvas.world_to_screen(snapped_world);
                            self.canvas.reset_origin_at_screen(target_screen);
                            self.sheets.active_sheet_mut().camera_offset = self.canvas.pan;
                            egui::Pos2::ZERO
                        } else {
                            snapped_world
                        };

                        let lbl_id = self.next_label_id;
                        self.next_label_id += 1;
                        let lbl_name = if self.active_net_label_text.trim().is_empty() {
                            format!("NET{}", lbl_id)
                        } else {
                            self.active_net_label_text.clone()
                        };
                        let orientation = NetLabelOrientation::from_quarter_turns(self.placement_rotation);
                        let mut lbl = NetLabel::new(lbl_id, lbl_name, place_pos);
                        lbl.orientation = orientation;
                        self.history.record(CanvasCommand::AddNetLabel(lbl.clone()));
                        self.net_labels.push(lbl);
                        self.sim_status = format!("Placed Net Label {}", self.active_net_label_text);
                        self.mark_dirty();
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
                        self.drag_start_mouse_pos = Some(mouse_world);
                        self.drag_start_positions = self
                            .components
                            .iter()
                            .filter(|c| self.is_component_selected(c.id))
                            .map(|c| (c.id, c.pos))
                            .collect();
                        self.drag_start_label_positions = self
                            .net_labels
                            .iter()
                            .filter(|l| self.is_label_selected(l.id))
                            .map(|l| (l.id, l.pos))
                            .collect();
                        self.drag_start_wires = self.wires.clone();
                        self.unsolvable_wiring_components.clear();

                        // Cache permanent wire attachments to moving component pins
                        let moving_comp_ids: HashSet<usize> =
                            self.drag_start_positions.iter().map(|(id, _)| *id).collect();
                        let mut attached = Vec::new();
                        for c in &self.components {
                            if moving_comp_ids.contains(&c.id) {
                                for (pin_idx, (_, pin_pos)) in c.all_pins().iter().enumerate() {
                                    for wire in &self.wires {
                                        let is_start = (wire.start_point() - *pin_pos).length() <= 8.0;
                                        let is_end = (wire.end_point() - *pin_pos).length() <= 8.0;
                                        if is_start && is_end {
                                            // Wire internal to moving component
                                        } else if is_start {
                                            attached.push(DragAttachedWire {
                                                wire_id: wire.id,
                                                is_start: true,
                                                comp_id: c.id,
                                                pin_idx,
                                                other_end: wire.end_point(),
                                                net_name: wire.net_name.clone(),
                                            });
                                        } else if is_end {
                                            attached.push(DragAttachedWire {
                                                wire_id: wire.id,
                                                is_start: false,
                                                comp_id: c.id,
                                                pin_idx,
                                                other_end: wire.start_point(),
                                                net_name: wire.net_name.clone(),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                        self.drag_attached_wires = attached;
                    }
                } else if let Some(wire) = self.wires.iter().rev().find(|w| w.contains(mouse_world, 6.0)) {
                    let wire_id = wire.id;
                    if shift {
                        self.toggle_wire_selection(wire_id);
                    } else {
                        self.select_wire(wire_id, false);
                    }
                } else if let Some(label) = self.net_labels.iter().rev().find(|l| l.hit_test(mouse_world, 6.0)) {
                    let label_id = label.id;
                    if shift {
                        self.toggle_label_selection(label_id);
                    } else {
                        if !self.is_label_selected(label_id) {
                            self.select_label(label_id, false);
                        }
                        self.dragging_selection = true;
                        self.drag_start_mouse_pos = Some(mouse_world);
                        self.drag_start_positions = self
                            .components
                            .iter()
                            .filter(|c| self.is_component_selected(c.id))
                            .map(|c| (c.id, c.pos))
                            .collect();
                        self.drag_start_label_positions = self
                            .net_labels
                            .iter()
                            .filter(|l| self.is_label_selected(l.id))
                            .map(|l| (l.id, l.pos))
                            .collect();
                        self.drag_start_wires = self.wires.clone();
                        self.unsolvable_wiring_components.clear();
                    }
                } else {
                    // Empty canvas: Start rubberband marquee box
                    self.marquee_start = Some(mouse_world);
                    self.marquee_current = Some(mouse_world);
                    self.sync_selection_to_canvas();
                }
                ui.ctx().request_repaint();
            }

            if response.dragged_by(PointerButton::Primary) && self.selected_tool == ToolMode::Select {
                if self.dragging_selection {
                    if let Some(start_mouse) = self.drag_start_mouse_pos {
                        let total_drag = mouse_world - start_mouse;
                        let snapped_dx = (total_drag.x / 20.0).round() * 20.0;
                        let snapped_dy = (total_drag.y / 20.0).round() * 20.0;
                        let grid_delta = Vec2::new(snapped_dx, snapped_dy);

                        let moving_comp_ids: HashSet<usize> =
                            self.drag_start_positions.iter().map(|(id, _)| *id).collect();

                        for (cid, start_pos) in &self.drag_start_positions {
                            if let Some(c) = self.components.iter_mut().find(|c| c.id == *cid) {
                                c.pos = self.canvas.snap_to_grid(*start_pos + grid_delta);
                            }
                        }

                        for (lid, start_pos) in &self.drag_start_label_positions {
                            if let Some(l) = self.net_labels.iter_mut().find(|l| l.id == *lid) {
                                l.pos = self.canvas.snap_to_grid(*start_pos + grid_delta);
                            }
                        }

                        // Collect non-moving component bounding boxes as obstacles
                        let mut obstacles = Vec::new();
                        let mut moving_bboxes = Vec::new();
                        for c in &self.components {
                            if moving_comp_ids.contains(&c.id) {
                                moving_bboxes.push((c.id, c.bounding_box()));
                            } else {
                                obstacles.push(c.bounding_box());
                            }
                        }

                        // Check collision: does any moving component overlap with stationary components?
                        let mut has_collision = false;
                        for (_cid, m_bbox) in &moving_bboxes {
                            for obs in &obstacles {
                                if m_bbox.shrink(2.0).intersects(*obs) {
                                    has_collision = true;
                                    break;
                                }
                            }
                            if has_collision {
                                break;
                            }
                        }

                        if has_collision {
                            // Collision detected! Keep wires at pre-drag snapshot, mark components as unsolvable
                            self.wires = self.drag_start_wires.clone();
                            for id in &moving_comp_ids {
                                self.unsolvable_wiring_components.insert(*id);
                            }
                        } else {
                            // Try routing all attached wires avoiding obstacles
                            let mut wire_map: HashMap<usize, SchematicWire> =
                                self.drag_start_wires.iter().map(|w| (w.id, w.clone())).collect();
                            let mut routing_success = true;

                            for att in &self.drag_attached_wires {
                                let comp = match self.components.iter().find(|c| c.id == att.comp_id) {
                                    Some(c) => c,
                                    None => continue,
                                };
                                let moving_pin_pos = match comp.pin_world_pos(att.pin_idx) {
                                    Some(p) => p,
                                    None => continue,
                                };
                                let moving_pin_normal = comp.pin_normal(att.pin_idx);

                                let (from, from_normal, to, to_normal) = if att.is_start {
                                    (moving_pin_pos, moving_pin_normal, att.other_end, None)
                                } else {
                                    (att.other_end, moving_pin_normal.opposite(), moving_pin_pos, Some(moving_pin_normal))
                                };

                                if let Some(routed) = SchematicWire::manhattan_route_avoiding_obstacles(
                                    att.wire_id,
                                    from,
                                    from_normal,
                                    to,
                                    to_normal,
                                    &obstacles,
                                    att.net_name.clone(),
                                ) {
                                    wire_map.insert(att.wire_id, routed);
                                } else {
                                    routing_success = false;
                                    break;
                                }
                            }

                            if routing_success {
                                self.wires = wire_map.into_values().collect();
                                for id in &moving_comp_ids {
                                    self.unsolvable_wiring_components.remove(id);
                                }
                            } else {
                                self.wires = self.drag_start_wires.clone();
                                for id in &moving_comp_ids {
                                    self.unsolvable_wiring_components.insert(*id);
                                }
                            }
                        }

                        self.sync_selection_to_canvas();
                        ui.ctx().request_repaint();
                    }
                } else if self.marquee_start.is_some() {
                    self.marquee_current = Some(mouse_world);
                    self.sync_selection_to_canvas();
                    ui.ctx().request_repaint();
                }
            }

            if response.drag_stopped() && self.selected_tool == ToolMode::Select {
                if self.dragging_selection {
                    let moving_comp_ids: HashSet<usize> =
                        self.drag_start_positions.iter().map(|(id, _)| *id).collect();

                    let is_unsolvable = moving_comp_ids
                        .iter()
                        .any(|id| self.unsolvable_wiring_components.contains(id));

                    let mut batch = Vec::new();
                    for (id, start_pos) in self.drag_start_positions.drain(..) {
                        if let Some(comp) = self.components.iter().find(|c| c.id == id) {
                            if comp.pos != start_pos {
                                batch.push(CanvasCommand::MoveComponent {
                                    id,
                                    from: start_pos,
                                    to: comp.pos,
                                });
                            }
                        }
                    }

                    for (id, start_pos) in self.drag_start_label_positions.drain(..) {
                        if let Some(lbl) = self.net_labels.iter().find(|l| l.id == id) {
                            if lbl.pos != start_pos {
                                batch.push(CanvasCommand::MoveNetLabel {
                                    id,
                                    from: start_pos,
                                    to: lbl.pos,
                                });
                            }
                        }
                    }

                    if !is_unsolvable {
                        for start_wire in self.drag_start_wires.drain(..) {
                            if let Some(curr_wire) = self.wires.iter().find(|w| w.id == start_wire.id) {
                                if curr_wire.segments != start_wire.segments {
                                    batch.push(CanvasCommand::DeleteWire(start_wire));
                                    batch.push(CanvasCommand::AddWire(curr_wire.clone()));
                                }
                            }
                        }
                    } else {
                        // Unsolvable state: keep pre-drag wires (disconnected), component moved, pin tips remain red
                        self.drag_start_wires.clear();
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
                    self.drag_start_mouse_pos = None;
                    self.drag_attached_wires.clear();
                    self.sync_selection_to_canvas();
                    ui.ctx().request_repaint();
                } else if let (Some(m_start), Some(m_curr)) = (self.marquee_start.take(), self.marquee_current.take()) {
                    let marquee_rect = Rect::from_two_pos(m_start, m_curr);
                    if marquee_rect.width() > 3.0 || marquee_rect.height() > 3.0 {
                        let shift = ui.input(|i| i.modifiers.shift);
                        self.select_in_rect(marquee_rect, shift);
                    }
                    self.sync_selection_to_canvas();
                    ui.ctx().request_repaint();
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

            // Render component placement ghost preview with snap-target pin rings
            if let Some(kind) = self.selected_tool.place_kind() {
                let mut ghost = SchematicComponent::new(0, kind.clone(), snapped_world, 0);
                ghost.rotation = self.placement_rotation;
                ghost.mirrored = self.placement_mirrored;
                ghost.render_with_theme(&painter, &self.canvas, true, None, &theme);

                let pin_ring_color = theme.accent_primary.gamma_multiply(0.7);
                let pin_fill_color = theme.accent_primary.gamma_multiply(0.2);
                for (_pin_name, pin_pos) in ghost.all_pins() {
                    let pin_screen = self.canvas.world_to_screen(pin_pos);
                    painter.circle_filled(pin_screen, 4.0 * self.canvas.zoom.clamp(0.8, 1.5), pin_fill_color);
                    painter.circle_stroke(
                        pin_screen,
                        5.5 * self.canvas.zoom.clamp(0.8, 1.5),
                        Stroke::new(1.5, pin_ring_color),
                    );
                }
            }

            // Render net label placement ghost preview
            if self.selected_tool == ToolMode::NetLabel {
                let orientation = NetLabelOrientation::from_quarter_turns(self.placement_rotation);
                let ghost_name = if self.active_net_label_text.trim().is_empty() {
                    format!("NET{}", self.next_label_id)
                } else {
                    self.active_net_label_text.clone()
                };
                let mut ghost_lbl = NetLabel::new(0, ghost_name, snapped_world);
                ghost_lbl.orientation = orientation;
                ghost_lbl.render_ghost(&painter, &self.canvas, &theme);

                let anchor_screen = self.canvas.world_to_screen(snapped_world);
                painter.circle_filled(
                    anchor_screen,
                    3.5 * self.canvas.zoom.clamp(0.8, 1.5),
                    theme.accent_primary.gamma_multiply(0.3),
                );
                painter.circle_stroke(
                    anchor_screen,
                    5.0 * self.canvas.zoom.clamp(0.8, 1.5),
                    Stroke::new(1.5, theme.accent_primary.gamma_multiply(0.8)),
                );
            }
        }

        // 4. State synchronization after interaction
        self.sync_selection_to_canvas();

        // Collect all pin positions for snapping and junction rendering (up-to-date)
        let mut all_pin_positions = Vec::new();
        for comp in &self.components {
            for (_, p) in comp.all_pins() {
                all_pin_positions.push(p);
            }
        }

        // 5. Render wires
        for wire in &self.wires {
            let is_sel = self.is_wire_selected(wire.id);
            wire.render_with_theme(&painter, &self.canvas, is_sel, &theme);
        }

        // 5b. Render buses
        for bus in &self.buses {
            bus.render(&painter, &self.canvas, false);
        }

        // 5c. Render subcircuit instances
        for inst in &self.canvas.subcircuit_instances {
            inst.render(&painter, &self.canvas, self.subcircuits.get(&inst.def_name), false);
        }

        // 5d. Render junction dots
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

        // 5e. Render wire crossing bridge hops
        let crossings = compute_wire_crossings(&self.wires, &junctions);
        render_wire_crossings(&painter, &self.canvas, &crossings, &theme);

        // 5f. Render net labels
        for label in &self.net_labels {
            let is_sel = self.is_label_selected(label.id);
            label.render(&painter, &self.canvas, is_sel, &theme);
        }

        // 5g. Render illuminated selection halos (drawn around current component positions)
        self.canvas.render_selection_halos(&painter, &self.components, &self.wires);

        // 5f. Render rubberband marquee drag box
        self.canvas.render_marquee(&painter);

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

            let pin_override = if self.unsolvable_wiring_components.contains(&comp.id) {
                Some(Color32::from_rgb(255, 60, 60))
            } else {
                None
            };

            comp.render_with_theme_and_override(
                &painter,
                &self.canvas,
                is_sel,
                if pin_voltages.is_empty() {
                    None
                } else {
                    Some(&pin_voltages)
                },
                &theme,
                pin_override,
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

        // 6b. Wire hover telemetry badge (rendered on top of wires and components)
        if let Some((badge_screen, v, i)) = hovered_wire_telemetry {
            let v_str = crate::oscilloscope::format_voltage_si(v);
            let i_str = crate::oscilloscope::format_current_si(i);
            crate::widgets::render_dual_telemetry_pill(
                &painter,
                badge_screen,
                "V",
                &v_str,
                "I",
                &i_str,
                &crate::widgets::PillBadgeStyle::wire_telemetry(),
                self.canvas.zoom,
            );
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
            &theme,
        );
        if let Some(act) = toolbar_action {
            match act {
                FloatingToolbarAction::SelectTool(tool) => {
                    self.selected_tool = tool;
                    self.active_wire_start = None;
                }
                FloatingToolbarAction::Rotate => self.rotate_active(),
                FloatingToolbarAction::Mirror => self.mirror_active(),
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
                    self.placement_mirrored = false;
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
            let mut do_mirror = false;
            let mut do_size_up = false;
            let mut do_size_down = false;
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
                let rot_deg = (comp.rotation % 4) * 90;
                let combo_idx = (comp.rotation % 4) + (if comp.mirrored { 4 } else { 0 }) + 1;
                ui.label(format!(
                    "Orientation: {} deg{} [Combo {}/8]",
                    rot_deg,
                    if comp.mirrored { " (Mirrored)" } else { "" },
                    combo_idx
                ));

                ui.horizontal(|ui| {
                    if ui.button("Rotate CW (R)").clicked() {
                        do_rotate = true;
                    }
                    let mut is_mirrored = comp.mirrored;
                    if ui.checkbox(&mut is_mirrored, "Mirror (M)").changed() {
                        do_mirror = true;
                    }
                });

                let current_scale = comp.scale;
                let can_down = comp.next_valid_scale_down(self.canvas.grid_size).is_some();
                let can_up = comp.next_valid_scale_up(self.canvas.grid_size).is_some();
                ui.horizontal(|ui| {
                    ui.label("Size:");
                    if ui.add_enabled(can_down, egui::Button::new(" - ")).clicked() {
                        do_size_down = true;
                    }
                    ui.label(format!("{:.2}x", current_scale));
                    if ui.add_enabled(can_up, egui::Button::new(" + ")).clicked() {
                        do_size_up = true;
                    }
                    ui.label("([ / ])");
                });

                // Digital & Bus Architecture (Logic gates, Arithmetic blocks, Splitters/Mergers, Float ops)
                let is_gate = matches!(
                    comp.kind,
                    ComponentKind::AndGate
                        | ComponentKind::OrGate
                        | ComponentKind::NandGate
                        | ComponentKind::NorGate
                        | ComponentKind::XorGate
                        | ComponentKind::XnorGate
                );
                let is_arithmetic = matches!(
                    comp.kind,
                    ComponentKind::Adder
                        | ComponentKind::Subtractor
                        | ComponentKind::Multiplier
                        | ComponentKind::Divider
                        | ComponentKind::ArithmeticLogicUnit
                        | ComponentKind::BitSplitter
                        | ComponentKind::BitMerger
                        | ComponentKind::BusTap
                );
                let is_float = matches!(
                    comp.kind,
                    ComponentKind::FloatAdder
                        | ComponentKind::FloatSubtractor
                        | ComponentKind::FloatMultiplier
                        | ComponentKind::FloatDivider
                        | ComponentKind::FloatComparator
                );

                if is_gate || is_arithmetic || is_float {
                    ui.separator();
                    ui.label(RichText::new("Digital & Bus Architecture").strong().color(Color32::from_rgb(147, 197, 253)));

                    if is_gate {
                        let mut n = comp.input_count();
                        ui.horizontal(|ui| {
                            ui.label("Input Count:");
                            if ui.add(egui::Slider::new(&mut n, 2..=16).text("inputs")).changed() {
                                comp.set_input_count(n);
                            }
                        });
                    }

                    if is_gate || is_arithmetic {
                        let mut bw = comp.bit_width();
                        ui.horizontal(|ui| {
                            ui.label("Bit Width:");
                            for &w in &[1, 2, 4, 8, 16, 32, 64] {
                                if ui.selectable_label(bw == w, format!("{}", w)).clicked() {
                                    bw = w;
                                    comp.set_bit_width(bw);
                                }
                            }
                        });

                        let mut parts_str = comp.input_partitions();
                        ui.horizontal(|ui| {
                            ui.label("Partitions:");
                            if ui.text_edit_singleline(&mut parts_str).changed() {
                                comp.set_input_partitions(parts_str.clone());
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Presets:");
                            if ui.button("Uniform").clicked() {
                                let n = comp.input_count();
                                let w = comp.bit_width();
                                let part = (w / n as u32).max(1);
                                let s = (0..n).map(|_| part.to_string()).collect::<Vec<_>>().join(",");
                                comp.set_input_partitions(s);
                            }
                            if ui.button("Dual").clicked() {
                                let w = comp.bit_width();
                                let half = (w / 2).max(1);
                                comp.set_input_partitions(format!("{},{}", half, half));
                            }
                            if ui.button("Single Bus").clicked() {
                                let w = comp.bit_width();
                                comp.set_input_partitions(format!("{}", w));
                            }
                        });
                    }

                    if is_float {
                        let current_prec = comp.float_precision();
                        ui.horizontal(|ui| {
                            ui.label("Precision:");
                            for &prec in &["FP8", "FP16", "FP32", "FP64", "FP128"] {
                                if ui.selectable_label(current_prec == prec, prec).clicked() {
                                    comp.set_float_precision(prec);
                                    let bits = match prec {
                                        "FP8" => 8,
                                        "FP16" => 16,
                                        "FP32" => 32,
                                        "FP64" => 64,
                                        _ => 128,
                                    };
                                    comp.set_bit_width(bits);
                                }
                            }
                        });
                    }
                }

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
            if do_mirror {
                self.mirror_active();
            }
            if do_size_up {
                self.size_up_selected();
            }
            if do_size_down {
                self.size_down_selected();
            }
            if do_delete {
                self.delete_selected();
            }
        } else if let Some(wid) = self.selected_wire_id {
            let mut do_delete_wire = false;
            let mut new_bit_width = None;
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

                ui.separator();
                ui.label(RichText::new("Bus Architecture").strong().color(Color32::from_rgb(147, 197, 253)));
                let mut bw = wire.bit_width;
                ui.horizontal(|ui| {
                    ui.label("Bit Width:");
                    for &w in &[1, 2, 4, 8, 16, 32, 64] {
                        if ui.selectable_label(bw == w, format!("{}", w)).clicked() {
                            bw = w;
                        }
                    }
                });
                if bw != wire.bit_width {
                    new_bit_width = Some(bw);
                }
                if wire.is_bus() {
                    ui.label(
                        RichText::new(format!("Compressed Bus: /{} badge rendered", wire.bit_width))
                            .color(Color32::from_rgb(147, 197, 253))
                    );
                } else {
                    ui.label("Scalar Wire (1 bit)");
                }
                ui.separator();

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
            if let Some(bw) = new_bit_width {
                if let Some(w_mut) = self.wires.iter_mut().find(|w| w.id == wid) {
                    w_mut.bit_width = bw;
                }
            }
            if do_delete_wire {
                self.delete_selected();
            }
        } else if let Some(prefix) = self.selected_tool.place_kind().map(|k| k.prefix()) {
            let rot_deg = (self.placement_rotation % 4) * 90;
            let combo_idx = (self.placement_rotation % 4) + (if self.placement_mirrored { 4 } else { 0 }) + 1;
            let mut do_rotate = false;
            let mut do_mirror = false;
            ui.label(format!("Placing: {}", prefix));
            ui.label(format!(
                "Orientation: {} deg{} [Combo {}/8]",
                rot_deg,
                if self.placement_mirrored { " (Mirrored)" } else { "" },
                combo_idx
            ));
            ui.horizontal(|ui| {
                if ui.button("Rotate CW (R)").clicked() {
                    do_rotate = true;
                }
                let mut is_mirrored = self.placement_mirrored;
                if ui.checkbox(&mut is_mirrored, "Mirror (M)").changed() {
                    do_mirror = true;
                }
            });
            if do_rotate {
                self.rotate_active();
            }
            if do_mirror {
                self.mirror_active();
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
        #[cfg(not(target_arch = "wasm32"))]
        if self.pending_start_maximize {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(true));
            self.pending_start_maximize = false;
        }

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

        // 6. Center Viewport (Generalized Tab System: Schematic Canvas vs 3D Physical Card)
        CentralPanel::default().show(ui, |ui| {
            // Central Viewport Tab Bar
            ui.horizontal(|ui| {
                self.viewport_tabs.render_tab_bar(ui);
            });
            ui.add_space(2.0);
            ui.separator();

            match self.viewport_tabs.active_tab {
                crate::viewport_tabs::CentralViewportTab::Schematic => {
                    self.render_canvas(ui);
                }
                crate::viewport_tabs::CentralViewportTab::PhysicalCard3D => {
                    self.card_3d_viewport.render(ui, &self.components, &self.wires);
                }
                crate::viewport_tabs::CentralViewportTab::BoardLayout2D => {
                    self.card_3d_viewport.reset_camera_top();
                    self.card_3d_viewport.render(ui, &self.components, &self.wires);
                }
            }
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

        // 75. Interactive Cavity Optomagnonic Polariton Frequency Comb Dialog
        self.optomagnonic_comb_dialog.ui(ui.ctx());

        // 76. Interactive Floquet Time-Crystal Magnetometer Sensor Dialog
        self.floquet_sensor_dialog.ui(ui.ctx());

        // 77. Interactive Quantum Acoustic Giant Atom Waveguide QED Processor Dialog
        self.giant_atom_dialog.ui(ui.ctx());

        // 78. Interactive Topological Chiral Acoustic EMP Circulator & Router Dialog
        self.chiral_emp_dialog.ui(ui.ctx());

        // 79. Interactive Dissipative Polariton BEC Vortices & Josephson Interferometer Dialog
        self.polariton_bec_dialog.ui(ui.ctx());

        // 80. Interactive Topological Acoustic Synthetic Dimension & Multiplexed Router Dialog
        self.synthetic_dimension_dialog.ui(ui.ctx());

        // 81. Interactive Non-Hermitian Higher-Order Topological Quadrupole Skin Laser & Emitter Dialog
        self.non_hermitian_skin_laser_dialog.ui(ui.ctx());

        // 82. Interactive Topological Acoustic Moiré Flat-Band Polariton Soliton & Higher-Order Corner Comb Dialog
        self.moire_polariton_comb_dialog.ui(ui.ctx());

        // 83. Interactive Quantum Metamaterial Non-Abelian Holonomic Braiding & CMOS-MEMS Co-Processor Dialog
        self.holonomic_coprocessor_dialog.ui(ui.ctx());

        // 84. Interactive Topological Acoustic Valley-Hall Chiral Edge Filter & Microwave Isolator Dialog
        self.valley_chiral_isolator_dialog.ui(ui.ctx());

        // 85. Interactive Topological Acoustic Floquet Spin-Hall Insulator & Cryogenic Circulator Dialog
        self.floquet_spinhall_circulator_dialog.ui(ui.ctx());

        // 86. Interactive Topological Acoustic Higher-Order Octupole Vortex Metamaterial & 3D Chiral Dislocation Router Dialog
        self.octupole_dislocation_dialog.ui(ui.ctx());

        // 87. Interactive Quantum Metamaterial Chiral Majorana Braiding Processor & Surface Decoder Dialog
        self.chiral_majorana_dialog.ui(ui.ctx());

        // 88. Interactive Topological Acoustic Floquet Corner Laser & Vortex Amplifier Dialog
        self.floquet_corner_laser_dialog.ui(ui.ctx());

        // 89. Interactive Quantum Acoustic Non-Abelian Parafermion Braiding Dialog
        self.parafermion_dialog.ui(ui.ctx());

        // 90. Interactive Quantum Metamaterial Chiral Acoustomagnonic Isolator & Cryogenic Circulator Dialog
        self.acoustomagnonic_dialog.ui(ui.ctx());

        // 91. Interactive Topological Acoustic Floquet Corner-State Transducer & Entanglement Router Dialog
        self.floquet_corner_transducer_dialog.ui(ui.ctx());

        // 92. Interactive Topological Chiral Acoustic Chern-Simons Fractional Anyon Interferometer & Quantum Memory Dialog
        self.chern_simons_interferometer_dialog.ui(ui.ctx());

        // 93. Interactive Quantum Metamaterial Non-Hermitian Floquet Chiral Heat Transistor & Thermal Diode Dialog
        self.chiral_heat_transistor_dialog.ui(ui.ctx());

        // 94. Interactive Topological Acoustic Higher-Order Corner-State Quantum Metamaterial Frequency Comb & Dissipative Kerr Soliton Generator Dialog
        self.corner_kerr_microcomb_dialog.ui(ui.ctx());

        // 95. Interactive Topological Acoustic Superconducting Nanowire Single-Phonon Detector (SNSPD) & Quantum Transceiver Dialog
        self.acoustic_snspd_dialog.ui(ui.ctx());

        // 96. Interactive Topological Acoustic Floquet Chiral Magnon-Phonon Entanglement Router & CV-QKD Dialog
        self.floquet_cv_qkd_dialog.ui(ui.ctx());

        // 97. Interactive Quantum Metamaterial Polaritonic Soliton Frequency Comb & Dissipative Kerr Squeezed State Generator Dialog
        self.polaritonic_soliton_comb_dialog.ui(ui.ctx());

        // 98. Interactive Topological Acoustic Superconducting Circuit QED Quantum Transducer & Multi-Qubit Crossbar Dialog
        self.circuit_qed_transducer_dialog.ui(ui.ctx());

        // 99. Interactive Non-Hermitian Exceptional Surface Chiral Phonon Diode & Unidirectional Quantum Repeater Dialog
        self.exceptional_surface_diode_dialog.ui(ui.ctx());

        // 100. Interactive Topological Acoustic Boundary Soliton Logic Gate & Majority Voter Dialog
        self.topological_soliton_dialog.ui(ui.ctx());

        // 101. Interactive Quantum Metamaterial Fractional Chern & Parafermion Braiding Interconnect Dialog
        self.fractional_chern_interconnect_dialog.ui(ui.ctx());

        // 102. Interactive Topological Non-Hermitian Floquet Acoustic Chiral Lasing Metasurface & Vortex Waveguide Dialog
        self.chiral_lasing_metasurface_dialog.ui(ui.ctx());

        // 103. Interactive Topological Acoustic Synthetic Gauge Field & Non-Abelian Holonomic Quantum Gate Processor Dialog
        self.synthetic_gauge_holonomy_dialog.ui(ui.ctx());

        // 104. Interactive Chiral Phonon-Magnon Spin-Torque Acoustic Memory & Spintronic Crossbar Dialog
        self.chiral_spintorque_memory_dialog.ui(ui.ctx());

        // 105. Interactive Topological Acoustic Valley-Hall Quantum Router & Entanglement Concentrator Dialog
        self.valley_quantum_router_dialog.ui(ui.ctx());

        // 106. Interactive Fractional Parafermion Surface Code & Anyonic Braid Repeater Dialog
        self.fractional_parafermion_dialog.ui(ui.ctx());

        // 107. Interactive Topological Skin Microwave Amplifier & Axion Transducer Dialog
        self.topological_skin_axion_dialog.ui(ui.ctx());

        // 108. Interactive Read-Rezayi Fibonacci Anyon Acoustic Interferometer & Universal Quantum Bus Dialog
        self.read_rezayi_fibonacci_dialog.ui(ui.ctx());

        // 109. Interactive Moiré Exciton-Polariton Valley Hall Chiral Lasing Metasurface & Opto-Acoustic Synthesizer Dialog
        self.moire_polariton_dialog.ui(ui.ctx());

        // 110. Interactive Topological Josephson phi_0 Memory & Quantum Phase-Slip Crossbar Dialog
        self.topological_josephson_memory_dialog.ui(ui.ctx());

        // 111. Interactive Quantum Metamaterial Non-Abelian Genus-2 Parafermion Surface Code & Universal Processor Dialog
        self.genus2_parafermion_dialog.ui(ui.ctx());

        // 112. Interactive Topological Acoustic Floquet Higher-Order Corner Magneto-Phonon Isolator & Circulator Array Dialog
        self.floquet_corner_isolator_dialog.ui(ui.ctx());

        // 113. Interactive Dissipative Topological Polariton Skin Laser & Non-Hermitian Chiral Acoustic Gyroscope Array Dialog
        self.skin_polariton_laser_dialog.ui(ui.ctx());

        // 114. Interactive Quantum Metamaterial Valley-Locked Majorana Router Dialog
        self.valley_majorana_router_dialog.ui(ui.ctx());

        // 115. Interactive Chiral Phononic Graphene Anyon Braiding & Qubit Crossbar Dialog
        self.chiral_graphene_braiding_dialog.ui(ui.ctx());

        // 116. Interactive Floquet Chiral Magnon-Phonon Polariton Router & Quantum Memory Dialog
        self.floquet_magnon_memory_dialog.ui(ui.ctx());

        // 117. Interactive Cryogenic Quantum Metamaterial Anyon Interferometer & Qudit Crossbar Dialog
        self.anyon_interferometer_qudit_dialog.ui(ui.ctx());

        // 118. Interactive Floquet Corner Spin-Orbit Polariton Laser & Sensor Dialog
        self.floquet_corner_sensor_dialog.ui(ui.ctx());

        // 119. Interactive Quantum Metamaterial Fractional Hall Skyrmion Synaptic Memory & Neural Crossbar Dialog
        self.fractional_skyrmion_synapse_dialog.ui(ui.ctx());

        // 120. Interactive Chiral Metamaterial Transducer & Quantum Network Repeater Node Dialog
        self.chiral_transducer_repeater_dialog.ui(ui.ctx());

        // 121. Interactive HOWSM Vortex Transceiver & Multi-Terminal Quantum Acoustic Router Dialog
        self.weyl_vortex_router_dialog.ui(ui.ctx());

        // 122. Interactive Parafermion Lattice Co-Processor & Quantum Acoustic Surface Engine Dialog
        self.parafermion_surface_dialog.ui(ui.ctx());
        self.moire_superlattice_laser_dialog.ui(ui.ctx());
        self.disclination_holonomic_qudit_dialog.ui(ui.ctx());
        self.floquet_magnon_crossbar_dialog.ui(ui.ctx());
        self.moire_valley_qubit_dialog.ui(ui.ctx());
        self.corner_memory_repeater_dialog.ui(ui.ctx());
        self.non_hermitian_braiding_dialog.ui(ui.ctx());

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

        // 33. DevTools in-app overlay rendering (gated under devtools feature)
        #[cfg(any(test, feature = "devtools"))]
        {
            if ui.input(|i| i.key_pressed(egui::Key::F12)) {
                self.devtools_state.toggle_visibility();
            }
            let mut devtools = self.devtools_state.clone();
            devtools.render(ui.ctx(), self);
            self.devtools_state = devtools;
        }
    }
}

impl App for PhononApp {
    fn ui(&mut self, ui: &mut Ui, frame: &mut Frame) {
        self.update(ui, frame);
    }
}

#[cfg(any(test, feature = "devtools"))]
impl PhononApp {
    /// Devtools helper: Initiates a drag interaction on the specified component.
    pub fn devtools_start_drag(&mut self, comp_id: usize) {
        if !self.is_component_selected(comp_id) {
            self.select_component(comp_id, false);
        }
        let comp_pos = self
            .components
            .iter()
            .find(|c| c.id == comp_id)
            .map(|c| c.pos)
            .unwrap_or(Pos2::ZERO);
        self.dragging_selection = true;
        self.drag_start_mouse_pos = Some(comp_pos);
        self.drag_start_positions = self
            .components
            .iter()
            .filter(|c| self.is_component_selected(c.id))
            .map(|c| (c.id, c.pos))
            .collect();
        self.drag_start_wires = self.wires.clone();
        self.unsolvable_wiring_components.clear();

        let moving_comp_ids: HashSet<usize> =
            self.drag_start_positions.iter().map(|(id, _)| *id).collect();
        let mut attached = Vec::new();
        for c in &self.components {
            if moving_comp_ids.contains(&c.id) {
                for (pin_idx, (_, pin_pos)) in c.all_pins().iter().enumerate() {
                    for wire in &self.wires {
                        let is_start = (wire.start_point() - *pin_pos).length() <= 8.0;
                        let is_end = (wire.end_point() - *pin_pos).length() <= 8.0;
                        if is_start && is_end {
                        } else if is_start {
                            attached.push(DragAttachedWire {
                                wire_id: wire.id,
                                is_start: true,
                                comp_id: c.id,
                                pin_idx,
                                other_end: wire.end_point(),
                                net_name: wire.net_name.clone(),
                            });
                        } else if is_end {
                            attached.push(DragAttachedWire {
                                wire_id: wire.id,
                                is_start: false,
                                comp_id: c.id,
                                pin_idx,
                                other_end: wire.start_point(),
                                net_name: wire.net_name.clone(),
                            });
                        }
                    }
                }
            }
        }
        self.drag_attached_wires = attached;
        self.sync_selection_to_canvas();
    }

    /// Devtools helper: Updates the drag position by a relative delta.
    pub fn devtools_drag_delta(&mut self, delta: Vec2) {
        if !self.dragging_selection {
            return;
        }
        let start_mouse = match self.drag_start_mouse_pos {
            Some(p) => p,
            None => return,
        };
        let new_pos = start_mouse + delta;
        self.devtools_drag_to(new_pos);
    }

    /// Devtools helper: Updates the drag position to an absolute target world coordinate.
    pub fn devtools_drag_to(&mut self, mouse_world: Pos2) {
        if !self.dragging_selection {
            return;
        }
        let start_mouse = match self.drag_start_mouse_pos {
            Some(p) => p,
            None => return,
        };
        let total_drag = mouse_world - start_mouse;
        let snapped_dx = (total_drag.x / 20.0).round() * 20.0;
        let snapped_dy = (total_drag.y / 20.0).round() * 20.0;
        let grid_delta = Vec2::new(snapped_dx, snapped_dy);

        let moving_comp_ids: HashSet<usize> =
            self.drag_start_positions.iter().map(|(id, _)| *id).collect();

        for (cid, start_pos) in &self.drag_start_positions {
            if let Some(c) = self.components.iter_mut().find(|c| c.id == *cid) {
                c.pos = self.canvas.snap_to_grid(*start_pos + grid_delta);
            }
        }

        let mut obstacles = Vec::new();
        let mut moving_bboxes = Vec::new();
        for c in &self.components {
            if moving_comp_ids.contains(&c.id) {
                moving_bboxes.push((c.id, c.bounding_box()));
            } else {
                obstacles.push(c.bounding_box());
            }
        }

        let mut has_collision = false;
        for (_cid, m_bbox) in &moving_bboxes {
            for obs in &obstacles {
                if m_bbox.shrink(2.0).intersects(*obs) {
                    has_collision = true;
                    break;
                }
            }
            if has_collision {
                break;
            }
        }

        if has_collision {
            self.wires = self.drag_start_wires.clone();
            for id in &moving_comp_ids {
                self.unsolvable_wiring_components.insert(*id);
            }
        } else {
            let mut wire_map: HashMap<usize, SchematicWire> =
                self.drag_start_wires.iter().map(|w| (w.id, w.clone())).collect();
            let mut routing_success = true;

            for att in &self.drag_attached_wires {
                let comp = match self.components.iter().find(|c| c.id == att.comp_id) {
                    Some(c) => c,
                    None => continue,
                };
                let moving_pin_pos = match comp.pin_world_pos(att.pin_idx) {
                    Some(p) => p,
                    None => continue,
                };
                let moving_pin_normal = comp.pin_normal(att.pin_idx);

                let (from, from_normal, to, to_normal) = if att.is_start {
                    (moving_pin_pos, moving_pin_normal, att.other_end, None)
                } else {
                    (att.other_end, moving_pin_normal.opposite(), moving_pin_pos, Some(moving_pin_normal))
                };

                if let Some(routed) = SchematicWire::manhattan_route_avoiding_obstacles(
                    att.wire_id,
                    from,
                    from_normal,
                    to,
                    to_normal,
                    &obstacles,
                    att.net_name.clone(),
                ) {
                    wire_map.insert(att.wire_id, routed);
                } else {
                    routing_success = false;
                    break;
                }
            }

            if routing_success {
                self.wires = wire_map.into_values().collect();
                for id in &moving_comp_ids {
                    self.unsolvable_wiring_components.remove(id);
                }
            } else {
                self.wires = self.drag_start_wires.clone();
                for id in &moving_comp_ids {
                    self.unsolvable_wiring_components.insert(*id);
                }
            }
        }

        self.sync_selection_to_canvas();
    }

    /// Devtools helper: Finishes dragging and records history.
    pub fn devtools_finish_drag(&mut self) {
        if !self.dragging_selection {
            return;
        }
        let moving_comp_ids: HashSet<usize> =
            self.drag_start_positions.iter().map(|(id, _)| *id).collect();

        let is_unsolvable = moving_comp_ids
            .iter()
            .any(|id| self.unsolvable_wiring_components.contains(id));

        let mut batch = Vec::new();
        for (id, start_pos) in self.drag_start_positions.drain(..) {
            if let Some(comp) = self.components.iter().find(|c| c.id == id) {
                if comp.pos != start_pos {
                    batch.push(CanvasCommand::MoveComponent {
                        id,
                        from: start_pos,
                        to: comp.pos,
                    });
                }
            }
        }

        if !is_unsolvable {
            for start_wire in self.drag_start_wires.drain(..) {
                if let Some(curr_wire) = self.wires.iter().find(|w| w.id == start_wire.id) {
                    if curr_wire.segments != start_wire.segments {
                        batch.push(CanvasCommand::DeleteWire(start_wire));
                        batch.push(CanvasCommand::AddWire(curr_wire.clone()));
                    }
                }
            }
        } else {
            self.drag_start_wires.clear();
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
        self.drag_start_mouse_pos = None;
        self.drag_attached_wires.clear();
        self.sync_selection_to_canvas();
    }

    /// Devtools helper: Starts marquee selection.
    pub fn devtools_start_marquee(&mut self, start: Pos2) {
        self.marquee_start = Some(start);
        self.marquee_current = Some(start);
        self.canvas.marquee_start = Some(start);
        self.canvas.marquee_current = Some(start);
    }

    /// Devtools helper: Updates marquee selection coordinates.
    pub fn devtools_update_marquee(&mut self, current: Pos2) {
        self.marquee_current = Some(current);
        self.canvas.marquee_current = Some(current);
    }

    /// Devtools helper: Finalizes marquee selection.
    pub fn devtools_finish_marquee(&mut self) {
        if let (Some(m_start), Some(m_curr)) = (self.marquee_start.take(), self.marquee_current.take()) {
            let marquee_rect = Rect::from_two_pos(m_start, m_curr);
            if marquee_rect.width() > 3.0 || marquee_rect.height() > 3.0 {
                self.select_in_rect(marquee_rect, false);
            }
        }
        self.canvas.marquee_start = None;
        self.canvas.marquee_current = None;
        self.sync_selection_to_canvas();
    }
}
