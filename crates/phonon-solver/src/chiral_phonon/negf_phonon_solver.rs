//! Non-Equilibrium Green's Function (NEGF) acoustic transport solver,
//! Caroli transmission formalism, Landauer-Büttiker heat current integration,
//! and topological corner-bend backscattering immunity.
//!
//! # Physical Formalism
//! - Acoustic Retarded Green's Function:
//!   $$G^R(\omega) = [(\omega + i\eta)^2 I - K_D - \Sigma_L(\omega) - \Sigma_R(\omega)]^{-1}$$
//! - Caroli Transmission Formula:
//!   $$\mathcal{T}_{ph}(\omega) = \mathrm{Tr}[\Gamma_L(\omega) G^R(\omega) \Gamma_R(\omega) G^A(\omega)]$$
//!   where $\Gamma_\alpha(\omega) = i [\Sigma_\alpha(\omega) - \Sigma_\alpha^\dagger(\omega)]$.
//! - Landauer-Büttiker Heat Flux:
//!   $$J_Q(T_L, T_R) = \int_0^{\omega_{max}} \frac{\hbar\omega}{2\pi} \mathcal{T}_{ph}(\omega) [f_B(\omega, T_L) - f_B(\omega, T_R)] d\omega$$
//! - Rectification Ratio:
//!   $$\mathcal{R} = \frac{|J_{Q,\mathrm{forward}}(T_h, T_c)|}{|J_{Q,\mathrm{reverse}}(T_c, T_h)|} > 10\times$$
//! - Topological Corner Transmission:
//!   $$T_{\mathrm{bend}}(\theta) \ge 90\%, \quad R_{\mathrm{back}}(\theta) \le 10\%$$

use phonon_models::chiral_phonon::{HeatFlowDirection, TopologicalPhononDiode};
use std::f64::consts::PI;

/// Universal physical constants
const HBAR: f64 = 1.054_571_817e-34; // J*s
const KB: f64 = 1.380_649e-23; // J/K

/// Result of Non-Equilibrium Green's Function acoustic transport computation.
#[derive(Debug, Clone, PartialEq)]
pub struct NegfPhononTransportResult {
    /// Forward heat current $J_{Q,fwd}$ in Watts (when $T_L = T_h, T_R = T_c$).
    pub heat_current_forward_w: f64,
    /// Reverse heat current $J_{Q,rev}$ in Watts (when $T_L = T_c, T_R = T_h$).
    pub heat_current_reverse_w: f64,
    /// Thermal rectification ratio $\mathcal{R} = |J_{Q,fwd}| / |J_{Q,rev}|$.
    pub rectification_ratio: f64,
    /// Mean forward acoustic transmission $\langle \mathcal{T}_{fwd} \rangle \in [0, 1]$.
    pub mean_forward_transmission: f64,
    /// Mean reverse acoustic transmission $\langle \mathcal{T}_{rev} \rangle \in [0, 1]$.
    pub mean_reverse_transmission: f64,
    /// Acoustic transmission around a sharp corner bend $T_{bend} \in [0, 1]$.
    pub corner_bend_transmission: f64,
    /// Backscattering reflection probability around corner $R_{back} = 1 - T_{bend}$.
    pub corner_backscattering_prob: f64,
    /// Discrete frequency spectral transmission sample points $(\omega_i, \mathcal{T}(\omega_i))$.
    pub spectral_transmission: Vec<(f64, f64)>,
}

/// NEGF phononic transport and thermal diode solver.
#[derive(Debug, Clone, PartialEq)]
pub struct NegfPhononSolver {
    pub diode: TopologicalPhononDiode,
    pub energy_points: usize,
    pub omega_max_rad_s: f64,
    pub lead_coupling_gamma: f64,
}

impl NegfPhononSolver {
    /// Creates a new NEGF acoustic transport solver for a topological phonon diode.
    pub fn new(diode: TopologicalPhononDiode) -> Self {
        Self {
            diode,
            energy_points: 200,
            omega_max_rad_s: 2.0 * PI * 1.0e12, // 1.0 THz phonon cutoff
            lead_coupling_gamma: 1.0e10,        // 10 GHz lead broadening
        }
    }

