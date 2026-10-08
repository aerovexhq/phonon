#![deny(unsafe_code)]

//! GUI widgets, visual status indicators, iconography, and custom window frames for Phonon Studio.

pub mod card_3d_viewport;
pub mod cluster_dashboard_dialog;
pub mod dynamics_status;
pub mod exceptional_point_dialog;
pub mod icon;
pub mod monte_carlo_dialog;
pub mod neuromorphic_snn_dialog;
pub mod palette;
pub mod polariton_cavity_dialog;
pub mod sensitivity_dialog;
pub mod smith_chart_dialog;
pub mod symbol_editor;
pub mod thermal_floorplan_dialog;
pub mod top_frame;
pub mod weyl_semimetal_dialog;
pub mod fqh_braiding_dialog;
pub mod jtwpa_dialog;
pub mod floquet_metasurface_dialog;
pub mod holonomic_processor_dialog;
pub mod twisted_moire_dialog;
pub mod chern_circulator_dialog;
pub mod kerr_microcomb_dialog;
pub mod exceptional_surface_dialog;
pub mod soti_corner_dialog;
pub mod lieb_lattice_dialog;
pub mod axion_insulator_dialog;
pub mod floquet_time_crystal_dialog;
pub mod command_palette;
pub mod confirmation_modal;
pub mod floating_toolbar;
pub mod pill_badge;
pub mod preferences_dialog;
pub mod project_dialog;
pub mod subcircuit_dialog;
pub mod lua_console_dialog;
pub mod quantum_braiding_lattice_dialog;
pub mod optomechanical_squeezing_dialog;
pub mod skyrmion_router_dialog;
pub mod acoustic_soliton_dialog;
pub mod valley_multiplexer_dialog;
pub mod non_hermitian_skin_dialog;
pub mod quadrupole_shg_dialog;
pub mod synthetic_4d_dialog;
pub mod pt_symmetric_dialog;
pub mod acoustic_bic_dialog;
pub mod euler_acoustic_dialog;
pub mod octupole_insulator_dialog;
pub mod aah_quasicrystal_dialog;
pub mod valley_hall_vortex_dialog;
pub mod skyrmion_deflector_dialog;
pub mod floquet_frequency_dialog;
pub mod corner_laser_dialog;
pub mod metasurface_hologram_dialog;
pub mod majorana_surface_code_dialog;
pub mod optomechanical_transducer_dialog;
pub mod corner_polariton_microcomb_dialog;
pub mod directional_radiation_dialog;
pub mod atmospheric_neutron_dialog;
pub mod thermal_vacuum_dialog;
pub mod space_avionics_bus_dialog;
pub mod rhbd_self_healing_dialog;
pub mod production_economics_dialog;
pub mod chiplet_packaging_dialog;
pub mod electrothermal_throttling_dialog;
pub mod pdn_droop_dialog;
pub mod silicon_aging_dialog;
pub mod wafer_yield_dialog;
pub mod dse_optimization_dialog;
pub mod silicon_lifecycle_dialog;
pub mod wasm_optimization_dialog;
pub mod pwa_offline_dialog;
pub mod desktop_ipc_dialog;
pub mod webrtc_mesh_dialog;
pub mod webgpu_spice_dialog;
pub mod wavepacket_scattering_dialog;
pub mod skyrmion_reservoir_dialog;
pub mod phonon_magnon_dialog;
pub mod quadrupole_parametric_dialog;
pub mod non_hermitian_sensor_dialog;
pub mod corner_doubler_dialog;
pub mod universal_braiding_dialog;
pub mod chiral_circulator_dialog;
pub mod josephson_parametric_dialog;
pub mod optomagnonic_comb_dialog;
pub mod floquet_sensor_dialog;
pub mod giant_atom_dialog;
pub mod chiral_emp_dialog;
pub mod polariton_bec_dialog;
pub mod synthetic_dimension_dialog;
pub mod non_hermitian_skin_laser_dialog;
pub mod moire_polariton_comb_dialog;
pub mod holonomic_coprocessor_dialog;
pub mod valley_chiral_isolator_dialog;
pub mod floquet_spinhall_circulator_dialog;
pub mod octupole_dislocation_dialog;
pub mod chiral_majorana_dialog;
pub mod floquet_corner_laser_dialog;
pub mod parafermion_dialog;
pub mod chiral_acoustomagnonic_dialog;
pub mod floquet_corner_transducer_dialog;
pub mod chern_simons_interferometer_dialog;
pub mod chiral_heat_transistor_dialog;
pub mod corner_kerr_microcomb_dialog;
pub mod acoustic_snspd_dialog;
pub mod floquet_cv_qkd_dialog;
pub mod polaritonic_soliton_comb_dialog;
pub mod circuit_qed_transducer_dialog;
pub mod exceptional_surface_diode_dialog;
pub mod topological_soliton_dialog;
pub mod fractional_chern_interconnect_dialog;
pub mod chiral_lasing_metasurface_dialog;
pub mod synthetic_gauge_holonomy_dialog;
pub mod chiral_spintorque_memory_dialog;
pub mod valley_quantum_router_dialog;
pub mod fractional_parafermion_dialog;
pub mod topological_skin_axion_dialog;
pub mod read_rezayi_fibonacci_dialog;
pub mod moire_polariton_dialog;
pub mod topological_josephson_memory_dialog;
pub mod genus2_parafermion_dialog;
pub mod floquet_corner_isolator_dialog;
pub mod skin_polariton_laser_dialog;
pub mod valley_majorana_router_dialog;
pub mod chiral_graphene_braiding_dialog;
pub mod floquet_magnon_memory_dialog;

