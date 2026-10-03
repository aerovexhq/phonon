#![deny(unsafe_code)]

//! Four-Wave Mixing (4WM) and Resonant Phase Matching (RPM) parametric amplification kernel.
//!
//! Models degenerate four-wave mixing:
//! omega_p + omega_p = omega_s + omega_i
//! coupled mode solutions, total phase mismatch, quantum noise squeezing below the Standard Quantum Limit,
//! and added noise quanta approaching Caves' quantum limit.

use super::transmission_line::JosephsonTransmissionLine;

/// Configuration parameters for a JTWPA simulation run.
#[derive(Debug, Clone, PartialEq)]
pub struct JtwpaParams {
    /// Transmission line ladder configuration.
    pub line: JosephsonTransmissionLine,
    /// Pump microwave frequency f_p in Hz (default ~6.0 GHz / 6.0e9 Hz).
    pub pump_freq_hz: f64,
    /// Generator pump power P_p in dBm (default ~-50.0 dBm).
    pub pump_power_dbm: f64,
    /// Signal sweep start frequency in Hz (default 4.0 GHz / 4.0e9 Hz).
    pub signal_freq_start_hz: f64,
    /// Signal sweep stop frequency in Hz (default 8.0 GHz / 8.0e9 Hz).
    pub signal_freq_stop_hz: f64,
    /// Number of frequency sample points across octave sweep (default 100).
    pub sample_points: usize,
    /// Attenuation of microwave drive line inside cryostat in dB (default 34.0 dB).
    pub attenuation_db: f64,
}

impl Default for JtwpaParams {
    fn default() -> Self {
        Self {
            line: JosephsonTransmissionLine::default(),
            pump_freq_hz: 6.0e9,
            pump_power_dbm: -50.0,
            signal_freq_start_hz: 4.0e9,
            signal_freq_stop_hz: 8.0e9,
            sample_points: 100,
            attenuation_db: 34.0,
        }
    }
}

impl JtwpaParams {
    /// Creates custom JTWPA parameters.
    pub fn new(
        line: JosephsonTransmissionLine,
        pump_freq_hz: f64,
        pump_power_dbm: f64,
        signal_freq_start_hz: f64,
        signal_freq_stop_hz: f64,
        sample_points: usize,
        attenuation_db: f64,
    ) -> Self {
        Self {
            line,
            pump_freq_hz: pump_freq_hz.max(1.0e6),
            pump_power_dbm,
            signal_freq_start_hz: signal_freq_start_hz.max(1.0e6),
            signal_freq_stop_hz: signal_freq_stop_hz.max(signal_freq_start_hz + 1.0e6),
            sample_points: sample_points.max(10),
            attenuation_db: attenuation_db.max(0.0),
        }
    }

    /// Effective on-chip pump power in dBm arriving at the JTWPA input port.
    pub fn on_chip_pump_power_dbm(&self) -> f64 {
        self.pump_power_dbm - self.attenuation_db
    }

    /// Effective pump power in Watts: P_p = 10^((P_dBm - 30) / 10).
    pub fn pump_power_watts(&self) -> f64 {
        10.0f64.powf((self.on_chip_pump_power_dbm() - 30.0) / 10.0)
    }
}

/// Simulation result at a single signal frequency point across the gain spectrum.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GainSpectrumPoint {
    /// Signal frequency f_s in GHz.
    pub f_s: f64,
    /// Idler frequency f_i = 2 * f_p - f_s in GHz.
    pub f_i: f64,
    /// Signal power gain in linear scale G_s.
    pub gain_s_linear: f64,
    /// Signal power gain in decibels G_s (dB).
    pub gain_s_db: f64,
    /// Idler power gain in decibels G_i (dB).
    pub gain_i_db: f64,
    /// Total phase mismatch without RPM: Delta_k = 2*k_p - k_s - k_i - 2*gamma_NL*P_p in rad/m.
    pub delta_k: f64,
    /// Total phase mismatch with RPM engineering: Delta_k_rpm in rad/m.
    pub delta_k_rpm: f64,
}

/// Quantum noise squeezing and added noise performance metrics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuantumSqueezing {
    /// Squeezing parameter r = asinh(sqrt(G_s - 1)).
    pub r: f64,
    /// Squeezed quadrature variance S_xx = 0.25 * exp(-2 * r).
    pub s_xx: f64,
    /// Anti-squeezed quadrature variance S_yy = 0.25 * exp(2 * r).
    pub s_yy: f64,
    /// Squeezing level in dB below Standard Quantum Limit (SQL = 0.25): -10 * log10(4 * S_xx).
    pub squeezing_db: f64,
    /// Added noise quanta n_add = 0.5 * (1 - 1 / G_s) approaching quantum limit 0.5.
    pub n_add: f64,
}

impl QuantumSqueezing {
    /// Computes quantum quadrature squeezing metrics from linear signal power gain G_s.
    pub fn from_gain(gain_s_linear: f64) -> Self {
        let g = gain_s_linear.max(1.0);
        let excess_gain = (g - 1.0).max(0.0);
        // r = asinh(sqrt(G - 1)) = ln(sqrt(G) + sqrt(G - 1))
        let r = (g.sqrt() + excess_gain.sqrt()).ln().max(0.0);
        let s_xx = 0.25 * (-2.0 * r).exp();
        let s_yy = 0.25 * (2.0 * r).exp();
        let squeezing_db = -10.0 * (4.0 * s_xx.max(1.0e-30)).log10();
        let n_add = 0.5 * (1.0 - 1.0 / g);

        Self {
            r,
            s_xx,
            s_yy,
            squeezing_db,
            n_add,
        }
    }
}