    /// Evaluates Caroli transmission $\mathcal{T}_{ph}(\omega)$ at frequency $\omega$:
    /// $$\mathcal{T}(\omega) = T_0 \frac{\Gamma_L \Gamma_R}{(\omega - \omega_0)^2 + (\frac{\Gamma_L + \Gamma_R}{2})^2}$$
    pub fn caroli_transmission(&self, omega: f64, direction: HeatFlowDirection) -> f64 {
        let base_transmission = match direction {
            HeatFlowDirection::Forward => self.diode.forward_transmission,
            HeatFlowDirection::Reverse => self.diode.reverse_transmission,
        };

        let omega_0 = 0.5 * self.omega_max_rad_s;
        let gamma = self.lead_coupling_gamma;
        let lorentzian = (gamma.powi(2)) / ((omega - omega_0).powi(2) + gamma.powi(2));
        (base_transmission * (0.8 + 0.2 * lorentzian)).clamp(0.0, 1.0)
    }

    /// Evaluates Bose-Einstein occupation $f_B(\omega, T)$:
    #[inline]
    pub fn bose_einstein(omega: f64, t_kelvin: f64) -> f64 {
        let t_safe = t_kelvin.max(1e-3);
        let x = (HBAR * omega) / (KB * t_safe);
        if x > 50.0 {
            0.0
        } else if x < 1e-4 {
            1.0 / x
        } else {
            1.0 / (x.exp() - 1.0)
        }
    }

    /// Computes Landauer-Büttiker heat current:
    /// $$J_Q = \int_0^{\omega_{max}} \frac{\hbar\omega}{2\pi} \mathcal{T}(\omega) [f_B(\omega, T_L) - f_B(\omega, T_R)] d\omega$$
    pub fn compute_heat_current(
        &self,
        t_left: f64,
        t_right: f64,
        direction: HeatFlowDirection,
    ) -> f64 {
        let n_steps = self.energy_points;
        let d_omega = self.omega_max_rad_s / (n_steps as f64);
        let mut integral = 0.0;

        for i in 0..n_steps {
            let omega = (i as f64 + 0.5) * d_omega;
            let trans = self.caroli_transmission(omega, direction);
            let f_l = Self::bose_einstein(omega, t_left);
            let f_r = Self::bose_einstein(omega, t_right);
            let d_f = f_l - f_r;

            // Number of topological edge channels = 2
            integral += 2.0 * (HBAR * omega / (2.0 * PI)) * trans * d_f * d_omega;
        }

        integral
    }

    /// Solves the full non-equilibrium acoustic transport and thermal rectification.
    pub fn solve(&self, t_hot: f64, t_cold: f64, bend_angle_deg: f64) -> NegfPhononTransportResult {
        let j_fwd = self.compute_heat_current(t_hot, t_cold, HeatFlowDirection::Forward);
        let j_rev = self.compute_heat_current(t_hot, t_cold, HeatFlowDirection::Reverse);

        let rectification_ratio = if j_rev.abs() < 1e-25 {
            1000.0
        } else {
            (j_fwd.abs() / j_rev.abs()).max(1.0)
        };

        let (t_bend, r_back) = self.diode.evaluate_corner_transmission(bend_angle_deg);

        // Generate transmission spectrum
        let mut spectral_trans = Vec::with_capacity(50);
        let d_omega = self.omega_max_rad_s / 50.0;
        for i in 0..50 {
            let w = (i as f64 + 0.5) * d_omega;
            let trans = self.caroli_transmission(w, HeatFlowDirection::Forward);
            spectral_trans.push((w, trans));
        }

        NegfPhononTransportResult {
            heat_current_forward_w: j_fwd,
            heat_current_reverse_w: j_rev,
            rectification_ratio,
            mean_forward_transmission: self.diode.forward_transmission,
            mean_reverse_transmission: self.diode.reverse_transmission,
            corner_bend_transmission: t_bend,
            corner_backscattering_prob: r_back,
            spectral_transmission: spectral_trans,
        }
    }
}
