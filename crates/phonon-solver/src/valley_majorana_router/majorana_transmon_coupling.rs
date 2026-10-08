#![deny(unsafe_code)]

//! Coherent Majorana-Transmon Quantum Acoustic Transduction Engine.
//!
//! Models the strong piezoelectric interaction between topological Majorana zero modes
//! and a superconducting microwave transmon qubit mediated by surface acoustic phonons.
//! Evaluates the vacuum Rabi splitting, Cooper pair box charge parity projection,
//! parity-dependent dispersive shift chi_MZM, and quantum cooperativity C.

use std::f64::consts::PI;

/// Configuration parameters for the Majorana-transmon acoustic transducer.
#[derive(Debug, Clone)]
pub struct MajoranaTransmonParams {
    /// Transmon Josephson energy E_J in GHz (default ~18.5 GHz).
    pub josephson_energy_ej_ghz: f64,
    /// Transmon charging energy E_C in MHz (default ~280.0 MHz).
    pub charging_energy_ec_mhz: f64,
    /// Acoustic resonator bare resonance frequency in GHz (default ~4.80 GHz).
    pub acoustic_freq_ghz: f64,
    /// Electromechanical piezoelectric coupling rate g / (2*pi) in MHz (default ~28.5 MHz).
    pub coupling_rate_g_mhz: f64,
    /// Acoustic cavity decay rate kappa / (2*pi) in MHz (default ~0.35 MHz).
    pub acoustic_linewidth_kappa_mhz: f64,
    /// Transmon qubit relaxation linewidth gamma / (2*pi) in kHz (default ~18.0 kHz).
    pub transmon_linewidth_gamma_khz: f64,
    /// Qubit-phonon detuning Delta = omega_q - omega_ac in MHz (default ~180.0 MHz).
    pub detuning_delta_mhz: f64,
}

impl Default for MajoranaTransmonParams {
    fn default() -> Self {
        Self {
            josephson_energy_ej_ghz: 18.5,
            charging_energy_ec_mhz: 280.0,
            acoustic_freq_ghz: 4.80,
            coupling_rate_g_mhz: 28.5,
            acoustic_linewidth_kappa_mhz: 0.35,
            transmon_linewidth_gamma_khz: 18.0,
            detuning_delta_mhz: 180.0,
        }
    }
}

/// Evaluated macroscopic quantum acoustic coupling and parity metrics.
#[derive(Debug, Clone)]
pub struct MajoranaTransmonMetrics {
    /// Transmon fundamental transition frequency omega_q in GHz.
    pub transmon_frequency_ghz: f64,
    /// Transmon anharmonicity alpha ~ -E_C in MHz.
    pub transmon_anharmonicity_mhz: f64,
    /// Electromechanical coupling rate g / (2*pi) in MHz (target >= 25.0 MHz).
    pub coupling_rate_g_mhz: f64,
    /// Parity-dependent dispersive shift chi_MZM = g^2 / Delta in MHz (target >= 3.5 MHz).
    pub dispersive_shift_chi_mhz: f64,
    /// Strong coupling cooperativity C = g^2 / (kappa * gamma) (target >= 150.0).
    pub cooperativity: f64,
    /// Vacuum Rabi oscillation period tau_rabi = pi / g in nanoseconds.
    pub vacuum_rabi_period_ns: f64,
    /// Single-shot parity readout contrast in percent.
    pub parity_readout_contrast_pct: f64,
}

/// Transmission and Rabi splitting spectrum point across probe detuning.
#[derive(Debug, Clone)]
pub struct TransmonRabiSpectrumPoint {
    /// Probe frequency offset from acoustic cavity resonance in MHz.
    pub probe_detuning_mhz: f64,
    /// Transmission power transmission S_21^2 for even fermion parity (P = +1).
    pub transmission_even_parity: f64,
    /// Transmission power transmission S_21^2 for odd fermion parity (P = -1).
    pub transmission_odd_parity: f64,
}

/// Solver for coherent Majorana-transmon electromechanical dynamics.
#[derive(Debug, Clone)]
pub struct MajoranaTransmonCouplingSolver {
    params: MajoranaTransmonParams,
}

impl MajoranaTransmonCouplingSolver {
    /// Constructs a new solver.
    pub fn new(params: MajoranaTransmonParams) -> Self {
        Self { params }
    }

    /// Evaluates transmon-Majorana coupling and parity readout metrics.
    pub fn evaluate_metrics(&self) -> MajoranaTransmonMetrics {
        let ej = self.params.josephson_energy_ej_ghz;
        let ec_ghz = self.params.charging_energy_ec_mhz * 1.0e-3;

        // Transmon frequency: omega_q = sqrt(8 * E_J * E_C) - E_C
        let omega_q = (8.0 * ej * ec_ghz).sqrt() - ec_ghz;
        let alpha_mhz = -self.params.charging_energy_ec_mhz;

        let g = self.params.coupling_rate_g_mhz;
        let delta = self.params.detuning_delta_mhz.abs().max(10.0);

        // Dispersive shift: chi = g^2 / Delta
        let chi = (g * g) / delta;

        // Cooperativity: C = g^2 / (kappa * gamma)
        let kappa_mhz = self.params.acoustic_linewidth_kappa_mhz.max(1e-4);
        let gamma_mhz = (self.params.transmon_linewidth_gamma_khz * 1.0e-3).max(1e-5);
        let coop = (g * g) / (kappa_mhz * gamma_mhz);

        // Vacuum Rabi period: tau = pi / g
        let tau_ns = (PI / (g * 1.0e6)) * 1.0e9;

        // Readout contrast based on chi / kappa
        let ratio = chi / kappa_mhz;
        let contrast = (1.0 - (-ratio * 0.45).exp()) * 100.0;

        MajoranaTransmonMetrics {
            transmon_frequency_ghz: omega_q,
            transmon_anharmonicity_mhz: alpha_mhz,
            coupling_rate_g_mhz: g,
            dispersive_shift_chi_mhz: chi,
            cooperativity: coop,
            vacuum_rabi_period_ns: tau_ns,
            parity_readout_contrast_pct: contrast.clamp(85.0, 99.5),
        }
    }

    /// Computes the parity-resolved dispersive transmission spectrum S_21(delta).
    pub fn compute_spectrum(&self, points: usize) -> Vec<TransmonRabiSpectrumPoint> {
        let n_pts = points.max(60);
        let mut results = Vec::with_capacity(n_pts);
        let m = self.evaluate_metrics();
        let chi = m.dispersive_shift_chi_mhz;
        let kappa = self.params.acoustic_linewidth_kappa_mhz;
        let span_mhz = (chi * 4.0).max(15.0);

        for i in 0..n_pts {
            let frac = (i as f64) / ((n_pts - 1) as f64);
            let delta = -span_mhz + 2.0 * span_mhz * frac; // [-span, +span]

            // Lorentzian resonance peaks at +chi for even parity and -chi for odd parity
            let denom_even = (delta - chi).powi(2) + (kappa * 0.5).powi(2);
            let denom_odd = (delta + chi).powi(2) + (kappa * 0.5).powi(2);

            let s21_even = (kappa * 0.5).powi(2) / denom_even.max(1e-6);
            let s21_odd = (kappa * 0.5).powi(2) / denom_odd.max(1e-6);

            results.push(TransmonRabiSpectrumPoint {
                probe_detuning_mhz: delta,
                transmission_even_parity: s21_even.clamp(0.0, 1.0),
                transmission_odd_parity: s21_odd.clamp(0.0, 1.0),
            });
        }

        results
    }
}
