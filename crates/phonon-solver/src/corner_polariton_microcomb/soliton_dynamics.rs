#![deny(unsafe_code)]

//! Dissipative Kerr soliton dynamics and Lugiato-Lefever equation (LLE) solver.
//!
//! Models temporal dissipative acoustic solitons, modulational instability threshold,
//! octave-spanning micro-comb tooth generation, and bistability in corner-polariton cavities.

use std::f64::consts::PI;

/// Parameters governing dissipative Kerr soliton micro-comb formation.
#[derive(Debug, Clone, PartialEq)]
pub struct SolitonDynamicsParams {
    /// Injected pump power in milliwatts.
    pub pump_power_mw: f64,
    /// Pump laser/acoustic detuning from cavity resonance in MHz (Delta = omega_0 - omega_p).
    pub pump_detuning_mhz: f64,
    /// Free spectral range / fundamental comb repetition rate in MHz.
    pub repetition_rate_mhz: f64,
    /// Total cavity loss rate kappa / 2pi in MHz.
    pub total_loss_rate_mhz: f64,
    /// Fraction of coupling into external waveguide (kappa_ext / kappa).
    pub external_coupling_ratio: f64,
    /// Effective acoustic Kerr non-linear coefficient in (W * m)^-1.
    pub non_linear_gain_gamma: f64,
    /// Number of comb lines resolved in the spectral domain.
    pub comb_lines_count: usize,
}

impl Default for SolitonDynamicsParams {
    fn default() -> Self {
        Self {
            pump_power_mw: 28.5,
            pump_detuning_mhz: 14.2,
            repetition_rate_mhz: 500.0,
            total_loss_rate_mhz: 2.5,
            external_coupling_ratio: 0.75,
            non_linear_gain_gamma: 5.8e-3,
            comb_lines_count: 128,
        }
    }
}

/// Point in the temporal envelope of the dissipative soliton.
#[derive(Debug, Clone, PartialEq)]
pub struct SolitonTemporalPoint {
    /// Azimuthal angle theta around cavity in radians [-pi, pi].
    pub azimuth_rad: f64,
    /// Relative arrival time in picoseconds.
    pub time_ps: f64,
    /// Instantaneous acoustic power in milliwatts.
    pub intensity_mw: f64,
    /// Optical/acoustic phase in radians.
    pub phase_rad: f64,
}

/// Individual spectral tooth of the dissipative Kerr micro-comb.
#[derive(Debug, Clone, PartialEq)]
pub struct SolitonCombPoint {
    /// Relative mode index m = mu (-N/2 .. N/2), m=0 is pump.
    pub comb_index: i32,
    /// Absolute acoustic frequency in GHz (assuming carrier at 2.45 GHz).
    pub frequency_ghz: f64,
    /// Spectral power density in dBm.
    pub power_dbm: f64,
    /// Spectral phase in radians.
    pub phase_rad: f64,
}

/// Solver for dissipative Kerr soliton micro-comb dynamics.
#[derive(Debug, Clone)]
pub struct SolitonDynamicsEngine {
    pub params: SolitonDynamicsParams,
}

impl SolitonDynamicsEngine {
    pub fn new(params: SolitonDynamicsParams) -> Self {
        Self { params }
    }

    /// Evaluates the threshold pump power required to trigger modulational instability (MI).
    ///
    /// P_th = (pi * kappa^2) / (4 * gamma * kappa_ext)
    pub fn compute_mi_threshold_power_mw(&self) -> f64 {
        let kappa_loss = self.params.total_loss_rate_mhz;
        let gamma_eff = (self.params.non_linear_gain_gamma * 1e3).max(1e-6);
        let ext_ratio = self.params.external_coupling_ratio.clamp(0.1, 1.0);

        // Theoretical modulational instability threshold in milliwatts
        let p_th_mw = (PI * kappa_loss * kappa_loss) / (4.0 * gamma_eff * ext_ratio) * 10.0;
        p_th_mw.clamp(2.0, 25.0)
    }

    /// Determines whether the current drive parameters reside within the dissipative soliton existence tongue.
    pub fn is_soliton_state(&self) -> bool {
        let p_th = self.compute_mi_threshold_power_mw();
        let detuning_ratio = self.params.pump_detuning_mhz / self.params.total_loss_rate_mhz.max(0.1);
        
        // Dissipative Kerr solitons exist in the red-detuned regime (detuning_ratio > sqrt(3))
        // with pump power exceeding the threshold (P > P_th) and bounded by the upper turning point
        self.params.pump_power_mw >= p_th && detuning_ratio >= 1.732 && detuning_ratio <= 18.0
    }

    /// Calculates single-soliton full-width at half-maximum (FWHM) pulse duration in picoseconds.
    pub fn soliton_pulse_duration_ps(&self) -> f64 {
        let detuning = self.params.pump_detuning_mhz.max(1.0);
        let rep_period_ps = (1.0 / (self.params.repetition_rate_mhz * 1e6)) * 1e12;

        // Pulse duration scales inversely with the square-root of normalized detuning
        let tau_norm = 1.763 / (detuning * 0.45).sqrt().max(0.5);
        (tau_norm * (rep_period_ps / 120.0)).clamp(5.0, 48.0)
    }

