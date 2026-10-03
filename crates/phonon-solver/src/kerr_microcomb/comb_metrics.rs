#![deny(unsafe_code)]

//! Optical & Phononic Frequency Comb Spectrum and Soliton Metrics.
//!
//! Provides spectral decomposition into comb line powers S(mu) in dBm,
//! comb bandwidth evaluation at 3-dB, 10-dB, and 20-dB thresholds,
//! and hyperbolic secant (sech^2) pulse fitting with FWHM duration in picoseconds.

use crate::kerr_microcomb::lle_solver::{fft_1d, MicrocombState, MicroresonatorParams};
use std::f64::consts::PI;

/// Factor relating FWHM to characteristic soliton width: 2 * acosh(sqrt(2)) ~ 1.762747174
const SECH2_FWHM_FACTOR: f64 = 1.762747174092892;

/// Optical/acoustic Kerr microcomb frequency spectrum and temporal soliton metrics.
#[derive(Debug, Clone, PartialEq)]
pub struct CombSpectrum {
    /// Relative mode numbers mu in -N/2 .. N/2 - 1 (mu = 0 is pump line).
    pub mu: Vec<i32>,
    /// Comb line powers in dBm: S(mu) = 10 * log10(|tilde_psi_mu|^2 / P_ref).
    pub powers_dbm: Vec<f64>,
    /// Relative comb line powers in dB below the peak line: S(mu) - max(S).
    pub relative_powers_db: Vec<f64>,
    /// Repetition rate f_rep = FSR in Hz.
    pub repetition_rate_hz: f64,
    /// 3-dB comb bandwidth in Hz.
    pub bandwidth_3db_hz: f64,
    /// Number of comb modes within the 3-dB bandwidth span.
    pub mode_count_3db: usize,
    /// 10-dB comb bandwidth in Hz.
    pub bandwidth_10db_hz: f64,
    /// Number of comb modes within the 10-dB bandwidth span.
    pub mode_count_10db: usize,
    /// 20-dB comb bandwidth in Hz.
    pub bandwidth_20db_hz: f64,
    /// Number of comb modes within the 20-dB bandwidth span.
    pub mode_count_20db: usize,
    /// Azimuthal angles theta in [-pi, pi).
    pub theta: Vec<f64>,
    /// Soliton temporal pulse profile intensity |psi(theta)|^2.
    pub intensity: Vec<f64>,
    /// Hyperbolic secant sech^2 analytical fit to the localized pulse.
    pub sech2_fit: Vec<f64>,
    /// Temporal pulse duration tau_FWHM in picoseconds (ps).
    pub tau_fwhm_ps: f64,
    /// Peak intensity max(|psi|^2).
    pub peak_intensity: f64,
    /// CW background intensity min(|psi|^2).
    pub background_intensity: f64,
    /// Soliton pulse center azimuthal angle theta_0 in radians.
    pub pulse_center_theta: f64,
    /// Soliton pulse width parameter theta_s in radians.
    pub pulse_width_rad: f64,
    /// R^2 goodness of fit for sech^2 envelope.
    pub fit_r_squared: f64,
}

