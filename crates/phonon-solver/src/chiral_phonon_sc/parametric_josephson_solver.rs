//! Multi-physics solver for light-driven parametric Josephson modulators,
//! dynamic critical current amplification, and ultrafast phase dynamics.

use phonon_models::chiral_phonon_sc::{
    ChiralPhononDriveParams, DynamicJosephsonMetrics, DynamicJosephsonParams,
    TransientPairingParams,
};

/// Multi-physics solver for dynamically modulated Josephson junctions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParametricJosephsonSolver {
    pub josephson_params: DynamicJosephsonParams,
    pub pairing_params: TransientPairingParams,
    pub drive_params: ChiralPhononDriveParams,
}

impl ParametricJosephsonSolver {
    /// Creates a new parametric Josephson solver.
    pub fn new(
        josephson_params: DynamicJosephsonParams,
        pairing_params: TransientPairingParams,
        drive_params: ChiralPhononDriveParams,
    ) -> Self {
        Self {
            josephson_params,
            pairing_params,
            drive_params,
        }
    }

    /// Solves parametric amplification and ultrafast modulation metrics.
    pub fn solve_dynamic_josephson(&self) -> DynamicJosephsonMetrics {
        let pairing_metrics = self.pairing_params.evaluate_pairing(&self.drive_params);
        self.josephson_params
            .evaluate_josephson(&pairing_metrics, self.drive_params.pulse_duration_ps)
    }

    /// Solves parametric amplification gain spectrum as a function of pump-signal detuning $\\delta f$ in $\\text{GHz}$:
    /// $$\mathcal{G}(\\delta f) = \frac{\\mathcal{G}_0}{1 + 4 (\\delta f / \\gamma_J)^2}$$
    pub fn solve_gain_spectrum_db(&self, detuning_ghz: f64, linewidth_ghz: f64) -> f64 {
        let metrics = self.solve_dynamic_josephson();
        let g0_lin = 10.0_f64.powf(metrics.parametric_gain_db / 10.0);
        let gamma = linewidth_ghz.max(0.1);
        let detuning_term = 4.0 * (detuning_ghz / gamma).powi(2);
        let g_lin = g0_lin / (1.0 + detuning_term);
        10.0 * g_lin.max(1.0).log10()
    }

    /// Solves the time-dependent critical current trajectory $I_c(t)$ across the pulse duration in microamperes ($\\mu\\text{A}$).
    pub fn solve_critical_current_trajectory(&self, num_points: usize) -> Vec<(f64, f64)> {
        let n = num_points.max(2);
        let duration_ps = self.drive_params.pulse_duration_ps;
        let pairing_metrics = self.pairing_params.evaluate_pairing(&self.drive_params);
        let ic_base_ua = self.josephson_params.equilibrium_critical_current_a * 1.0e6;
        let enhancement = pairing_metrics.pairing_enhancement_fraction;

        let mut trajectory = Vec::with_capacity(n);
        for i in 0..n {
            let t_ps = (i as f64 / (n - 1) as f64) * duration_ps;
            // Gaussian pulse envelope for non-equilibrium enhancement:
            let center_ps = duration_ps * 0.5;
            let sigma_ps = duration_ps * 0.35;
            let envelope = (-0.5 * ((t_ps - center_ps) / sigma_ps).powi(2)).exp();
            let ic_t = ic_base_ua * (1.0 + enhancement * envelope);
            trajectory.push((t_ps, ic_t));
        }
        trajectory
    }
}
