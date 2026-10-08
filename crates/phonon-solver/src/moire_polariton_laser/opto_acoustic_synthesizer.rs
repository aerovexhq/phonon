#![deny(unsafe_code)]

//! Moiré Exciton-Polariton Opto-Acoustic Frequency Synthesizer & Microwave Transducer Engine.
//!
//! Models acoustic-phonon mediated parametric phase modulation of moiré exciton-polaritons,
//! synthesizing optical frequency combs with microwave-rate spacing (2.0 to 10.0 GHz),
//! high-fidelity microwave-to-optical quantum transduction, and ultra-low phase noise.

use std::f64::consts::PI;

/// Parameters for the opto-acoustic frequency synthesizer.
#[derive(Debug, Clone)]
pub struct OptoAcousticSynthesizerParams {
    /// Optical carrier resonance frequency in THz (e.g. 400.0 THz / ~750 nm).
    pub carrier_frequency_thz: f64,
    /// Microwave acoustic phonon drive frequency Omega_m in GHz.
    pub microwave_acoustic_freq_ghz: f64,
    /// Applied microwave acoustic phonon power in mW.
    pub acoustic_drive_power_mw: f64,
    /// Single-phonon opto-acoustic coupling rate g_om in MHz.
    pub optoacoustic_coupling_mhz: f64,
    /// Optical microcavity linewidth kappa in GHz.
    pub optical_cavity_linewidth_ghz: f64,
    /// Acoustic phonon resonator damping rate gamma_m in MHz.
    pub acoustic_damping_mhz: f64,
    /// Piezoelectric IDT acoustic transduction efficiency in [0.0, 1.0].
    pub piezo_transduction_efficiency: f64,
}

impl Default for OptoAcousticSynthesizerParams {
    fn default() -> Self {
        Self {
            carrier_frequency_thz: 400.0,
            microwave_acoustic_freq_ghz: 3.6,
            acoustic_drive_power_mw: 2.5,
            optoacoustic_coupling_mhz: 18.0,
            optical_cavity_linewidth_ghz: 12.0,
            acoustic_damping_mhz: 0.80,
            piezo_transduction_efficiency: 0.88,
        }
    }
}

/// Evaluated metrics for the opto-acoustic frequency synthesizer.
#[derive(Debug, Clone)]
pub struct OptoAcousticMetrics {
    /// Optical phase modulation index beta in radians.
    pub modulation_index_beta: f64,
    /// Carrier suppression ratio in dB relative to unmodulated carrier.
    pub carrier_suppression_ratio_db: f64,
    /// First-order sideband power fraction P(+/-1) / P_total.
    pub first_sideband_power_fraction: f64,
    /// Second-order sideband power fraction P(+/-2) / P_total.
    pub second_sideband_power_fraction: f64,
    /// Microwave-to-optical quantum transduction efficiency in [0.0, 1.0].
    pub microwave_to_optical_efficiency: f64,
    /// Synthesized optical carrier phase noise at 10 kHz offset in dBc/Hz.
    pub phase_noise_at_10khz_dbc_hz: f64,
    /// Quantum added noise quanta n_add.
    pub quantum_added_noise_quanta: f64,
    /// Optomechanical cooperativity C_om = 4 * g^2 / (kappa * gamma_m).
    pub optomechanical_cooperativity: f64,
}

/// Spectral line in the synthesized optical frequency comb.
#[derive(Debug, Clone)]
pub struct FrequencyCombLine {
    /// Sideband order m in [-3, +3].
    pub order: i32,
    /// Absolute optical frequency in THz.
    pub frequency_thz: f64,
    /// Offset from carrier in GHz.
    pub offset_ghz: f64,
    /// Normalized spectral line power in dB.
    pub power_db: f64,
}

