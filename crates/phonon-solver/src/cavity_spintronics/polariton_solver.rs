//! Polariton Spectrum & Anti-Crossing Solver.
//!
//! Solves microwave cavity transmission $S_{21}(\omega)$, cavity-magnon polariton
//! branch dispersions, anti-crossing gaps, and polariton linewidths.

use phonon_models::cavity_spintronics::MagnonCavityCoupling;
use std::f64::consts::PI;

/// Microwave transmission spectrum result across a frequency band.
#[derive(Debug, Clone, PartialEq)]
pub struct TransmissionSpectrum {
    /// Probe frequencies in Hz.
    pub frequencies_hz: Vec<f64>,
    /// Transmission magnitude $|S_{21}|$.
    pub s21_magnitude: Vec<f64>,
    /// Transmission in decibels: $20 \log_{10}(|S_{21}|)$.
    pub s21_db: Vec<f64>,
    /// Transmission phase $\arg(S_{21})$ in radians.
    pub s21_phase: Vec<f64>,
}

impl TransmissionSpectrum {
    /// Finds the peak frequencies in the transmission spectrum (polariton resonance peaks).
    pub fn peak_frequencies_hz(&self) -> Vec<f64> {
        let n = self.s21_magnitude.len();
        if n < 3 {
            return Vec::new();
        }

        let mut peaks = Vec::new();
        let max_val = self.s21_magnitude.iter().cloned().fold(0.0, f64::max);
        let threshold = max_val * 0.1;

        for i in 1..n - 1 {
            let prev = self.s21_magnitude[i - 1];
            let curr = self.s21_magnitude[i];
            let next = self.s21_magnitude[i + 1];

            if curr > prev && curr > next && curr > threshold {
                peaks.push(self.frequencies_hz[i]);
            }
        }

        peaks
    }
}

/// Point on the polariton branch dispersion diagram.
#[derive(Debug, Clone, PartialEq)]
pub struct PolaritonBranchPoint {
    /// DC bias magnetic field $B_0$ in Tesla.
    pub bias_field_t: f64,
    /// Uncoupled bare magnon frequency $f_m(B_0)$ in Hz.
    pub magnon_freq_hz: f64,
    /// Uncoupled bare cavity resonance frequency $f_c$ in Hz.
    pub cavity_freq_hz: f64,
    /// Frequency detuning $f_c - f_m$ in Hz.
    pub detuning_hz: f64,
    /// Lower polariton branch frequency in Hz.
    pub lower_polariton_hz: f64,
    /// Upper polariton branch frequency in Hz.
    pub upper_polariton_hz: f64,
    /// Polariton splitting frequency in Hz: $f_+ - f_-$.
    pub splitting_hz: f64,
    /// Lower polariton linewidth in Hz.
    pub lower_linewidth_hz: f64,
    /// Upper polariton linewidth in Hz.
    pub upper_linewidth_hz: f64,
}

/// Microwave cavity polariton solver.
#[derive(Debug, Default, Clone)]
pub struct PolaritonSolver;

impl PolaritonSolver {
    /// Creates a new polariton solver.
    pub fn new() -> Self {
        Self
    }