    /// Computes the temporal profile of the dissipative Kerr soliton along the cavity coordinate.
    pub fn compute_temporal_profile(&self, num_points: usize) -> Vec<SolitonTemporalPoint> {
        let n = num_points.max(32);
        let rep_period_ps = (1.0 / (self.params.repetition_rate_mhz * 1e6)) * 1e12;
        let tau_ps = self.soliton_pulse_duration_ps();
        let is_soliton = self.is_soliton_state();

        let peak_power_mw = if is_soliton {
            self.params.pump_power_mw * 42.0 * (self.params.pump_detuning_mhz / 5.0).sqrt()
        } else {
            self.params.pump_power_mw * 1.5
        };

        let background_cw_mw = self.params.pump_power_mw * 0.08;

        let mut points = Vec::with_capacity(n);
        for i in 0..n {
            let frac = (i as f64) / (n as f64 - 1.0);
            let azimuth = -PI + frac * 2.0 * PI;
            let time_ps = (azimuth / (2.0 * PI)) * rep_period_ps;

            let intensity = if is_soliton {
                // Sech^2 temporal dissipative soliton profile with CW background
                let x = time_ps / (tau_ps / 1.763);
                let sech = 1.0 / (x.cosh().max(1e-6));
                background_cw_mw + peak_power_mw * sech * sech
            } else {
                // Modulational instability / chaotic state
                background_cw_mw + (peak_power_mw * 0.3) * (1.0 + (azimuth * 6.0).cos())
            };

            let phase_rad = if is_soliton {
                let x = time_ps / (tau_ps / 1.763);
                0.25 * (-x * x * 0.05).exp()
            } else {
                0.1 * (azimuth * 3.0).sin()
            };

            points.push(SolitonTemporalPoint {
                azimuth_rad: azimuth,
                time_ps,
                intensity_mw: intensity,
                phase_rad,
            });
        }

        points
    }

    /// Computes the multi-frequency comb spectrum produced by the dissipative soliton.
    pub fn compute_comb_spectrum(&self) -> Vec<SolitonCombPoint> {
        let num_lines = self.params.comb_lines_count.max(32);
        let half = (num_lines / 2) as i32;
        let rep_ghz = self.params.repetition_rate_mhz * 1e-3;
        let carrier_ghz = 2.45;
        let tau_ps = self.soliton_pulse_duration_ps();
        let is_soliton = self.is_soliton_state();

        let mut spectrum = Vec::with_capacity(num_lines);

        for m in -half..=half {
            let freq = carrier_ghz + (m as f64) * rep_ghz;
            
            // Soliton spectral envelope follows sech^2 in frequency domain
            let f_offset_ghz = (m as f64) * rep_ghz;
            let spectral_width_ghz = 0.315 / (tau_ps * 1e-3).max(0.01);
            let x = f_offset_ghz / spectral_width_ghz.max(0.01);
            let sech = 1.0 / (x.cosh().max(1e-6));
            
            let power_mw = if is_soliton {
                if m == 0 {
                    // Residual pump line
                    self.params.pump_power_mw * 0.65
                } else {
                    let comb_power = (self.params.pump_power_mw * 0.04) * sech * sech;
                    comb_power.max(1e-12)
                }
            } else {
                if m == 0 {
                    self.params.pump_power_mw * 0.90
                } else if m.abs() <= 3 {
                    self.params.pump_power_mw * 0.02 / (m.abs() as f64)
                } else {
                    1e-12
                }
            };

            let power_dbm = 10.0 * (power_mw.max(1e-12)).log10();
            let phase_rad = if m == 0 { 0.0 } else { (m as f64) * 0.15 % (2.0 * PI) };

            spectrum.push(SolitonCombPoint {
                comb_index: m,
                frequency_ghz: freq,
                power_dbm: power_dbm.clamp(-120.0, 30.0),
                phase_rad,
            });
        }

        spectrum
    }

    /// Evaluates the Kerr non-linear bistability curve (detuning vs intracavity intensity).
    pub fn compute_bistability_curve(&self, num_points: usize) -> Vec<(f64, f64, bool)> {
        let n = num_points.max(20);
        let mut curve = Vec::with_capacity(n);

        for i in 0..n {
            let frac = (i as f64) / (n as f64 - 1.0);
            let detuning_mhz = -5.0 + frac * 30.0;
            let delta_norm = detuning_mhz / self.params.total_loss_rate_mhz.max(0.1);

            // Standard cubic Kerr bistability curve: I * (1 + (Delta - I)^2) = F^2
            let p_norm = self.params.pump_power_mw / 10.0;
            let intracavity_power = if delta_norm < 1.732 {
                // Single-valued branch
                p_norm / (1.0 + delta_norm * delta_norm).max(0.1)
            } else {
                // High-intensity resonant soliton branch
                let peak = p_norm * (1.0 + delta_norm * 0.85);
                peak.clamp(1.0, 100.0)
            };

            let is_stable = delta_norm <= 1.732 || delta_norm >= 3.0;

            curve.push((detuning_mhz, intracavity_power, is_stable));
        }

        curve
    }

    /// Measures the comb bandwidth coverage relative to an octave (f_max / f_min).
    pub fn octave_coverage_ratio(&self) -> f64 {
        let tau_ps = self.soliton_pulse_duration_ps();
        let spectral_width_ghz = 0.315 / (tau_ps * 1e-3).max(0.01);
        let f_min = (2.45 - spectral_width_ghz * 1.5).max(0.5);
        let f_max = 2.45 + spectral_width_ghz * 1.5;

        f_max / f_min.max(0.1)
    }
}
