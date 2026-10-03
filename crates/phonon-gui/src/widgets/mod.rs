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

pub use cluster_dashboard_dialog::ClusterDashboardDialog;
pub use dynamics_status::DynamicsStatusBadge;
pub use exceptional_point_dialog::ExceptionalPointDialog;
pub use icon::render_phonon_icon;
pub use monte_carlo_dialog::{MonteCarloParamEntry, MonteCarloYieldDialog};
pub use neuromorphic_snn_dialog::NeuromorphicSnnDialog;
pub use palette::ComponentPalette;
pub use polariton_cavity_dialog::PolaritonCavityDialog;
pub use sensitivity_dialog::SensitivityDialog;
pub use smith_chart_dialog::{HoveredMarker, SmithChartDialog};
pub use symbol_editor::{SymbolEditorDialog, SymbolEditorTool};
pub use thermal_floorplan_dialog::ThermalFloorplanDialog;
pub use top_frame::{render_top_frame, render_top_frame_with_app, TopFrameAction, TopFrameConfig};