/// Solver for Four-Wave Mixing (4WM) coupled equations in JTWPA devices.
pub struct JtwpaSolver;

impl JtwpaSolver {
    /// Solves four-wave mixing equations at a single signal frequency.
    pub fn solve_point(params: &JtwpaParams, fs_hz: f64, with_rpm: bool) -> GainSpectrumPoint {
        let fp_hz = params.pump_freq_hz;
        let fi_hz = 2.0 * fp_hz - fs_hz;
        let p_p = params.pump_power_watts();
        let gamma_nl = params.line.kerr_coefficient(fp_hz);
        let g0 = gamma_nl * p_p;
        let length_l = params.line.total_length_m();

        // Bare propagation constants without RPM
        let kp_bare = params.line.dispersion_k(fp_hz, false);
        let ks_bare = params.line.dispersion_k(fs_hz, false);
        let ki_bare = params.line.dispersion_k(fi_hz, false);
        let delta_k_bare = 2.0 * kp_bare - ks_bare - ki_bare - 2.0 * g0;

        // Propagation constants with RPM engineering
        let kp_rpm = params.line.dispersion_k(fp_hz, true);
        let ks_rpm = params.line.dispersion_k(fs_hz, true);
        let ki_rpm = params.line.dispersion_k(fi_hz, true);
        // RPM stubs compensate for the nonlinear Kerr phase shift 2*gamma_NL*P_p
        let delta_k_rpm = 2.0 * kp_rpm - ks_rpm - ki_rpm;

        let active_dk = if with_rpm { delta_k_rpm } else { delta_k_bare };
        let diff = g0 * g0 - (active_dk * 0.5) * (active_dk * 0.5);

        let g_s_linear = if diff > 1.0e-12 {
            let g = diff.sqrt();
            let cosh_term = (g * length_l).cosh().powi(2);
            let sinh_term = (g * length_l).sinh().powi(2);
            cosh_term + ((active_dk / (2.0 * g)).powi(2) * sinh_term)
        } else if diff < -1.0e-12 {
            let kappa = (-diff).sqrt();
            let cos_term = (kappa * length_l).cos().powi(2);
            let sin_term = (kappa * length_l).sin().powi(2);
            cos_term + ((active_dk / (2.0 * kappa)).powi(2) * sin_term)
        } else {
            1.0 + (g0 * length_l).powi(2)
        }
        .max(1.0);

        let g_s_db = 10.0 * g_s_linear.log10();
        let g_i_linear = (g_s_linear - 1.0).max(1.0e-12);
        let g_i_db = 10.0 * g_i_linear.log10();

        GainSpectrumPoint {
            f_s: fs_hz / 1.0e9,
            f_i: fi_hz / 1.0e9,
            gain_s_linear: g_s_linear,
            gain_s_db: g_s_db,
            gain_i_db: g_i_db,
            delta_k: delta_k_bare,
            delta_k_rpm,
        }
    }

    /// Solves the full continuous gain spectrum curve across the octave frequency sweep [f_start, f_stop].
    pub fn solve_spectrum(params: &JtwpaParams) -> Vec<GainSpectrumPoint> {
        let n = params.sample_points.max(10);
        let df = (params.signal_freq_stop_hz - params.signal_freq_start_hz) / (n as f64 - 1.0);
        let with_rpm = params.line.rpm_params.is_some();

        (0..n)
            .map(|i| {
                let fs = params.signal_freq_start_hz + i as f64 * df;
                Self::solve_point(params, fs, with_rpm)
            })
            .collect()
    }

    /// Calculates quantum quadrature squeezing for the specified signal frequency.
    pub fn quantum_squeezing(params: &JtwpaParams, fs_hz: f64) -> QuantumSqueezing {
        let with_rpm = params.line.rpm_params.is_some();
        let point = Self::solve_point(params, fs_hz, with_rpm);
        QuantumSqueezing::from_gain(point.gain_s_linear)
    }

    /// Computes peak gain and 3-dB amplification bandwidth from a gain spectrum.
    /// Returns (peak_gain_db, bandwidth_ghz).
    pub fn bandwidth_3db(spectrum: &[GainSpectrumPoint]) -> (f64, f64) {
        if spectrum.is_empty() {
            return (0.0, 0.0);
        }

        let mut peak_gain_db = f64::NEG_INFINITY;
        for pt in spectrum {
            if pt.gain_s_db > peak_gain_db {
                peak_gain_db = pt.gain_s_db;
            }
        }

        let threshold_db = peak_gain_db - 3.0;
        let mut min_f = f64::INFINITY;
        let mut max_f = f64::NEG_INFINITY;

        for pt in spectrum {
            if pt.gain_s_db >= threshold_db {
                if pt.f_s < min_f {
                    min_f = pt.f_s;
                }
                if pt.f_s > max_f {
                    max_f = pt.f_s;
                }
            }
        }

        let bw = if max_f >= min_f { max_f - min_f } else { 0.0 };
        (peak_gain_db, bw)
    }
}
