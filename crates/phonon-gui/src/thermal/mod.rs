//! 2D/3D thermal network heatmaps, colormaps, and junction temperature overlay.

pub mod heatmap;

pub use heatmap::{sample_colormap, Colormap, ThermalOverlay};
