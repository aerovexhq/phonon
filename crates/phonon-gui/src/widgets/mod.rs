#![deny(unsafe_code)]

//! GUI widgets, visual status indicators, iconography, and custom window frames for Phonon Studio.

pub mod dynamics_status;
pub mod icon;
pub mod top_frame;

pub use dynamics_status::DynamicsStatusBadge;
pub use icon::render_phonon_icon;
pub use top_frame::{render_top_frame, render_top_frame_with_app, TopFrameAction, TopFrameConfig};