    /// Evaluates the complex scattering parameter $S_{21}(\omega)$ at a single probe frequency $f$ in Hz.
    ///
    /// $$S_{21}(\omega) = \frac{-i \kappa_{ext}/2}{\omega - \omega_c + i\kappa_c/2 - \frac{g_{mp}^2}{\omega - \omega_m + i\gamma_m/2}}$$
    pub fn evaluate_s21(&self, coupling: &MagnonCavityCoupling, freq_hz: f64) -> (f64, f64, f64) {
        let omega = 2.0 * PI * freq_hz;
        let omega_c = coupling.cavity.omega_c();
        let omega_m = coupling.magnon.kittel_frequency_rad(coupling.bias_field);

        let delta_c = omega - omega_c;
        let delta_m = omega - omega_m;

        let kappa_c = coupling.cavity.total_decay_rate_rad();
        let kappa_ext = coupling.cavity.external_decay_rate_rad();
        let gamma_m = coupling
            .magnon
            .magnon_dissipation_rate_rad(coupling.bias_field);
        let g = coupling.coupling_rate_rad();

        // Magnon susceptibility denominator: Zm = delta_m + i * gamma_m / 2
        let zm_mag_sq = delta_m * delta_m + 0.25 * gamma_m * gamma_m;
        let g_sq = g * g;

        // g^2 / Zm = g^2 * (delta_m - i * gamma_m/2) / |Zm|^2
        let sigma_re = g_sq * delta_m / zm_mag_sq;
        let sigma_im = -g_sq * (0.5 * gamma_m) / zm_mag_sq;

        // Total denominator D = (delta_c - sigma_re) + i * (kappa_c/2 - sigma_im)
        let d_re = delta_c - sigma_re;
        let d_im = 0.5 * kappa_c - sigma_im;
        let d_mag_sq = d_re * d_re + d_im * d_im;

        // Numerator N = -i * (kappa_ext / 2)
        // S21 = N / D = -i * (kappa_ext / 2) * (d_re - i * d_im) / |D|^2
        //     = (- (kappa_ext / 2) * d_im - i * (kappa_ext / 2) * d_re) / |D|^2
        let factor = 0.5 * kappa_ext / d_mag_sq;
        let s21_re = -factor * d_im;
        let s21_im = -factor * d_re;

        let mag = (s21_re * s21_re + s21_im * s21_im).sqrt();
        let db = 20.0 * (mag + 1e-15).log10();
        let phase = s21_im.atan2(s21_re);

        (mag, db, phase)
    }

    /// Computes the transmission spectrum across a frequency grid.
    pub fn compute_transmission(
        &self,
        coupling: &MagnonCavityCoupling,
        frequencies_hz: &[f64],
    ) -> TransmissionSpectrum {
        let mut s21_magnitude = Vec::with_capacity(frequencies_hz.len());
        let mut s21_db = Vec::with_capacity(frequencies_hz.len());
        let mut s21_phase = Vec::with_capacity(frequencies_hz.len());

        for &f in frequencies_hz {
            let (mag, db, phase) = self.evaluate_s21(coupling, f);
            s21_magnitude.push(mag);
            s21_db.push(db);
            s21_phase.push(phase);
        }

        TransmissionSpectrum {
            frequencies_hz: frequencies_hz.to_vec(),
            s21_magnitude,
            s21_db,
            s21_phase,
        }
    }

    /// Scans polariton branches across a range of DC bias magnetic fields $B_0$.
    pub fn scan_branches(
        &self,
        base_coupling: &MagnonCavityCoupling,
        bias_fields_t: &[f64],
    ) -> Vec<PolaritonBranchPoint> {
        let mut points = Vec::with_capacity(bias_fields_t.len());

        for &b0 in bias_fields_t {
            let mut coupling = base_coupling.clone();
            coupling.bias_field = b0;

            let (f_plus, f_minus) = coupling.polariton_frequencies_hz();
            let (gamma_plus, gamma_minus) = coupling.polariton_linewidths_hz();

            let upper = f_plus.max(f_minus);
            let lower = f_plus.min(f_minus);
            let upper_lw = if f_plus >= f_minus {
                gamma_plus
            } else {
                gamma_minus
            };
            let lower_lw = if f_plus >= f_minus {
                gamma_minus
            } else {
                gamma_plus
            };

            points.push(PolaritonBranchPoint {
                bias_field_t: b0,
                magnon_freq_hz: coupling.magnon.kittel_frequency_hz(b0),
                cavity_freq_hz: coupling.cavity.resonant_frequency_hz,
                detuning_hz: coupling.detuning_hz(),
                lower_polariton_hz: lower,
                upper_polariton_hz: upper,
                splitting_hz: upper - lower,
                lower_linewidth_hz: lower_lw,
                upper_linewidth_hz: upper_lw,
            });
        }

        points
    }

    /// Extracts the minimum splitting (anti-crossing gap) across a branch scan.
    pub fn extract_minimum_splitting(branches: &[PolaritonBranchPoint]) -> f64 {
        branches
            .iter()
            .map(|p| p.splitting_hz)
            .fold(f64::INFINITY, f64::min)
    }
}
