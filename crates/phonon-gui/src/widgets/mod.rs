#![deny(unsafe_code)]

//! GUI widgets, visual status indicators, iconography, and custom window frames for Phonon Studio.

pub mod dynamics_status;
pub mod icon;
pub mod monte_carlo_dialog;
pub mod palette;
pub mod sensitivity_dialog;
pub mod symbol_editor;
pub mod top_frame;

pub use dynamics_status::DynamicsStatusBadge;
pub use icon::render_phonon_icon;
pub use monte_carlo_dialog::{MonteCarloParamEntry, MonteCarloYieldDialog};
pub use palette::ComponentPalette;
pub use sensitivity_dialog::SensitivityDialog;
pub use symbol_editor::{SymbolEditorDialog, SymbolEditorTool};
pub use top_frame::{render_top_frame, render_top_frame_with_app, TopFrameAction, TopFrameConfig};

