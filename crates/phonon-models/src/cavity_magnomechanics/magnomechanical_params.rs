//! Tripartite microwave cavity magnomechanical system parameters,
//! dispersive magnetostriction, and ground-state dynamical backaction cooling.

use phonon_core::constants::{BOLTZMANN_CONSTANT, H_BAR};
use std::f64::consts::PI;

/// Parameters for a tripartite microwave cavity magnomechanical system.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CavityMagnomechanicalParams {
    /// Microwave cavity resonance frequency in $\\text{GHz}$ (nominal $8.0 - 10.0\\text{ GHz}$).
    pub cavity_frequency_ghz: f64,
    /// Kittel magnon resonance frequency in $\\text{GHz}$ (nominal $8.0 - 10.0\\text{ GHz}$).
    pub magnon_frequency_ghz: f64,
    /// Mechanical phonon resonance frequency in $\\text{MHz}$ (nominal $10.0 - 40.0\\text{ MHz}$).
    pub phonon_frequency_mhz: f64,
    /// Microwave cavity decay rate $\\kappa_a / 2\\pi$ in $\\text{MHz}$ (nominal $1.0 - 5.0\\text{ MHz}$).
    pub photon_damping_mhz: f64,
    /// Kittel magnon damping rate $\\gamma_m / 2\\pi$ in $\\text{MHz}$ (nominal $1.0 - 3.0\\text{ MHz}$).
    pub magnon_damping_mhz: f64,
    /// Mechanical phonon damping rate $\\gamma_b / 2\\pi$ in $\\text{Hz}$ (nominal $50.0 - 200.0\\text{ Hz}$).
    pub phonon_damping_hz: f64,
    /// Photon-magnon magnetic dipole coupling $g_{ma} / 2\\pi$ in $\\text{MHz}$ (nominal $15.0 - 35.0\\text{ MHz}$).
    pub photon_magnon_coupling_mhz: f64,
    /// Bare single-spin magnetostrictive coupling $g_{mb} / 2\\pi$ in $\\text{Hz}$ (nominal $0.1 - 1.0\\text{ Hz}$).
    pub single_spin_magnetostriction_hz: f64,
    /// Number of driven coherent magnons $N_m$ (nominal $10^9 - 10^{12}$).
    pub coherent_magnon_number: f64,
    /// Cryogenic dilution refrigerator bath temperature in millikelvin ($\\text{mK}$) (nominal $10.0 - 50.0\\text{ mK}$).
    pub bath_temperature_mk: f64,
}

impl Default for CavityMagnomechanicalParams {
    fn default() -> Self {
        Self {
            cavity_frequency_ghz: 9.0,
            magnon_frequency_ghz: 9.0,
            phonon_frequency_mhz: 20.0,
            photon_damping_mhz: 2.5,
            magnon_damping_mhz: 1.5,
            phonon_damping_hz: 100.0,
            photon_magnon_coupling_mhz: 25.0,
            single_spin_magnetostriction_hz: 0.5,
            coherent_magnon_number: 1.0e11,
            bath_temperature_mk: 20.0,
        }
    }
}

/// Evaluated metrics for the tripartite cavity magnomechanical interaction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MagnomechanicalMetrics {
    /// Linearized effective magnon-phonon coupling $G_{mb} = g_{mb} \\sqrt{N_m}$ in $\\text{MHz}$.
    pub effective_coupling_g_mb_mhz: f64,
    /// Magnon-phonon cooperativity $C_{mb} = \\frac{4 G_{mb}^2}{\\gamma_m \\gamma_b (2 \\bar{n}_b + 1)}$.
    pub magnon_phonon_cooperativity: f64,
    /// Thermal equilibrium phonon occupation number $\\bar{n}_b = \\frac{k_B T}{\\hbar \\omega_b}$.
    pub thermal_phonon_number: f64,
    /// Dynamical backaction-cooled effective phonon occupation $n_{\\mathrm{eff}} \\approx \\frac{\\bar{n}_b}{1 + C_{mb}}$.
    pub effective_phonon_occupation: f64,
    /// Resolved sideband parameter $\\frac{\\omega_b}{\\gamma_m}$.
    pub sideband_parameter: f64,
}

impl CavityMagnomechanicalParams {
    /// Creates a new cavity magnomechanical parameter set.
    pub fn new(
        phonon_frequency_mhz: f64,
        coherent_magnon_number: f64,
        bath_temperature_mk: f64,
    ) -> Self {
        Self {
            phonon_frequency_mhz: phonon_frequency_mhz.max(1.0),
            coherent_magnon_number: coherent_magnon_number.max(1e6),
            bath_temperature_mk: bath_temperature_mk.clamp(1.0, 500.0),
            ..Default::default()
        }
    }

    /// Evaluates basic magnomechanical coupling and cooling metrics.
    pub fn evaluate_metrics(&self) -> MagnomechanicalMetrics {
        // Effective linearized coupling G_mb = g_mb * sqrt(N_m) in MHz
        let g_lin_hz = self.single_spin_magnetostriction_hz * self.coherent_magnon_number.sqrt();
        let g_lin_mhz = g_lin_hz * 1.0e-6;

        // Thermal phonon number: n_th = k_B * T / (hbar * omega_b)
        let omega_b_rad_s = self.phonon_frequency_mhz * 1.0e6 * 2.0 * PI;
        let t_k = self.bath_temperature_mk * 1.0e-3;
        let n_th = (BOLTZMANN_CONSTANT * t_k) / (H_BAR * omega_b_rad_s);

        // Cooperativity C_mb = 4 * G_mb^2 / (gamma_m * gamma_b * (2*n_th + 1))
        let gamma_m_hz = self.magnon_damping_mhz * 1.0e6;
        let gamma_b_hz = self.phonon_damping_hz;
        let cooperativity =
            (4.0 * g_lin_hz * g_lin_hz) / (gamma_m_hz * gamma_b_hz * (2.0 * n_th + 1.0).max(1.0));

        // Effective phonon occupation under dynamical backaction cooling:
        // n_eff = n_th / (1 + C_mb) + n_quant
        let n_eff = n_th / (1.0 + cooperativity);

        // Sideband parameter omega_b / gamma_m
        let sideband = (self.phonon_frequency_mhz * 1.0e6) / gamma_m_hz;

        MagnomechanicalMetrics {
            effective_coupling_g_mb_mhz: g_lin_mhz,
            magnon_phonon_cooperativity: cooperativity,
            thermal_phonon_number: n_th,
            effective_phonon_occupation: n_eff,
            sideband_parameter: sideband,
        }
    }
}
