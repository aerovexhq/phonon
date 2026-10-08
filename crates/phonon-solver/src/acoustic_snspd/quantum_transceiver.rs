#![deny(unsafe_code)]

//! Gigahertz-rate quantum acoustic transceiver, single-phonon state discrimination,
//! and timing jitter distribution engine.
//!
//! Models quantum state discrimination across phononic Fock states (|0>, |1>, |2>),
//! instrumental response function (IRF) timing jitter histograms, maximum count rates (MCR),
//! quantum bit error rate (QBER), and continuous secret key generation rate.

use std::f64::consts::PI;

/// Parameters for gigahertz-rate quantum acoustic communications.
#[derive(Debug, Clone, PartialEq)]
pub struct QuantumTransceiverParams {
    /// Quantum communication repetition clock rate in megahertz (e.g. 1000.0 MHz = 1.0 GHz).
    pub repetition_rate_mhz: f64,
    /// Mean photon/phonon number per communication pulse (mu, e.g. 0.15).
    pub mean_phonon_number: f64,
    /// Channel transmission distance in meters (e.g. 50.0 m).
    pub acoustic_distance_m: f64,
    /// Topological waveguide attenuation in decibels per meter (e.g. 0.05 dB/m).
    pub waveguide_loss_db_m: f64,
    /// Topological defect bend insertion loss in decibels (e.g. 0.08 dB).
    pub topological_bend_loss_db: f64,
    /// Detector dead time in nanoseconds (e.g. 1.1 ns).
    pub dead_time_ns: f64,
    /// Detector dark count rate in Hertz (e.g. 5.0 Hz).
    pub dark_count_rate_hz: f64,
    /// Detector quantum efficiency in percent (e.g. 91.5%).
    pub detector_efficiency_percent: f64,
    /// Timing jitter FWHM in picoseconds (e.g. 3.2 ps).
    pub timing_jitter_fwhm_ps: f64,
    /// Sifting protocol error correction efficiency factor (f_EC, typically 1.15).
    pub error_correction_factor: f64,
}

impl Default for QuantumTransceiverParams {
    fn default() -> Self {
        Self {
            repetition_rate_mhz: 1000.0,
            mean_phonon_number: 0.15,
            acoustic_distance_m: 50.0,
            waveguide_loss_db_m: 0.05,
            topological_bend_loss_db: 0.08,
            dead_time_ns: 1.1,
            dark_count_rate_hz: 5.0,
            detector_efficiency_percent: 91.5,
            timing_jitter_fwhm_ps: 3.2,
            error_correction_factor: 1.15,
        }
    }
}

/// Point on the timing jitter instrumental response function (IRF) histogram.
#[derive(Debug, Clone, PartialEq)]
pub struct JitterHistogramPoint {
    /// Relative arrival time offset in picoseconds.
    pub time_offset_ps: f64,
    /// Normalized probability density / counts.
    pub count_density: f64,
}

/// Population and discrimination probability for phononic Fock states.
#[derive(Debug, Clone, PartialEq)]
pub struct FockDiscriminationPoint {
    /// Fock state index n (0, 1, 2, 3).
    pub fock_state: usize,
    /// Theoretical Poissonian or state probability.
    pub probability: f64,
    /// Measurement fidelity / discrimination accuracy.
    pub discrimination_fidelity: f64,
}

/// Quantum transceiver performance metrics.
#[derive(Debug, Clone, PartialEq)]
pub struct TransceiverMetrics {
    /// Total channel transmission efficiency (0.0 to 1.0).
    pub channel_transmittance: f64,
    /// Raw detection count rate in megacounts per second (Mcps).
    pub raw_count_rate_mcps: f64,
    /// Maximum count rate (MCR = 1 / tau_dead) in megacounts per second (Mcps).
    pub max_count_rate_mcps: f64,
    /// Quantum Bit Error Rate (QBER) in percent.
    pub qber_percent: f64,
    /// Sifted secret key generation rate in megabits per second (Mbps).
    pub secret_key_rate_mbps: f64,
    /// Single-phonon state discrimination fidelity (for |1>).
    pub single_phonon_fidelity: f64,
    /// Channel acoustic transit time in nanoseconds.
    pub acoustic_flight_time_ns: f64,
    /// Signal-to-noise ratio in decibels.
    pub signal_to_noise_ratio_db: f64,
}

/// Quantum transceiver physics engine.
#[derive(Debug, Clone)]
pub struct QuantumTransceiverEngine {
    pub params: QuantumTransceiverParams,
}

impl QuantumTransceiverEngine {
    pub fn new(params: QuantumTransceiverParams) -> Self {
        Self { params }
    }