/// Phase noise data point across offset frequency.
#[derive(Debug, Clone)]
pub struct OptoAcousticPhaseNoisePoint {
    /// Offset frequency Delta f in kHz.
    pub offset_freq_khz: f64,
    /// Single-sideband phase noise L(Delta f) in dBc/Hz.
    pub phase_noise_dbc_hz: f64,
}

/// Solver engine for opto-acoustic frequency synthesis and quantum transduction.
#[derive(Debug, Clone)]
pub struct OptoAcousticSynthesizerSolver {
    params: OptoAcousticSynthesizerParams,
}

impl OptoAcousticSynthesizerSolver {
    /// Constructs a new opto-acoustic synthesizer solver.
    pub fn new(params: OptoAcousticSynthesizerParams) -> Self {
        Self { params }
    }

    /// Returns a reference to the active parameters.
    pub fn params(&self) -> &OptoAcousticSynthesizerParams {
        &self.params
    }

    /// Evaluates Bessel functions J_0, J_1, J_2, J_3 for modulation index beta.
    fn evaluate_bessel_j(beta: f64) -> (f64, f64, f64, f64) {
        // Taylor series approximations valid for beta in [0.0, 2.5]
        let b2 = beta * beta;
        let b4 = b2 * b2;
        let b6 = b4 * b2;

        let j0 = 1.0 - (b2 / 4.0) + (b4 / 64.0) - (b6 / 2304.0);
        let j1 = (beta / 2.0) * (1.0 - (b2 / 8.0) + (b4 / 192.0) - (b6 / 9216.0));
        let j2 = (b2 / 8.0) * (1.0 - (b2 / 12.0) + (b4 / 384.0));
        let j3 = (b2 * beta / 48.0) * (1.0 - (b2 / 16.0));

        (j0, j1, j2, j3)
    }

    /// Evaluates opto-acoustic synthesizer metrics.
    pub fn evaluate_metrics(&self) -> OptoAcousticMetrics {
        // Modulation depth beta proportional to sqrt(P_acoustic) * g_om / Omega_m
        let p_mw = self.params.acoustic_drive_power_mw.max(0.01);
        let g_mhz = self.params.optoacoustic_coupling_mhz;
        let omega_m_mhz = self.params.microwave_acoustic_freq_ghz * 1.0e3;

        let beta_raw = 0.58 * (p_mw.sqrt()) * (g_mhz / 15.0) * (3600.0 / omega_m_mhz);
        let beta = beta_raw.clamp(0.05, 2.4);

        let (j0, j1, j2, _j3) = Self::evaluate_bessel_j(beta);

        let p_carrier = (j0 * j0).max(1.0e-5);
        let p_sideband1 = j1 * j1;
        let p_sideband2 = j2 * j2;

        let carrier_suppression_db = 10.0 * (p_carrier).log10();

        // Optomechanical cooperativity:
        // C_om = 4 * g^2 / (kappa * gamma_m)
        let kappa_mhz = self.params.optical_cavity_linewidth_ghz * 1.0e3;
        let gamma_mhz = self.params.acoustic_damping_mhz;
        let c_om = 4.0 * (g_mhz * g_mhz * p_mw) / (kappa_mhz * gamma_mhz).max(1.0);

        // Transduction efficiency:
        // eta = 4 * C_om / (1 + C_om)^2 * eta_piezo
        let internal_eta = (4.0 * c_om) / ((1.0 + c_om).powi(2));
        let total_eta = internal_eta * self.params.piezo_transduction_efficiency;

        // Phase noise at 10 kHz offset:
        // High acoustic Q cavity phase noise <= -100 dBc/Hz
        let q_acoustic = (omega_m_mhz / gamma_mhz).max(100.0);
        let phase_noise_10khz = -102.0 - (10.0 * (q_acoustic / 1000.0).log10()).clamp(-20.0, 10.0);

        // Quantum added noise n_add = 0.5 * (1 + 1 / C_om)
        let n_add = 0.5 * (1.0 + (1.0 / c_om.max(0.1))).clamp(0.55, 3.5);

        OptoAcousticMetrics {
            modulation_index_beta: beta,
            carrier_suppression_ratio_db: carrier_suppression_db,
            first_sideband_power_fraction: p_sideband1,
            second_sideband_power_fraction: p_sideband2,
            microwave_to_optical_efficiency: total_eta.clamp(0.01, 0.95),
            phase_noise_at_10khz_dbc_hz: phase_noise_10khz,
            quantum_added_noise_quanta: n_add,
            optomechanical_cooperativity: c_om,
        }
    }

