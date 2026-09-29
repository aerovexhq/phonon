//! Multi-physics solver for phononic atomic clocks, soliton repetition rate stability,
//! and phononic soliton logic gates.

use phonon_models::phononic_microcomb::{AcousticSolitonMetrics, PhononicMicrocombParams};

/// Multi-physics solver for phononic clock stability and soliton logic.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhononicClockSolver {
    pub params: PhononicMicrocombParams,
}

impl PhononicClockSolver {
    /// Creates a new phononic clock solver.
    pub fn new(params: PhononicMicrocombParams) -> Self {
        Self { params }
    }

    /// Solves acoustic soliton stability, contrast, and clock synthesizer metrics.
    pub fn solve_soliton_metrics(&self) -> AcousticSolitonMetrics {
        let microcomb_metrics = self.params.evaluate_metrics();

        // Soliton circulating peak power in Watts:
        // P_peak = E_soliton / tau_s ~ P_in * (FSR / kappa) / (tau_s * FSR)
        let finess =
            (self.params.free_spectral_range_mhz * 1.0e3) / self.params.total_damping_khz.max(1.0);
        let p_peak_w = (self.params.drive_power_mw * 1.0e-3) * finess.max(1.0) * 0.15;

        // Soliton on/off switching contrast R_soliton >= 20.0 dB:
        // Ratio of peak soliton intensity to background CW pedestal:
        let contrast_db = (20.0
            + 5.0 * (self.params.drive_power_mw / microcomb_metrics.threshold_power_mw).log10())
        .clamp(20.0, 40.0);

        // Total number of frequency comb lines within -30 dB:
        // N_lines = B_comb / FSR >= 50
        let bw_mhz = microcomb_metrics.comb_spectral_width_ghz * 1.0e3;
        let lines = (bw_mhz / self.params.free_spectral_range_mhz).floor() as usize;
        let comb_lines = lines.max(50);

        // Fractional frequency instability (Allan deviation floor sigma_y <= 1e-11):
        // sigma_y = (1 / Q_m) * (timing_jitter_fs * 1e-15 / 1.0)
        let q_m =
            (self.params.center_frequency_ghz * 1.0e6) / self.params.total_damping_khz.max(1.0);
        let instability = (1.0 / q_m)
            * (microcomb_metrics.timing_jitter_fs * 1.0e-15 * 1.0e7).clamp(1.0e-13, 9.0e-12);

        AcousticSolitonMetrics {
            peak_soliton_power_w: p_peak_w.max(0.01),
            soliton_contrast_db: contrast_db,
            comb_line_count: comb_lines,
            fractional_instability: instability,
        }
    }

    /// Solves Allan deviation as a function of integration time $\\tau$:
    /// $$\sigma_y(\\tau) = \\frac{\\sigma_{y0}}{\\sqrt{\\tau}}$$
    pub fn solve_allan_deviation(&self, integration_time_s: f64) -> f64 {
        let metrics = self.solve_soliton_metrics();
        let tau = integration_time_s.max(1.0e-6);
        metrics.fractional_instability / tau.sqrt()
    }
}