pub use pill_badge::{
    proportional_zoom_scale, render_dual_telemetry_pill, render_pill_badge, PillBadgeStyle,
};
pub use subcircuit_dialog::{SubcircuitDialogAction, SubcircuitPackageDialog};
pub use command_palette::CommandPalette;
pub use confirmation_modal::{
    ConfirmationDecision, ConfirmationModal, DemoCircuitKind, PendingAction,
};
pub use floating_toolbar::{FloatingToolbarAction, FloatingToolbarState, FloatingToolbarTheme};
pub use preferences_dialog::{PreferencesDialog, PreferencesTab};
pub use project_dialog::{ProjectDialog, ProjectDialogAction, ProjectDialogMode};
pub use card_3d_viewport::{Camera3D, Card3dViewport};
pub use chern_circulator_dialog::ChernCirculatorDialog;
pub use cluster_dashboard_dialog::ClusterDashboardDialog;
pub use dynamics_status::DynamicsStatusBadge;
pub use exceptional_point_dialog::ExceptionalPointDialog;
pub use exceptional_surface_dialog::ExceptionalSurfaceDialog;
pub use floquet_metasurface_dialog::FloquetMetasurfaceDialog;
pub use fqh_braiding_dialog::FqhBraidingDialog;
pub use holonomic_processor_dialog::HolonomicProcessorDialog;
pub use icon::render_phonon_icon;
pub use jtwpa_dialog::JtwpaDialog;
pub use kerr_microcomb_dialog::KerrMicrocombDialog;
pub use monte_carlo_dialog::{MonteCarloParamEntry, MonteCarloYieldDialog};
pub use neuromorphic_snn_dialog::NeuromorphicSnnDialog;
pub use palette::{ComponentPalette, PaletteAction, SidebarTab};
pub use polariton_cavity_dialog::PolaritonCavityDialog;
pub use sensitivity_dialog::SensitivityDialog;
pub use smith_chart_dialog::{HoveredMarker, SmithChartDialog};
pub use symbol_editor::{SymbolEditorDialog, SymbolEditorTool};
pub use thermal_floorplan_dialog::ThermalFloorplanDialog;
pub use top_frame::{render_top_frame, render_top_frame_with_app, TopFrameAction, TopFrameConfig};
pub use twisted_moire_dialog::TwistedMoireDialog;
pub use weyl_semimetal_dialog::{SemimetalMode, WeylSemimetalDialog};
pub use soti_corner_dialog::{SotiCornerDialog, SpatialModeSelection};
pub use lieb_lattice_dialog::{LiebLatticeDialog, LiebPlotTab, LiebSpatialModeSelection};
pub use axion_insulator_dialog::{AxionInsulatorDialog, AxionPlotTab, AxionSpatialViewMode};
pub use floquet_time_crystal_dialog::{FloquetPlotTab, FloquetTimeCrystalDialog};
pub use lua_console_dialog::{LuaConsoleDialog, ScriptPreset};
pub use quantum_braiding_lattice_dialog::{BraidingDialogTab, QuantumBraidingLatticeDialog};
pub use optomechanical_squeezing_dialog::{
    OptomechDialogTab, OptomechanicalSqueezingDialog, WignerColormap,
};
pub use skyrmion_router_dialog::{SkyrmionDialogTab, SkyrmionRouterDialog};
pub use acoustic_soliton_dialog::{AcousticSolitonDialog, SolitonDialogTab};
pub use valley_multiplexer_dialog::{ValleyDialogTab, ValleyMultiplexerDialog};
pub use non_hermitian_skin_dialog::{NonHermitianSkinDialog, NonHermitianSkinDialogTab};
pub use quadrupole_shg_dialog::{QuadrupoleShgDialog, QuadrupoleShgDialogTab};
pub use synthetic_4d_dialog::{Synthetic4dDialog, Synthetic4dDialogTab};
pub use pt_symmetric_dialog::{PtSymmetricDialog, PtSymmetricDialogTab};
pub use acoustic_bic_dialog::{AcousticBicDialog, AcousticBicDialogTab};
pub use euler_acoustic_dialog::{EulerAcousticDialog, EulerAcousticDialogTab};
pub use octupole_insulator_dialog::{OctupoleDialogTab, OctupoleInsulatorDialog};
pub use aah_quasicrystal_dialog::{AahDialogTab, AahQuasicrystalDialog};
pub use valley_hall_vortex_dialog::{ValleyHallDialogTab, ValleyHallVortexDialog};
pub use skyrmion_deflector_dialog::{SkyrmionDeflectorDialog, SkyrmionDeflectorTab};
pub use floquet_frequency_dialog::{FloquetFrequencyDialog, FloquetFrequencyTab};
pub use corner_laser_dialog::{CornerLaserDialog, CornerLaserTab};
pub use metasurface_hologram_dialog::{MetasurfaceHologramDialog, MetasurfaceHologramTab};
pub use majorana_surface_code_dialog::{MajoranaSurfaceCodeDialog, MajoranaSurfaceCodeTab};
pub use optomechanical_transducer_dialog::{
    OptomechanicalTransducerDialog, OptomechanicalTransducerTab,
};
pub use corner_polariton_microcomb_dialog::{
    CornerPolaritonMicrocombDialog, CornerPolaritonMicrocombTab,
};
pub use directional_radiation_dialog::{DirectionalRadiationDialog, DirectionalRadiationTab};
pub use atmospheric_neutron_dialog::{AtmosphericNeutronDialog, AtmosphericNeutronTab};
pub use thermal_vacuum_dialog::{ThermalVacuumDialog, ThermalVacuumTab};
pub use space_avionics_bus_dialog::{SpaceAvionicsBusDialog, SpaceAvionicsBusTab};
pub use rhbd_self_healing_dialog::{RhbdSelfHealingDialog, RhbdTab};
pub use production_economics_dialog::{EconomicsTab, ProductionEconomicsDialog};
pub use chiplet_packaging_dialog::{ChipletPackagingDialog, PackagingTab};
pub use electrothermal_throttling_dialog::{ElectrothermalThrottlingDialog, ThrottlingTab};
pub use pdn_droop_dialog::{PdnDroopDialog, PdnTab};
pub use silicon_aging_dialog::{AgingTab, SiliconAgingDialog};
pub use wafer_yield_dialog::{WaferMapColorMode, WaferYieldDialog, WaferYieldTab};
pub use dse_optimization_dialog::{DseOptimizationDialog, DseTab};
pub use silicon_lifecycle_dialog::{SiliconLifecycleDialog, SlmTab};
pub use wasm_optimization_dialog::{WasmOptTab, WasmOptimizationDialog};
pub use pwa_offline_dialog::{PwaOfflineDialog, PwaTab};
pub use desktop_ipc_dialog::{
    DesktopIpcDialog, DesktopIpcTab, GerberLayer, IpcOpcode, IpcPacketError, IpcPacketHeader,
    PlatformAuditItem, compute_adler32, decode_ipc_packet, encode_ipc_packet, IPC_MAGIC,
    IPC_VERSION,
};
pub use webrtc_mesh_dialog::{
    CrdtOpKind, CrdtOperation, DistributedSimChunk, LamportTimestamp, MeshAuditCriterion,
    MeshPeerNode, PeerConnectionState, PeerPresence, WebRtcMeshDialog, WebRtcMeshTab,
};
pub use webgpu_spice_dialog::{
    lttb_decimate, CsrMatrix, GpuBackendKind, WebGpuAuditCriterion, WebGpuSpiceDialog,
    WebGpuSpiceTab,
};
pub use wavepacket_scattering_dialog::{
    WavepacketAuditCriterion, WavepacketScatteringDialog, WavepacketScatteringTab,
};
pub use skyrmion_reservoir_dialog::{
    SkyrmionReservoirDialog, SkyrmionReservoirTab, SpintronicAuditCriterion,
};
pub use phonon_magnon_dialog::{
    PhononMagnonDialog, PhononMagnonTab, PolaritonAuditCriterion,
};
pub use quadrupole_parametric_dialog::{
    CanvasColormap, QuadrupoleAuditCriterion, QuadrupoleParametricDialog,
    QuadrupoleParametricTab, RealSpaceMode,
};
pub use non_hermitian_sensor_dialog::{
    NhSensorAuditCriterion, NonHermitianSensorDialog, NonHermitianSensorTab,
};
pub use corner_doubler_dialog::{
    CornerDoublerAuditCriterion, CornerDoublerDialog, CornerDoublerTab, DialogColormap,
    DisplayModeType,
};
pub use universal_braiding_dialog::{UniversalBraidingDialog, UniversalBraidingTab};
pub use chiral_circulator_dialog::{ChiralCirculatorDialog, ChiralCirculatorTab};
pub use josephson_parametric_dialog::{JosephsonParametricDialog, JosephsonParametricTab};
pub use optomagnonic_comb_dialog::{OptomagnonicCombDialog, OptomagnonicCombTab};
pub use floquet_sensor_dialog::{FloquetSensorDialog, FloquetSensorTab};
pub use giant_atom_dialog::{GiantAtomDialog, GiantAtomDialogTab};
pub use chiral_emp_dialog::{ChiralEmpDialog, ChiralEmpDialogTab};
pub use polariton_bec_dialog::{PolaritonBecDialog, PolaritonBecTab};
pub use synthetic_dimension_dialog::{SyntheticDimensionDialog, SyntheticDimensionTab};
pub use non_hermitian_skin_laser_dialog::{NonHermitianSkinLaserDialog, SkinLaserTab};
pub use moire_polariton_comb_dialog::{MoireCombTab, MoirePolaritonCombDialog};
pub use holonomic_coprocessor_dialog::{HolonomicCoprocessorDialog, HolonomicCoprocessorTab};
pub use valley_chiral_isolator_dialog::{ValleyChiralIsolatorDialog, ValleyChiralTab};
pub use floquet_spinhall_circulator_dialog::{
    FloquetSpinHallCirculatorDialog, FloquetSpinHallTab,
};
pub use octupole_dislocation_dialog::{
    OctupoleDislocationDialog, OctupoleDislocationTab,
};
pub use chiral_majorana_dialog::{
    ChiralMajoranaDialog, ChiralMajoranaTab,
};
pub use floquet_corner_laser_dialog::{
    FloquetCornerLaserDialog, FloquetCornerLaserTab,
};
pub use parafermion_dialog::{
    ParafermionDialog, ParafermionTab,
};
pub use chiral_acoustomagnonic_dialog::{
    AcoustomagnonicTab, ChiralAcoustomagnonicDialog,
};
pub use floquet_corner_transducer_dialog::{
    CornerTransducerTab, FloquetCornerTransducerDialog,
};
pub use chern_simons_interferometer_dialog::{
    ChernSimonsInterferometerDialog, ChernSimonsTab,
};
pub use chiral_heat_transistor_dialog::{
    ChiralHeatTransistorDialog, HeatTransistorTab,
};
pub use corner_kerr_microcomb_dialog::{
    CornerCombTab, CornerKerrMicrocombDialog,
};
pub use acoustic_snspd_dialog::{
    AcousticSnspdDialog, SnspdTab,
};
pub use floquet_cv_qkd_dialog::{
    CvQkdTab, FloquetCvQkdDialog,
};
pub use polaritonic_soliton_comb_dialog::{
    PolaritonicCombTab, PolaritonicSolitonCombDialog,
};
pub use circuit_qed_transducer_dialog::{
    CircuitQedTab, CircuitQedTransducerDialog,
};
pub use exceptional_surface_diode_dialog::{
    ExceptionalSurfaceDiodeDialog, ExceptionalSurfaceDiodeTab,
};
pub use topological_soliton_dialog::{
    TopologicalSolitonDialog, TopologicalSolitonTab,
};
pub use fractional_chern_interconnect_dialog::{
    FractionalChernInterconnectDialog, FractionalChernTab,
};
pub use chiral_lasing_metasurface_dialog::{
    ChiralLasingMetasurfaceDialog, ChiralLasingTab,
};
pub use synthetic_gauge_holonomy_dialog::{
    SyntheticGaugeHolonomyDialog, SyntheticGaugeTab,
};
pub use chiral_spintorque_memory_dialog::{
    ChiralSpinTorqueMemoryDialog, MemoryDialogTab,
};
pub use valley_quantum_router_dialog::{
    ValleyQuantumRouterDialog, ValleyRouterTab,
};
pub use fractional_parafermion_dialog::{
    FractionalParafermionDialog, ParafermionDialogTab,
};
pub use topological_skin_axion_dialog::{
    SkinAxionDialogTab, TopologicalSkinAxionDialog,
};
pub use read_rezayi_fibonacci_dialog::{
    ReadRezayiDialogTab, ReadRezayiFibonacciDialog,
};
pub use moire_polariton_dialog::{
    MoirePolaritonDialog, MoirePolaritonDialogTab,
};
pub use topological_josephson_memory_dialog::{
    TopologicalJosephsonMemoryDialog, TopologicalJosephsonTab,
};
pub use genus2_parafermion_dialog::{
    Genus2ParafermionDialog, Genus2ParafermionTab,
};
pub use floquet_corner_isolator_dialog::{
    FloquetCornerIsolatorDialog, FloquetCornerIsolatorTab,
};
pub use skin_polariton_laser_dialog::{
    SkinPolaritonLaserDialog, SkinPolaritonLaserTab,
};
pub use valley_majorana_router_dialog::{
    ValleyMajoranaRouterDialog, ValleyMajoranaRouterTab,
};
pub use chiral_graphene_braiding_dialog::{
    ChiralGrapheneBraidingDialog, ChiralGrapheneBraidingTab,
};
pub use floquet_magnon_memory_dialog::{
    FloquetMagnonMemoryDialog, FloquetMagnonMemoryTab,
};



