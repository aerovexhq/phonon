//! Multi-physics solver for phononic Lugiato-Lefever non-linear envelope equations,
//! dissipative Kerr soliton formation, and octave-spanning acoustic frequency combs.

use phonon_models::phononic_microcomb::{MicrocombMetrics, PhononicMicrocombParams};
use std::f64::consts::PI;

/// Multi-physics solver for dissipative phononic Kerr soliton microcombs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LugiatoLefeverSolver {
    pub params: PhononicMicrocombParams,
}

impl LugiatoLefeverSolver {
    /// Creates a new Lugiato-Lefever solver.
    pub fn new(params: PhononicMicrocombParams) -> Self {
        Self { params }
    }

    /// Solves microcomb generation and dissipative soliton metrics.
    pub fn solve_microcomb_metrics(&self) -> MicrocombMetrics {
        self.params.evaluate_metrics()
    }

    /// Solves the coherent frequency comb spectral envelope $P(\\mu)$ in $\\text{dBm}$:
    /// $$P(\\mu) = P_0 \\operatorname{sech}^2\\left(\\frac{\\mu D_1 \\tau_s}{2}\\right)$$
    pub fn solve_comb_spectrum(&self, max_modes: usize) -> Vec<(i32, f64)> {
        let metrics = self.solve_microcomb_metrics();
        let tau_s = metrics.soliton_duration_ps * 1.0e-12;
        let d1_rad_s = self.params.free_spectral_range_mhz * 1.0e6 * 2.0 * PI;
        let p_peak_dbm = 10.0 * (self.params.drive_power_mw).max(0.1).log10();

        let n = max_modes as i32;
        let mut spectrum = Vec::with_capacity((2 * n + 1) as usize);

        for mu in -n..=n {
            let arg = (mu as f64 * d1_rad_s * tau_s * 0.5).abs().min(15.0);
            let sech_val = 1.0 / arg.cosh();
            let lin_power = sech_val.powi(2);
            let p_dbm = p_peak_dbm + 10.0 * lin_power.max(1e-9).log10();
            spectrum.push((mu, p_dbm));
        }

        spectrum
    }

    /// Solves the temporal soliton circulating intensity profile $|\\psi(\\theta)|^2$:
    pub fn solve_soliton_temporal_profile(&self, num_points: usize) -> Vec<(f64, f64)> {
        let metrics = self.solve_microcomb_metrics();
        let n = num_points.max(8);
        let tau_s_ps = metrics.soliton_duration_ps;
        let period_ps = (1.0 / (self.params.free_spectral_range_mhz * 1.0e6)) * 1.0e12;
        let theta_s = 2.0 * PI * (tau_s_ps / period_ps);

        let mut profile = Vec::with_capacity(n);
        for i in 0..n {
            let theta = -PI + (2.0 * PI * i as f64 / (n - 1) as f64);
            let arg = (theta / theta_s).abs().min(15.0);
            let sech_val = 1.0 / arg.cosh();
            let intensity = sech_val.powi(2);
            profile.push((theta, intensity));
        }

        profile
    }

    /// Solves harmonic generation power efficiency for a given acoustic harmonic order $N$:
    pub fn solve_harmonic_power_ratio(&self, order: usize) -> f64 {
        let metrics = self.solve_microcomb_metrics();
        if order > metrics.highest_harmonic_order || order == 0 {
            return 0.0;
        }
        // Non-linear harmonic power scaling: P_n / P_0 ~ (0.45)^n
        (0.45_f64).powi(order as i32)
    }
}