    /// Computes discrete optical frequency comb spectrum for orders m in [-3, +3].
    pub fn compute_frequency_comb_spectrum(&self) -> Vec<FrequencyCombLine> {
        let metrics = self.evaluate_metrics();
        let (j0, j1, j2, j3) = Self::evaluate_bessel_j(metrics.modulation_index_beta);

        let carrier_thz = self.params.carrier_frequency_thz;
        let acoustic_ghz = self.params.microwave_acoustic_freq_ghz;

        let orders: [i32; 7] = [-3, -2, -1, 0, 1, 2, 3];
        let mut lines = Vec::with_capacity(orders.len());

        for &m in &orders {
            let amp = match m.abs() {
                0 => j0.abs(),
                1 => j1.abs(),
                2 => j2.abs(),
                _ => j3.abs(),
            };
            let power_frac = (amp * amp).max(1.0e-5);
            let power_db = 10.0 * power_frac.log10();
            let offset_ghz = (m as f64) * acoustic_ghz;
            let freq_thz = carrier_thz + (offset_ghz * 1.0e-3);

            lines.push(FrequencyCombLine {
                order: m,
                frequency_thz: freq_thz,
                offset_ghz,
                power_db,
            });
        }

        lines
    }

    /// Computes phase noise spectrum L(Delta f) vs offset frequency Delta f [1 kHz to 10 MHz].
    pub fn compute_phase_noise_spectrum(&self, points: usize) -> Vec<OptoAcousticPhaseNoisePoint> {
        let pts = points.max(16);
        let base_10khz = self.evaluate_metrics().phase_noise_at_10khz_dbc_hz;
        let mut results = Vec::with_capacity(pts);

        let min_log = 0.0; // 1 kHz = 10^0 kHz
        let max_log = 4.0; // 10,000 kHz (10 MHz) = 10^4 kHz

        for i in 0..pts {
            let frac = (i as f64) / ((pts - 1) as f64);
            let log_f = min_log + frac * (max_log - min_log);
            let offset_khz = 10.0f64.powf(log_f);

            // 1/f^2 phase noise slope (-20 dB/decade) relative to 10 kHz
            let delta_decades = (offset_khz / 10.0).log10();
            let noise_dbc = base_10khz - (20.0 * delta_decades);

            results.push(OptoAcousticPhaseNoisePoint {
                offset_freq_khz: offset_khz,
                phase_noise_dbc_hz: noise_dbc.clamp(-160.0, -40.0),
            });
        }

        results
    }

    /// Computes transduction efficiency vs applied microwave power.
    /// Returns (power_mw, efficiency_ratio).
    pub fn compute_efficiency_vs_power(&self, points: usize) -> Vec<(f64, f64)> {
        let pts = points.max(16);
        let p_max = self.params.acoustic_drive_power_mw * 3.0;
        let mut results = Vec::with_capacity(pts);

        for i in 0..pts {
            let frac = (i as f64) / ((pts - 1) as f64);
            let p_mw = 0.05 + frac * p_max;

            let mut temp_params = self.params.clone();
            temp_params.acoustic_drive_power_mw = p_mw;
            let temp_solver = Self::new(temp_params);
            let eff = temp_solver.evaluate_metrics().microwave_to_optical_efficiency;

            results.push((p_mw, eff));
        }

        results
    }
}
