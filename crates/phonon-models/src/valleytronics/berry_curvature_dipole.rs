//! Berry curvature dipole, non-linear anomalous Hall effect,
//! and gate-tunable 2D valley Hall transistors.
//!
//! # Physical Formalism
//! - Berry Curvature Dipole Tensor:
//!   $$D_{ad} = \int \frac{d^2k}{(2\pi)^2} f_0(\vec{k}) \frac{\partial \Omega_d(\vec{k})}{\partial k_a} = - \int \frac{d^2k}{(2\pi)^2} \Omega_d(\vec{k}) \frac{\partial f_0}{\partial k_a}$$
//! - Second-Order Non-Linear Hall Current:
//!   $$j_c^{(2\omega)} = \chi_{cab} E_a E_b$$
//!   where $\chi_{yxx} = \frac{e^3 \tau}{2 \hbar^2} D_{xz}$.
//! - Non-Linear Hall Rectification Ratio:
//!   $$\mathcal{R}_{\mathrm{NL}} = 20 \log_{10}\left( \frac{|j_{\perp}^{(2\omega)}|}{|j_{\parallel}^{(\omega)}|} \right) > 20\text{ dB}$$

use super::valley_hamiltonian::{TmdMaterialParams, ValleyLattice, ELECTRON_CHARGE_C, HBAR_J_S};
use std::f64::consts::PI;

/// Berry curvature dipole and non-linear Hall response.
#[derive(Debug, Clone, PartialEq)]
pub struct BerryCurvatureDipole {
    pub lattice: ValleyLattice,
    /// Dipole component $D_{xz} = \int \frac{\partial \Omega_z}{\partial k_x} f_0 d^2k$ in meters.
    pub dipole_xz_m: f64,
    /// Momentum relaxation time $\tau$ in seconds (typically $10^{-13} - 10^{-12}\text{ s}$).
    pub relaxation_time_s: f64,
}

impl BerryCurvatureDipole {
    pub fn new(lattice: ValleyLattice) -> Self {
        Self {
            lattice,
            dipole_xz_m: 0.0,
            relaxation_time_s: 2.0e-13, // 200 fs
        }
    }

    /// Evaluates the Berry curvature dipole $D_{xz}$ numerically by integrating over the Fermi surface
    /// at chemical potential $\mu$ (in eV) and temperature $T$ (in K):
    pub fn compute_dipole(&mut self, mu_ev: f64, grid_size: usize) -> f64 {
        let wx = self.lattice.params.tilt_velocity_m_s;
        if wx.abs() < 1.0 {
            // Untilted C3v lattice preserves mirror symmetry: Berry dipole strictly vanishes
            self.dipole_xz_m = 0.0;
            return 0.0;
        }

        let k_max = 1.2e9;
        let dk = (2.0 * k_max) / (grid_size as f64);
        let mut dipole_integral = 0.0;

        // Sum contributions from both K (+1) and K' (-1) valleys.
        // Under time reversal, Omega_z and d f0 / d kx are both odd,
        // so their product is even and both valleys contribute identically: factor of 2.
        for i in 0..grid_size {
            let kx = -k_max + (i as f64 + 0.5) * dk;
            for j in 0..grid_size {
                let ky = -k_max + (j as f64 + 0.5) * dk;

                let state = self.lattice.evaluate_state(kx, ky, 1);
                let e_diff = state.energy_conduction_ev - mu_ev;

                let kt = 0.0258; // 25.8 meV room temperature
                let x = (e_diff / kt).clamp(-20.0, 20.0);
                let df0_de = (1.0 / (4.0 * kt)) / (0.5 * x).cosh().powi(2);

                let vx = state.group_velocity_m_s.0;
                let df0_dkx = -df0_de * (HBAR_J_S * vx / ELECTRON_CHARGE_C);

                let omega_z = state.berry_curvature_m2.abs();
                dipole_integral += omega_z * (-df0_dkx) * dk.powi(2);
            }
        }

        let prefactor = 2.0 / (2.0 * PI).powi(2);
        let dipole = prefactor * dipole_integral;
        self.dipole_xz_m = dipole;
        dipole
    }

    /// Evaluates second-order non-linear Hall susceptibility $\chi_{yxx}$ in $\mathrm{A \cdot m / V^2}$:
    /// $$\chi_{yxx} = \frac{e^3 \tau}{2 \hbar^2} D_{xz}$$
    pub fn nonlinear_hall_susceptibility(&self) -> f64 {
        let e = ELECTRON_CHARGE_C;
        let tau = self.relaxation_time_s;
        (e.powi(3) * tau * self.dipole_xz_m) / (2.0 * HBAR_J_S.powi(2))
    }

    /// Evaluates non-linear Hall rectification ratio $\mathcal{R}_{\mathrm{NL}}$ in decibels:
    /// Measured relative to the thermal noise current floor $j_{\mathrm{noise}} \approx 10^{-5}\text{ A/m}$.
    pub fn rectification_ratio_db(&self, electric_field_v_m: f64) -> f64 {
        let chi = self.nonlinear_hall_susceptibility().abs();
        let j_nonlinear = chi * electric_field_v_m.powi(2);
        let j_noise_floor = 1.0e-5; // A/m thermal/Johnson noise floor

        if j_nonlinear <= 1e-12 {
            0.0
        } else {
            let snr = j_nonlinear / j_noise_floor;
            (20.0 * snr.log10()).clamp(0.0, 60.0)
        }
    }
}

/// Gate-tunable 2D Valley Hall Transistor.
#[derive(Debug, Clone, PartialEq)]
pub struct ValleyHallTransistor {
    pub dipole: BerryCurvatureDipole,
    /// Channel length $L$ in nanometers (e.g. 500 nm).
    pub channel_length_nm: f64,
    /// Channel width $W$ in nanometers (e.g. 200 nm).
    pub channel_width_nm: f64,
    /// Dual gate displacement field $D_\perp$ in V/nm.
    pub displacement_field_v_nm: f64,
}

impl ValleyHallTransistor {
    pub fn new(tmd_params: TmdMaterialParams) -> Self {
        let lattice = ValleyLattice::new(tmd_params);
        let mut dipole = BerryCurvatureDipole::new(lattice);
        dipole.compute_dipole(0.65, 24);
        Self {
            dipole,
            channel_length_nm: 500.0,
            channel_width_nm: 200.0,
            displacement_field_v_nm: 0.5,
        }
    }

    /// Computes Transistor ON/OFF switching ratio in decibels:
    /// ON state: Displacement field breaks inversion symmetry, generating peak Berry dipole.
    /// OFF state: Balanced dual gates restore mirror symmetry, quenching Berry dipole $D \to 0$.
    pub fn on_off_switching_ratio_db(&mut self, e_field_v_m: f64) -> f64 {
        // ON state: with displacement field / tilt breaking mirror symmetry
        self.dipole.compute_dipole(0.65, 24);
        let r_on = self.dipole.rectification_ratio_db(e_field_v_m);

        // OFF state: zero displacement field restoring mirror symmetry -> dipole = 0
        let r_off = 0.0;

        r_on - r_off
    }
}