impl CombSpectrum {
    /// Computes frequency comb spectrum and pulse metrics from the solver state.
    ///
    /// `p_ref_mw`: reference power for 0 dBm in mW (typically 1.0 mW).
    pub fn from_state(
        state: &MicrocombState,
        params: &MicroresonatorParams,
        p_ref_mw: f64,
    ) -> Self {
        let n = state.psi.len();
        let repetition_rate_hz = params.fsr_hz;
        let p_ref = if p_ref_mw > 0.0 { p_ref_mw } else { 1.0 };

        // 1. Modal Fourier Transform
        let fft_raw = fft_1d(&state.psi, false);

        // Map modes mu in -n/2 .. n/2 - 1
        let half = (n / 2) as i32;
        let mut modes: Vec<(i32, f64)> = Vec::with_capacity(n);

        for (k, val) in fft_raw.iter().enumerate() {
            let mu = if (k as i32) < half {
                k as i32
            } else {
                (k as i32) - (n as i32)
            };
            // Spectral power: |tilde_psi_mu|^2 with normalization 1 / N
            let amp_sq = val.norm_sq() / ((n * n) as f64);
            // Convert to physical mW scale (intracavity photon/power factor)
            let power_mw = amp_sq * 10.0;
            modes.push((mu, power_mw));
        }

        // Sort by mode index mu ascending from -half to half - 1
        modes.sort_by_key(|&(m, _)| m);

        let mut mu_vec = Vec::with_capacity(n);
        let mut powers_dbm = Vec::with_capacity(n);

        let mut peak_dbm = -1000.0;
        for &(m, p_mw) in &modes {
            mu_vec.push(m);
            let s_dbm = 10.0 * (p_mw.max(1e-15) / p_ref).log10();
            if s_dbm > peak_dbm {
                peak_dbm = s_dbm;
            }
            powers_dbm.push(s_dbm);
        }

        let mut relative_powers_db = Vec::with_capacity(n);
        for &s in &powers_dbm {
            relative_powers_db.push(s - peak_dbm);
        }

        // 2. Comb Bandwidth Metrics: 3-dB, 10-dB, 20-dB
        let (mode_count_3db, bandwidth_3db_hz) =
            Self::calc_bandwidth(&mu_vec, &powers_dbm, peak_dbm - 3.0, repetition_rate_hz);
        let (mode_count_10db, bandwidth_10db_hz) =
            Self::calc_bandwidth(&mu_vec, &powers_dbm, peak_dbm - 10.0, repetition_rate_hz);
        let (mode_count_20db, bandwidth_20db_hz) =
            Self::calc_bandwidth(&mu_vec, &powers_dbm, peak_dbm - 20.0, repetition_rate_hz);

        // 3. Temporal Pulse Profile & Sech^2 Fit
        let theta = state.theta.clone();
        let intensity: Vec<f64> = state.psi.iter().map(|p| p.norm_sq()).collect();

        let (
            sech2_fit,
            tau_fwhm_ps,
            peak_intensity,
            background_intensity,
            pulse_center_theta,
            pulse_width_rad,
            fit_r_squared,
        ) = Self::fit_sech2(&theta, &intensity, repetition_rate_hz);

        Self {
            mu: mu_vec,
            powers_dbm,
            relative_powers_db,
            repetition_rate_hz,
            bandwidth_3db_hz,
            mode_count_3db,
            bandwidth_10db_hz,
            mode_count_10db,
            bandwidth_20db_hz,
            mode_count_20db,
            theta,
            intensity,
            sech2_fit,
            tau_fwhm_ps,
            peak_intensity,
            background_intensity,
            pulse_center_theta,
            pulse_width_rad,
            fit_r_squared,
        }
    }

    /// Evaluates bandwidth and mode count above a threshold power level in dBm.
    fn calc_bandwidth(
        mu: &[i32],
        powers: &[f64],
        threshold_dbm: f64,
        f_rep: f64,
    ) -> (usize, f64) {
        let mut matching_modes = Vec::new();
        for (&m, &p) in mu.iter().zip(powers.iter()) {
            if p >= threshold_dbm {
                matching_modes.push(m);
            }
        }

        if matching_modes.is_empty() {
            (0, 0.0)
        } else {
            let count = matching_modes.len();
            let min_m = *matching_modes.first().unwrap();
            let max_m = *matching_modes.last().unwrap();
            let span_modes = (max_m - min_m).max(1) as f64;
            let bw_hz = span_modes * f_rep;
            (count, bw_hz)
        }
    }

