//! Terahertz cavity magnon polaritons, ultra-strong coupling,
//! and vacuum Rabi splitting in split-ring microcavities.

use super::afm_params::AfmMaterialParams;
use std::f64::consts::PI;

/// Parameters for a Terahertz microcavity coupled to an antiferromagnet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThzCavityPolaritonParams {
    /// Bare cavity photon resonance frequency in Terahertz ($\text{THz}$) (nominal $0.5 - 2.0\text{ THz}$).
    pub cavity_frequency_thz: f64,
    /// Cavity photon loaded quality factor $Q_c$ (nominal $100 - 1000$).
    pub cavity_quality_factor: f64,
    /// Cavity effective modal volume $V_c$ in $\text{m}^3$ (nominal $1.0 \times 10^{-14} - 1.0 \times 10^{-12}\text{ m}^3$).
    pub modal_volume_m3: f64,
    /// Collective vacuum Rabi coupling frequency $g_{\mathrm{amp}} / (2\pi)$ in $\text{GHz}$ (nominal $100 - 300\text{ GHz}$).
    pub vacuum_coupling_ghz: f64,
}

impl Default for ThzCavityPolaritonParams {
    fn default() -> Self {
        Self {
            cavity_frequency_thz: 1.0,
            cavity_quality_factor: 250.0,
            modal_volume_m3: 1.0e-13,
            vacuum_coupling_ghz: 150.0,
        }
    }
}

/// Evaluated polariton state metrics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MagnonPolaritonMetrics {
    /// Lower polariton branch frequency in $\text{THz}$.
    pub lower_polariton_thz: f64,
    /// Upper polariton branch frequency in $\text{THz}$.
    pub upper_polariton_thz: f64,
    /// Vacuum Rabi splitting $\Omega_R = \omega_+ - \omega_-$ at zero detuning in $\text{GHz}$ ($> 100\text{ GHz}$).
    pub rabi_splitting_ghz: f64,
    /// Normalized coupling strength $\eta_c = g / \omega_c$ ($\ge 0.10$ for ultra-strong coupling).
    pub normalized_coupling_eta: f64,
    /// Polariton cooperativity $\mathcal{C}_{\mathrm{amp}} = \frac{4 g^2}{\kappa_c \gamma_m} \gg 100$.
    pub cooperativity: f64,
}

impl ThzCavityPolaritonParams {
    /// Creates new Terahertz cavity polariton parameters.
    pub fn new(
        cavity_frequency_thz: f64,
        cavity_quality_factor: f64,
        vacuum_coupling_ghz: f64,
    ) -> Self {
        Self {
            cavity_frequency_thz: cavity_frequency_thz.max(0.1),
            cavity_quality_factor: cavity_quality_factor.max(10.0),
            modal_volume_m3: 1.0e-13,
            vacuum_coupling_ghz: vacuum_coupling_ghz.max(1.0),
        }
    }

    /// Evaluates upper and lower polariton branch frequencies and Rabi splitting
    /// coupled to an AFM material with given resonance.
    pub fn evaluate_polaritons(
        &self,
        afm_params: &AfmMaterialParams,
        k_inv_m: f64,
    ) -> MagnonPolaritonMetrics {
        let f_cav = self.cavity_frequency_thz;
        let w_mag = afm_params.spin_wave_frequency_rad_s(k_inv_m);
        let f_mag = w_mag / (2.0 * PI * 1.0e12);

        let g_thz = self.vacuum_coupling_ghz * 1e-3;

        // Polariton eigenvalues:
        // omega_pm = (f_cav + f_mag)/2 +- sqrt( ((f_cav - f_mag)/2)^2 + g^2 )
        let f_avg = (f_cav + f_mag) * 0.5;
        let detuning_half = (f_cav - f_mag) * 0.5;
        let rad = (detuning_half * detuning_half + g_thz * g_thz).sqrt();

        let upper_thz = f_avg + rad;
        let lower_thz = f_avg - rad;

        let rabi_ghz = 2.0 * g_thz * 1.0e3;
        let eta = (self.vacuum_coupling_ghz * 1e9) / (self.cavity_frequency_thz * 1e12);

        // Cavity decay rate kappa_c = omega_c / Q_c
        let kappa_c = (self.cavity_frequency_thz * 1e12) / self.cavity_quality_factor;
        // Magnon decay rate gamma_m = alpha * omega_afmr
        let gamma_m =
            afm_params.gilbert_damping * (afm_params.afmr_angular_frequency_rad_s() / (2.0 * PI));
        let g_hz = self.vacuum_coupling_ghz * 1e9;
        let cooperativity = (4.0 * g_hz * g_hz) / (kappa_c * gamma_m).max(1.0);

        MagnonPolaritonMetrics {
            lower_polariton_thz: lower_thz,
            upper_polariton_thz: upper_thz,
            rabi_splitting_ghz: rabi_ghz,
            normalized_coupling_eta: eta,
            cooperativity,
        }
    }
}