    /// Evaluates quantum acoustic transceiver communication performance.
    pub fn evaluate_transceiver_metrics(&self) -> TransceiverMetrics {
        let p = &self.params;

        // Channel loss in dB
        let total_channel_loss_db = p.acoustic_distance_m * p.waveguide_loss_db_m + p.topological_bend_loss_db;
        let channel_transmittance = 10.0_f64.powf(-total_channel_loss_db / 10.0);

        // Acoustic flight time at v_acoustic ~ 3500 m/s
        let acoustic_speed_m_s = 3500.0;
        let acoustic_flight_time_ns = (p.acoustic_distance_m / acoustic_speed_m_s) * 1e9;

        // Effective overall detection probability per pulse
        let eta_det = (p.detector_efficiency_percent / 100.0) * channel_transmittance;
        let signal_prob_per_pulse = 1.0 - (-p.mean_phonon_number * eta_det).exp();

        // Dark count probability per pulse: P_dark = DCR * (gate_window ~ 2 * tau_jitter)
        let gate_window_s = (p.timing_jitter_fwhm_ps * 2.0) * 1e-12;
        let dark_prob_per_pulse = p.dark_count_rate_hz * gate_window_s;

        let total_prob_per_pulse = signal_prob_per_pulse + dark_prob_per_pulse;

        // Count rates
        let raw_count_rate_s = p.repetition_rate_mhz * 1e6 * total_prob_per_pulse;
        let dead_time_s = p.dead_time_ns * 1e-9;
        // Saturation correction: R_meas = R_raw / (1 + R_raw * tau_dead)
        let saturated_count_rate_s = raw_count_rate_s / (1.0 + raw_count_rate_s * dead_time_s);
        let raw_count_rate_mcps = saturated_count_rate_s / 1e6;
        let max_count_rate_mcps = (1.0 / dead_time_s) / 1e6;

        // Quantum Bit Error Rate (QBER)
        // QBER = (0.5 * P_dark + e_optical * P_signal) / (P_signal + P_dark)
        let intrinsic_optical_error = 0.005; // 0.5% baseline optical/acoustic misalignment
        let error_prob = 0.5 * dark_prob_per_pulse + intrinsic_optical_error * signal_prob_per_pulse;
        let qber = error_prob / total_prob_per_pulse.max(1e-12);
        let qber_percent = (qber * 100.0).clamp(0.01, 50.0);

        // Binary Shannon entropy H_2(QBER)
        let q = qber.clamp(1e-6, 0.5);
        let h2 = -q * q.log2() - (1.0 - q) * (1.0 - q).log2();

        // Secret key rate: R_key = R_meas * [1 - (1 + f_EC) * H_2(QBER)]
        let key_fraction = (1.0 - (1.0 + p.error_correction_factor) * h2).max(0.0);
        let secret_key_rate_bps = saturated_count_rate_s * key_fraction;
        let secret_key_rate_mbps = secret_key_rate_bps / 1e6;

        // Signal to noise ratio: 10 * log10(signal / dark)
        let snr_db = 10.0 * (signal_prob_per_pulse / dark_prob_per_pulse.max(1e-15)).log10();

        // Single phonon fidelity
        let single_phonon_fidelity = (1.0 - qber).clamp(0.0, 1.0);

        TransceiverMetrics {
            channel_transmittance,
            raw_count_rate_mcps,
            max_count_rate_mcps,
            qber_percent,
            secret_key_rate_mbps,
            single_phonon_fidelity,
            acoustic_flight_time_ns,
            signal_to_noise_ratio_db: snr_db.min(60.0),
        }
    }

    /// Generates timing jitter Gaussian Instrumental Response Function (IRF) points.
    pub fn generate_jitter_histogram(&self) -> Vec<JitterHistogramPoint> {
        let fwhm = self.params.timing_jitter_fwhm_ps;
        let sigma = fwhm / 2.35482;
        let range_ps = fwhm * 3.5;
        let steps = 100;
        let dt = (2.0 * range_ps) / (steps as f64);

        (0..=steps).map(|i| {
            let t = -range_ps + (i as f64) * dt;
            let exponent = -0.5 * (t / sigma).powi(2);
            let density = (1.0 / (sigma * (2.0 * PI).sqrt())) * exponent.exp();
            JitterHistogramPoint {
                time_offset_ps: t,
                count_density: density * sigma * (2.0 * PI).sqrt(), // normalized peak to 1.0
            }
        }).collect()
    }

    /// Generates Fock state discrimination distribution points for states |0>, |1>, |2>, |3>.
    pub fn generate_fock_discrimination(&self) -> Vec<FockDiscriminationPoint> {
        let mu = self.params.mean_phonon_number;
        let p_0 = (-mu).exp();
        let p_1 = (-mu).exp() * mu;
        let p_2 = (-mu).exp() * (mu.powi(2) / 2.0);
        let p_3 = (-mu).exp() * (mu.powi(3) / 6.0);

        vec![
            FockDiscriminationPoint {
                fock_state: 0,
                probability: p_0,
                discrimination_fidelity: 0.998,
            },
            FockDiscriminationPoint {
                fock_state: 1,
                probability: p_1,
                discrimination_fidelity: 0.985,
            },
            FockDiscriminationPoint {
                fock_state: 2,
                probability: p_2,
                discrimination_fidelity: 0.942,
            },
            FockDiscriminationPoint {
                fock_state: 3,
                probability: p_3,
                discrimination_fidelity: 0.890,
            },
        ]
    }
}
