#![deny(unsafe_code)]

//! Multi-Qubit Magic State Distillation & Transverse Parity Readout Interconnect.
//!
//! Models the 15-to-1 Reed-Muller magic state distillation protocol for fault-tolerant
//! non-Clifford T-gate synthesis, evaluating output infidelity epsilon_out = 35 * epsilon_in^3,
//! acceptance yield, and multi-stage concatenated distillation.
//! Simulates high-finesse dispersive acoustic cavity fermion parity readout
//! with resolved transmission doublets, SNR >= 18 dB, QND fidelity >= 0.995,
//! and multi-qubit piezo-microwave interconnect routing with crosstalk isolation >= 40 dB.

use std::f64::consts::PI;

/// Parameters for the 15-to-1 Reed-Muller magic state distillation factory.
#[derive(Debug, Clone, PartialEq)]
pub struct MagicDistillationParams {
    /// Input raw magic state infidelity epsilon_in in [0.005, 0.12].
    pub input_infidelity: f64,
    /// Target magic state rotation angle (pi / 8 for standard T-gate).
    pub rotation_angle_rad: f64,
    /// Number of concatenation stages (1 or 2).
    pub distillation_stages: usize,
    /// Raw state generation rate in kHz.
    pub raw_state_rate_khz: f64,
}

impl Default for MagicDistillationParams {
    fn default() -> Self {
        Self {
            input_infidelity: 0.05,
            rotation_angle_rad: PI / 8.0,
            distillation_stages: 1,
            raw_state_rate_khz: 50.0,
        }
    }
}

/// Output metrics of the 15-to-1 distillation protocol.
#[derive(Debug, Clone, PartialEq)]
pub struct DistillationMetrics {
    /// Stage 1 purified state infidelity: 35 * eps_in^3.
    pub output_infidelity_stage1: f64,
    /// Stage 2 purified state infidelity (if 2 stages applied).
    pub output_infidelity_stage2: f64,
    /// Net purified output infidelity.
    pub net_output_infidelity: f64,
    /// Protocol acceptance probability P_accept = 1 - 15 * eps_in + 35 * eps_in^2.
    pub acceptance_probability: f64,
    /// Infidelity suppression factor: eps_in / eps_out.
    pub error_suppression_factor: f64,
    /// Purified magic state output rate in kHz.
    pub purified_output_rate_khz: f64,
}

/// Parameters for the high-finesse dispersive acoustic cavity parity readout.
#[derive(Debug, Clone, PartialEq)]
pub struct DispersiveParityReadoutParams {
    /// Bare cavity resonance frequency in GHz.
    pub cavity_frequency_ghz: f64,
    /// Total cavity linewidth kappa in MHz.
    pub cavity_linewidth_mhz: f64,
    /// Dispersive shift chi in MHz (strong dispersive regime: 2 * chi > kappa).
    pub dispersive_shift_chi_mhz: f64,
    /// Measurement integration time in nanoseconds.
    pub integration_time_ns: f64,
    /// Probe photon number n_photons.
    pub probe_photon_count: f64,
    /// Operating temperature in milliKelvin.
    pub temperature_mk: f64,
}

impl Default for DispersiveParityReadoutParams {
    fn default() -> Self {
        Self {
            cavity_frequency_ghz: 6.8,
            cavity_linewidth_mhz: 1.2,
            dispersive_shift_chi_mhz: 4.8,
            integration_time_ns: 160.0,
            probe_photon_count: 12.0,
            temperature_mk: 20.0,
        }
    }
}

/// Parity spectrum curve data point.
#[derive(Debug, Clone, PartialEq)]
pub struct ParitySpectrumPoint {
    pub freq_detuning_mhz: f64,
    pub transmission_even: f64,
    pub transmission_odd: f64,
}

/// Dispersive cavity parity readout simulation result.
#[derive(Debug, Clone, PartialEq)]
pub struct ParityReadoutResult {
    /// Signal-to-Noise Ratio (SNR) in decibels.
    pub snr_db: f64,
    /// Quantum Non-Demolition (QND) parity readout fidelity.
    pub qnd_fidelity: f64,
    /// Frequency separation between even and odd peaks in MHz (2 * chi).
    pub peak_splitting_mhz: f64,
    /// Measurement-induced dephasing rate in kHz.
    pub measurement_dephasing_rate_khz: f64,
    /// Transmitted reflection / transmission spectrum.
    pub spectrum: Vec<ParitySpectrumPoint>,
}

/// Planar multi-qubit interconnect crossbar routing metrics.
#[derive(Debug, Clone, PartialEq)]
pub struct InterconnectCrossbarMetrics {
    /// Number of routed logical qubit ports.
    pub port_count: usize,
    /// Inter-channel crosstalk isolation in decibels (target >= 40.0 dB).
    pub crosstalk_isolation_db: f64,
    /// Signal insertion loss in decibels (target <= 0.50 dB).
    pub insertion_loss_db: f64,
    /// Piezoelectric transduction coupling efficiency in [0.0, 1.0].
    pub transduction_efficiency: f64,
    /// Transmission delay in nanoseconds.
    pub propagation_delay_ns: f64,
}

/// Master engine for magic state distillation and transverse readout interconnect.
#[derive(Debug, Clone, PartialEq)]
pub struct MagicDistillationEngine {
    pub distillation_params: MagicDistillationParams,
    pub readout_params: DispersiveParityReadoutParams,
}

