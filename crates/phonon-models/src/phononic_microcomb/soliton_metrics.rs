//! Dissipative acoustic soliton contrast, repetition rate stability,
//! and phononic clock synthesis metrics.

/// Evaluated metrics for acoustic soliton stability and phononic logic.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcousticSolitonMetrics {
    /// Peak acoustic soliton circulating power in Watts ($W$).
    pub peak_soliton_power_w: f64,
    /// Soliton on/off switching extinction contrast in decibels $\\mathcal{R}_{\\mathrm{soliton}} \\ge 20.0\\text{ dB}$.
    pub soliton_contrast_db: f64,
    /// Total number of coherent frequency comb lines $N_{\\mathrm{lines}} \\ge 50$.
    pub comb_line_count: usize,
    /// Fractional frequency instability (Allan deviation floor $\\sigma_y \\le 10^{-11}$).
    pub fractional_instability: f64,
}
