//! Autonomous multi-physics solver for Terahertz cavity magnon polaritons,
//! Hopfield diagonalization, and ultra-strong coupling transmission spectra.

use phonon_models::afm_spintronics::{
    AfmMaterialParams, MagnonPolaritonMetrics, ThzCavityPolaritonParams,
};

/// Multi-physics solver for Terahertz cavity magnon polaritons.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AfmPolaritonSolver {
    pub afm_params: AfmMaterialParams,
    pub cavity_params: ThzCavityPolaritonParams,
}

impl AfmPolaritonSolver {
    /// Creates a new AFM polariton solver.
    pub fn new(afm_params: AfmMaterialParams, cavity_params: ThzCavityPolaritonParams) -> Self {
        Self {
            afm_params,
            cavity_params,
        }
    }

    /// Evaluates polariton metrics at a specific spin-wave wavevector $k$.
    pub fn solve_polaritons_at_k(&self, k_inv_m: f64) -> MagnonPolaritonMetrics {
        self.cavity_params
            .evaluate_polaritons(&self.afm_params, k_inv_m)
    }

    /// Solves the vacuum Rabi splitting $\Omega_R$ in $\text{GHz}$ at the resonant anti-crossing point.
    pub fn solve_rabi_splitting_ghz(&self) -> f64 {
        let metrics = self.solve_polaritons_at_k(0.0);
        metrics.rabi_splitting_ghz
    }

    /// Solves Hopfield coefficients $(|u_k|^2, |v_k|^2)$ at wavevector $k$:
    /// Photon fraction $|u_k|^2$ and magnon fraction $|v_k|^2$ satisfying $|u_k|^2 + |v_k|^2 = 1$.
    pub fn solve_hopfield_fractions(&self, k_inv_m: f64) -> (f64, f64) {
        let f_cav = self.cavity_params.cavity_frequency_thz;
        let w_mag = self.afm_params.spin_wave_frequency_rad_s(k_inv_m);
        let f_mag = w_mag / (2.0 * std::f64::consts::PI * 1.0e12);
        let delta = f_cav - f_mag;
        let two_g = 2.0 * self.cavity_params.vacuum_coupling_ghz * 1e-3;

        let denom = (delta * delta + two_g * two_g).sqrt().max(1e-12);
        let u2 = 0.5 * (1.0 + delta / denom);
        let v2 = (1.0 - u2).clamp(0.0, 1.0);
        (u2.clamp(0.0, 1.0), v2)
    }

    /// Solves transmission power spectrum $|S_{21}(f)|^2$ across probing frequencies.
    pub fn solve_transmission_spectrum(&self, freq_thz: &[f64]) -> Vec<(f64, f64)> {
        let metrics = self.solve_polaritons_at_k(0.0);
        let f_lower = metrics.lower_polariton_thz;
        let f_upper = metrics.upper_polariton_thz;
        let gamma_eff = (self.cavity_params.cavity_frequency_thz
            / self.cavity_params.cavity_quality_factor)
            .max(1e-4);

        freq_thz
            .iter()
            .map(|&f| {
                // Two Lorentzian transmission peaks at lower and upper polariton frequencies
                let l1 = 0.5 * gamma_eff / ((f - f_lower).powi(2) + (gamma_eff * 0.5).powi(2));
                let l2 = 0.5 * gamma_eff / ((f - f_upper).powi(2) + (gamma_eff * 0.5).powi(2));
                let s21_sq = (0.5 * (l1 + l2) * (gamma_eff * 0.5)).min(1.0);
                (f, s21_sq)
            })
            .collect()
    }
}
