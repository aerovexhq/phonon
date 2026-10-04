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
pub mod command_palette;
pub mod floating_toolbar;

pub use command_palette::CommandPalette;
pub use floating_toolbar::{FloatingToolbarAction, FloatingToolbarState};
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
pub use palette::ComponentPalette;
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