    /// Fits an analytical hyperbolic secant sech^2 envelope to the temporal intensity profile.
    fn fit_sech2(
        theta: &[f64],
        intensity: &[f64],
        f_rep: f64,
    ) -> (Vec<f64>, f64, f64, f64, f64, f64, f64) {
        let n = intensity.len();
        if n == 0 {
            return (Vec::new(), 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        }

        // Find peak index and background intensity
        let mut max_idx = 0;
        let mut max_val = intensity[0];
        let mut min_val = intensity[0];

        for (i, &val) in intensity.iter().enumerate() {
            if val > max_val {
                max_val = val;
                max_idx = i;
            }
            if val < min_val {
                min_val = val;
            }
        }

        let height = max_val - min_val;
        let center_theta = theta[max_idx];

        if height < 1e-4 {
            // Flat CW field: no localized pulse
            let fit = vec![max_val; n];
            return (fit, 0.0, max_val, min_val, center_theta, 0.0, 1.0);
        }

        let half_level = min_val + 0.5 * height;

        // Search right for half-maximum crossing
        let mut right_angle = center_theta;
        for step in 1..n {
            let idx = (max_idx + step) % n;
            if intensity[idx] <= half_level {
                let prev_idx = (max_idx + step + n - 1) % n;
                let val_prev = intensity[prev_idx];
                let val_curr = intensity[idx];
                let frac = if (val_prev - val_curr).abs() > 1e-12 {
                    (val_prev - half_level) / (val_prev - val_curr)
                } else {
                    0.5
                };
                let d_ang = 2.0 * PI / n as f64;
                right_angle = center_theta + ((step as f64) - 1.0 + frac) * d_ang;
                break;
            }
        }

        // Search left for half-maximum crossing
        let mut left_angle = center_theta;
        for step in 1..n {
            let idx = (max_idx + n - step) % n;
            if intensity[idx] <= half_level {
                let prev_idx = (max_idx + n - step + 1) % n;
                let val_prev = intensity[prev_idx];
                let val_curr = intensity[idx];
                let frac = if (val_prev - val_curr).abs() > 1e-12 {
                    (val_prev - half_level) / (val_prev - val_curr)
                } else {
                    0.5
                };
                let d_ang = 2.0 * PI / n as f64;
                left_angle = center_theta - ((step as f64) - 1.0 + frac) * d_ang;
                break;
            }
        }

        let fwhm_rad = (right_angle - left_angle).abs().clamp(1e-4, 2.0 * PI);
        let theta_s = fwhm_rad / SECH2_FWHM_FACTOR;

        // Temporal pulse duration tau_FWHM in seconds = (fwhm_rad / (2 * pi)) * (1 / f_rep)
        let tau_sec = (fwhm_rad / (2.0 * PI)) / f_rep.max(1.0);
        let tau_fwhm_ps = tau_sec * 1.0e12;

        // Generate sech^2 analytical fit profile
        let mut sech2_fit = Vec::with_capacity(n);
        let mut ss_tot = 0.0;
        let mut ss_res = 0.0;
        let mean_int: f64 = intensity.iter().sum::<f64>() / n as f64;

        for (i, &th) in theta.iter().enumerate() {
            let mut diff = (th - center_theta).abs();
            if diff > PI {
                diff = 2.0 * PI - diff;
            }
            let x = diff / theta_s.max(1e-6);
            let sech = 2.0 / (x.exp() + (-x).exp());
            let fit_val = min_val + height * sech * sech;
            sech2_fit.push(fit_val);

            ss_tot += (intensity[i] - mean_int).powi(2);
            ss_res += (intensity[i] - fit_val).powi(2);
        }

        let fit_r_squared = if ss_tot > 1e-12 {
            (1.0 - ss_res / ss_tot).clamp(0.0, 1.0)
        } else {
            1.0
        };

        (
            sech2_fit,
            tau_fwhm_ps,
            max_val,
            min_val,
            center_theta,
            theta_s,
            fit_r_squared,
        )
    }

    /// 3-dB comb bandwidth in GHz.
    #[inline]
    pub fn bandwidth_3db_ghz(&self) -> f64 {
        self.bandwidth_3db_hz * 1.0e-9
    }

    /// 10-dB comb bandwidth in GHz.
    #[inline]
    pub fn bandwidth_10db_ghz(&self) -> f64 {
        self.bandwidth_10db_hz * 1.0e-9
    }

    /// 20-dB comb bandwidth in GHz.
    #[inline]
    pub fn bandwidth_20db_ghz(&self) -> f64 {
        self.bandwidth_20db_hz * 1.0e-9
    }

    /// Repetition rate in MHz.
    #[inline]
    pub fn repetition_rate_mhz(&self) -> f64 {
        self.repetition_rate_hz * 1.0e-6
    }

    /// Power of the central pump line (mu = 0) in dBm.
    pub fn pump_power_dbm(&self) -> f64 {
        for (&m, &p) in self.mu.iter().zip(self.powers_dbm.iter()) {
            if m == 0 {
                return p;
            }
        }
        self.powers_dbm.first().copied().unwrap_or(0.0)
    }
}
