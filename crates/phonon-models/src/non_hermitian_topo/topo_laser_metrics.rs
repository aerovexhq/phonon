//! Topological acoustic laser arrays, chiral mode selection,
//! and unidirectional acoustic sound amplifiers.

/// Evaluated metrics for topological phonon lasers and directional sound amplifiers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TopologicalPhononLaserMetrics {
    /// Topological acoustic laser threshold pump power in milliwatts ($\\text{mW}$) ($\\le 5.0\\text{ mW}$).
    pub laser_threshold_power_mw: f64,
    /// Side-mode suppression ratio (SMSR) in decibels $\\ge 25.0\\text{ dB}$.
    pub side_mode_suppression_ratio_db: f64,
    /// Unidirectional sound amplification gain contrast in decibels $\\mathcal{G}_{\\mathrm{dir}} \\ge 25.0\\text{ dB}$.
    pub directional_amplification_gain_db: f64,
    /// Resonant single-mode topological lasing frequency in $\\text{GHz}$.
    pub lasing_frequency_ghz: f64,
}
