#![deny(unsafe_code)]

//! GUI widgets, visual status indicators, iconography, and custom window frames for Phonon Studio.

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

pub use pill_badge::{
    proportional_zoom_scale, render_dual_telemetry_pill, render_pill_badge, PillBadgeStyle,
};
pub use subcircuit_dialog::{SubcircuitDialogAction, SubcircuitPackageDialog};
pub use command_palette::CommandPalette;
pub use confirmation_modal::{
    ConfirmationDecision, ConfirmationModal, DemoCircuitKind, PendingAction,
};
pub use floating_toolbar::{FloatingToolbarAction, FloatingToolbarState};
pub use preferences_dialog::{PreferencesDialog, PreferencesTab};
pub use project_dialog::{ProjectDialog, ProjectDialogAction, ProjectDialogMode};
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


