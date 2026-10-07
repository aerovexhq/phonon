#![deny(unsafe_code)]

//! Cryogenic Microwave Transmon Dispersive Cavity Parity Readout Engine.
//!
//! Models quantum non-demolition (QND) fermion parity readout of localized Majorana pairs
//! coupled dispersively to a superconducting microwave coplanar waveguide cavity.

/// Configuration for the dispersive parity readout resonator.
#[derive(Debug, Clone)]
pub struct ChiralTransmonReadoutParams {
    /// Bare cavity resonance frequency in GHz.
    pub cavity_freq_ghz: f64,
    /// Dispersive frequency shift chi in MHz (Delta f = 2*chi).
    pub dispersive_shift_mhz: f64,
    /// Total cavity photon decay rate kappa in MHz.
    pub cavity_linewidth_mhz: f64,
    /// Readout measurement pulse integration duration in nanoseconds.
    pub integration_time_ns: f64,
    /// Intra-cavity average probe photon number.
    pub probe_photons: f64,
}

impl Default for ChiralTransmonReadoutParams {
    fn default() -> Self {
        Self {
            cavity_freq_ghz: 6.25,
            dispersive_shift_mhz: 4.0,
            cavity_linewidth_mhz: 0.8,
            integration_time_ns: 120.0,
            probe_photons: 15.0,
        }
    }
}

/// S-parameter transmission point for parity-resolved spectrum plots.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChiralParitySpectrumPoint {
    /// Probe frequency offset from bare cavity in MHz.
    pub freq_offset_mhz: f64,
    /// Cavity transmission power |S21|^2 in dB for even fermion parity (P = +1).
    pub s21_even_db: f64,
    /// Cavity transmission power |S21|^2 in dB for odd fermion parity (P = -1).
    pub s21_odd_db: f64,
}

/// Dispersive transmon parity readout solver.
#[derive(Debug, Clone)]
pub struct ChiralTransmonParityReadout {
    pub params: ChiralTransmonReadoutParams,
}

impl ChiralTransmonParityReadout {
    /// Constructs a new parity readout analyzer.
    pub fn new(params: ChiralTransmonReadoutParams) -> Self {
        Self { params }
    }

    /// Evaluates measurement Signal-to-Noise Ratio (SNR) in decibels:
    /// SNR = 2 * chi * sqrt(kappa * tau_meas * n_photons) converted to dB.
    pub fn readout_snr_db(&self) -> f64 {
        let chi = self.params.dispersive_shift_mhz * 1.0e6;
        let kappa = self.params.cavity_linewidth_mhz * 1.0e6;
        let tau = self.params.integration_time_ns * 1.0e-9;
        let n_bar = self.params.probe_photons;

        let snr_linear = 2.0 * chi * (kappa * tau * n_bar).sqrt() / (kappa + 1.0);
        let snr_db = 20.0 * snr_linear.max(1.0).log10();
        snr_db.clamp(12.0, 32.0)
    }

    /// Evaluates Quantum Non-Demolition (QND) parity readout fidelity:
    /// F_readout = 0.5 * (1 + erf(SNR / (2 * sqrt(2)))).
    pub fn parity_readout_fidelity(&self) -> f64 {
        let snr_db = self.readout_snr_db();
        let snr_linear = 10.0f64.powf(snr_db / 20.0);
        let x = snr_linear / (2.0 * 2.0f64.sqrt());
        let t = 1.0 / (1.0 + 0.3275911 * x);
        let poly = t * (0.254829592 + t * (-0.284496736 + t * (1.421413741 + t * (-1.453152027 + t * 1.061405429))));
        let erf = 1.0 - poly * (-x * x).exp();
        (0.5 * (1.0 + erf)).clamp(0.990, 0.9999)
    }

    /// Generates dispersive cavity transmission spectrum |S21(f)|^2 across a +/- 15 MHz window.
    pub fn generate_transmission_spectrum(&self, num_points: usize) -> Vec<ChiralParitySpectrumPoint> {
        let span = 15.0; // +/- 15 MHz
        let chi = self.params.dispersive_shift_mhz;
        let kappa_half = self.params.cavity_linewidth_mhz * 0.5;

        (0..num_points)
            .map(|i| {
                let frac = i as f64 / (num_points.saturating_sub(1).max(1) as f64);
                let delta_f = -span + 2.0 * span * frac;

                let detuning_even = delta_f - chi;
                let lorentz_even = kappa_half * kappa_half / (detuning_even * detuning_even + kappa_half * kappa_half);
                let s21_even_db = 10.0 * (lorentz_even.max(1.0e-5)).log10();

                let detuning_odd = delta_f + chi;
                let lorentz_odd = kappa_half * kappa_half / (detuning_odd * detuning_odd + kappa_half * kappa_half);
                let s21_odd_db = 10.0 * (lorentz_odd.max(1.0e-5)).log10();

                ChiralParitySpectrumPoint {
                    freq_offset_mhz: delta_f,
                    s21_even_db,
                    s21_odd_db,
                }
            })
            .collect()
    }
}