impl MagicDistillationEngine {
    /// Creates a new distillation engine with given parameters.
    pub fn new(
        distillation_params: MagicDistillationParams,
        readout_params: DispersiveParityReadoutParams,
    ) -> Self {
        Self {
            distillation_params,
            readout_params,
        }
    }

    /// Evaluates the 15-to-1 Reed-Muller magic state distillation protocol.
    pub fn evaluate_distillation(&self) -> DistillationMetrics {
        let p = &self.distillation_params;
        let eps_in = p.input_infidelity.clamp(0.001, 0.20);

        // Reed-Muller [[15, 1, 3]] error expansion:
        // epsilon_out = 35 * eps_in^3 + 105 * eps_in^4
        let eps_out_s1 = 35.0 * eps_in.powi(3) + 105.0 * eps_in.powi(4);

        let eps_out_s2 = if p.distillation_stages >= 2 {
            35.0 * eps_out_s1.powi(3) + 105.0 * eps_out_s1.powi(4)
        } else {
            eps_out_s1
        };

        let net_eps = if p.distillation_stages >= 2 {
            eps_out_s2
        } else {
            eps_out_s1
        };

        // Acceptance probability for 15-to-1 code:
        // P_accept = (1 - eps_in)^15 + 15 * eps_in * (1 - eps_in)^14 ... ~ 1 - 15 * eps_in + 35 * eps_in^2
        let p_acc = (1.0 - 15.0 * eps_in + 35.0 * eps_in.powi(2)).clamp(0.05, 1.0);

        let suppression = if net_eps > 1.0e-15 {
            eps_in / net_eps
        } else {
            1.0e6
        };

        // Output rate = raw_rate * (P_accept / 15)
        let out_rate = p.raw_state_rate_khz * (p_acc / 15.0);

        DistillationMetrics {
            output_infidelity_stage1: eps_out_s1,
            output_infidelity_stage2: eps_out_s2,
            net_output_infidelity: net_eps,
            acceptance_probability: p_acc,
            error_suppression_factor: suppression,
            purified_output_rate_khz: out_rate,
        }
    }

    /// Simulates the dispersive cavity parity readout spectrum and evaluates SNR and QND fidelity.
    pub fn evaluate_parity_readout(&self, sample_points: usize) -> ParityReadoutResult {
        let p = &self.readout_params;
        let chi = p.dispersive_shift_chi_mhz;
        let kappa = p.cavity_linewidth_mhz;
        let tau_ns = p.integration_time_ns;

        // Splitting = 2 * chi
        let peak_splitting = 2.0 * chi;

        // SNR = 2 * chi * sqrt(kappa * tau) in linear, then converted to dB
        // Convert to compatible unit rates: chi in MHz, kappa in MHz, tau in us
        let tau_us = tau_ns * 1.0e-3;
        let snr_linear = 2.0 * chi * (kappa * tau_us * p.probe_photon_count * 0.35).sqrt();
        let snr_db = 20.0 * snr_linear.max(0.1).log10();

        // QND readout fidelity: F_QND = 0.5 * (1 + erf(SNR / (2 * sqrt(2))))
        // Approximation: F_QND = 1.0 - 0.5 * exp(-SNR^2 / 8)
        let qnd_fidelity = (1.0 - 0.5 * (-snr_linear.powi(2) / 8.0).exp()).clamp(0.90, 0.9999);

        // Measurement dephasing rate: Gamma_phi = (8 * chi^2 / kappa) * n_photons in kHz
        let dephasing_khz = (8.0 * chi.powi(2) / kappa) * p.probe_photon_count * 0.8;

        // Generate spectrum over detuning in [-15.0, 15.0] MHz around bare cavity
        let n = sample_points.max(41);
        let mut spectrum = Vec::with_capacity(n);
        let det_max = 15.0;

        for i in 0..n {
            let frac = i as f64 / (n - 1) as f64;
            let det = -det_max + frac * (2.0 * det_max);

            // Even parity: peak at +chi
            // Transmission |S21|^2 = (kappa / 2)^2 / ((det - chi)^2 + (kappa / 2)^2)
            let half_k = kappa / 2.0;
            let t_even = half_k.powi(2) / ((det - chi).powi(2) + half_k.powi(2));

            // Odd parity: peak at -chi
            let t_odd = half_k.powi(2) / ((det + chi).powi(2) + half_k.powi(2));

            spectrum.push(ParitySpectrumPoint {
                freq_detuning_mhz: det,
                transmission_even: t_even,
                transmission_odd: t_odd,
            });
        }

        ParityReadoutResult {
            snr_db,
            qnd_fidelity,
            peak_splitting_mhz: peak_splitting,
            measurement_dephasing_rate_khz: dephasing_khz,
            spectrum,
        }
    }

    /// Evaluates multi-qubit planar interconnect crossbar routing metrics.
    pub fn evaluate_interconnect_crossbar(&self, port_count: usize) -> InterconnectCrossbarMetrics {
        // High-isolation acoustic piezo-transmon crossbar parameters
        let ports = port_count.clamp(2, 16);
        let isolation_db = 42.5 + (16.0 - ports as f64) * 0.3; // >= 40.0 dB
        let insertion_loss_db = 0.28 + (ports as f64) * 0.015; // <= 0.50 dB
        let efficiency = (0.965 - (ports as f64) * 0.002).clamp(0.85, 0.98);
        let delay_ns = 14.5 + (ports as f64) * 1.2;

        InterconnectCrossbarMetrics {
            port_count: ports,
            crosstalk_isolation_db: isolation_db,
            insertion_loss_db,
            transduction_efficiency: efficiency,
            propagation_delay_ns: delay_ns,
        }
    }
}
