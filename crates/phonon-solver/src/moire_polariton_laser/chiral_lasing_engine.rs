#![deny(unsafe_code)]

//! Moiré Exciton-Polariton Non-Reciprocal Chiral Lasing & BEC Condensation Engine.
//!
//! Models driven-dissipative exciton-polariton Bose-Einstein condensation (BEC) and lasing
//! in topological valley Hall edge modes. Evaluates non-linear S-curve input-output characteristics,
//! Schawlow-Townes linewidth narrowing across threshold, extended first-order temporal coherence g^(1)(tau),
//! non-linear blue-shifts, and high front-to-back directional emission purity.

use std::f64::consts::PI;

/// Configuration parameters for chiral polariton lasing.
#[derive(Debug, Clone)]
pub struct ChiralLasingParams {
    /// Optical pump excitation power density in uW / um^2.
    pub pump_power_uw_um2: f64,
    /// Condensation / lasing threshold power density P_th in uW / um^2.
    pub threshold_power_uw_um2: f64,
    /// Polariton lifetime tau_pol in picoseconds (default ~22.0 ps).
    pub polariton_lifetime_ps: f64,
    /// Non-radiative reservoir decay rate gamma_R in GHz.
    pub reservoir_decay_rate_ghz: f64,
    /// Polariton-polariton contact interaction strength g_pol in ueV * um^2.
    pub polariton_interaction_uev_um2: f64,
    /// Spontaneous emission coupling factor beta into the condensate mode.
    pub spontaneous_coupling_factor_beta: f64,
    /// Chirality excitation bias in [-1.0, 1.0] (+1.0 for purely forward/sigma+, -1.0 for backward/sigma-).
    pub pump_chirality_factor: f64,
}

impl Default for ChiralLasingParams {
    fn default() -> Self {
        Self {
            pump_power_uw_um2: 4.5,
            threshold_power_uw_um2: 2.2,
            polariton_lifetime_ps: 22.0,
            reservoir_decay_rate_ghz: 1.2,
            polariton_interaction_uev_um2: 12.0,
            spontaneous_coupling_factor_beta: 0.08,
            pump_chirality_factor: 0.95,
        }
    }
}

/// Evaluated metrics for the chiral polariton laser.
#[derive(Debug, Clone)]
pub struct ChiralLasingMetrics {
    /// Whether the active pump exceeds the condensation threshold (P > P_th).
    pub is_above_threshold: bool,
    /// Polariton condensate occupancy density n_pol in um^-2.
    pub polariton_density_um2: f64,
    /// Laser emission spectral linewidth in kHz.
    pub emission_linewidth_khz: f64,
    /// First-order temporal coherence time tau_c in picoseconds.
    pub temporal_coherence_time_ps: f64,
    /// Directional chiral emission front-to-back ratio in dB.
    pub chiral_front_to_back_ratio_db: f64,
    /// Non-linear interaction-induced energy blueshift in meV.
    pub polariton_blueshift_mev: f64,
    /// Second-order intensity correlation at zero delay g^(2)(0).
    pub second_order_coherence_g2_zero: f64,
}

/// S-curve input-output data point.
#[derive(Debug, Clone)]
pub struct InputOutputCurvePoint {
    /// Excitation pump power density in uW / um^2.
    pub pump_power_uw_um2: f64,
    /// Emitted coherent optical intensity in arbitrary units.
    pub emission_intensity_arb: f64,
    /// Emission spectral linewidth in kHz.
    pub linewidth_khz: f64,
    /// Energy blueshift in meV.
    pub blueshift_mev: f64,
}

/// First-order temporal coherence data point.
#[derive(Debug, Clone)]
pub struct TemporalCoherencePoint {
    /// Time delay tau in picoseconds.
    pub delay_time_ps: f64,
    /// First-order temporal coherence |g^(1)(tau)| in [0.0, 1.0].
    pub first_order_coherence_g1: f64,
}

/// Solver engine for chiral exciton-polariton lasing.
#[derive(Debug, Clone)]
pub struct ChiralLasingSolver {
    params: ChiralLasingParams,
}

impl ChiralLasingSolver {
    /// Constructs a new chiral lasing solver.
    pub fn new(params: ChiralLasingParams) -> Self {
        Self { params }
    }

    /// Returns a reference to the active parameters.
    pub fn params(&self) -> &ChiralLasingParams {
        &self.params
    }

    /// Evaluates steady-state polariton condensate density for a given pump power.
    pub fn compute_density(&self, pump: f64) -> f64 {
        let p_th = self.params.threshold_power_uw_um2;
        let beta = self.params.spontaneous_coupling_factor_beta;

        if pump <= p_th {
            // Below threshold: linear thermal/spontaneous polariton gas
            beta * (pump / p_th) * 8.0
        } else {
            // Above threshold: macroscopic BEC occupancy
            let excess = pump - p_th;
            let slope = 65.0; // condensate buildup slope
            (beta * 8.0) + (excess * slope)
        }
    }

    /// Evaluates comprehensive chiral lasing metrics.
    pub fn evaluate_metrics(&self) -> ChiralLasingMetrics {
        let pump = self.params.pump_power_uw_um2;
        let p_th = self.params.threshold_power_uw_um2;
        let is_above = pump > p_th;

        let density = self.compute_density(pump);

        // Linewidth narrowing: Schawlow-Townes formula
        // Sub-threshold ~3.2 MHz (3200 kHz); above threshold narrows to < 50 kHz
        let baseline_linewidth_khz = 3200.0;
        let linewidth_khz = if is_above {
            let narrowing_factor = 1.0 + (density / 2.0);
            (baseline_linewidth_khz / narrowing_factor).max(18.0)
        } else {
            let frac = (pump / p_th).clamp(0.0, 1.0);
            baseline_linewidth_khz * (1.0 - 0.25 * frac)
        };

        // Temporal coherence time tau_c approx 1 / (pi * Delta nu)
        let delta_nu_hz = linewidth_khz * 1.0e3;
        let tau_c_s = 1.0 / (PI * delta_nu_hz);
        let tau_c_ps = (tau_c_s * 1.0e12).clamp(2.5, 350.0);

        // Non-linear interaction blueshift Delta E = g_pol * n_pol
        // g_pol in ueV * um^2, density in um^-2 -> Delta E in meV
        let blueshift_mev = (self.params.polariton_interaction_uev_um2 * density) * 1.0e-3;

        // Front-to-back chiral emission ratio in dB
        // Above threshold, non-linear modal gain competition strongly amplifies the dominant chiral mode
        let chi = self.params.pump_chirality_factor.abs().clamp(0.0, 0.999);
        let linear_ratio = (1.0 + chi) / (1.0 - chi).max(1.0e-4);
        let gain_exp = if is_above { 1.8 } else { 1.0 };
        let fbr_db = (10.0 * linear_ratio.log10()) * gain_exp;

        // Second-order coherence g^(2)(0):
        // 2.0 for thermal/spontaneous regime below threshold, drops to 1.0 in coherent BEC
        let g2_zero = if is_above {
            1.0 + (0.95 / (1.0 + (density / 10.0)))
        } else {
            1.95 - (0.15 * (pump / p_th))
        };

        ChiralLasingMetrics {
            is_above_threshold: is_above,
            polariton_density_um2: density,
            emission_linewidth_khz: linewidth_khz,
            temporal_coherence_time_ps: tau_c_ps,
            chiral_front_to_back_ratio_db: fbr_db,
            polariton_blueshift_mev: blueshift_mev,
            second_order_coherence_g2_zero: g2_zero,
        }
    }

    /// Computes S-curve input-output characteristics across a range of pump powers.
    pub fn compute_input_output_curve(&self, points: usize) -> Vec<InputOutputCurvePoint> {
        let pts = points.max(16);
        let p_max = self.params.threshold_power_uw_um2 * 3.5;
        let mut results = Vec::with_capacity(pts);

        for i in 0..pts {
            let frac = (i as f64) / ((pts - 1) as f64);
            let pump = frac * p_max;

            let density = self.compute_density(pump);
            let intensity = density * 12.5;

            let p_th = self.params.threshold_power_uw_um2;
            let linewidth = if pump > p_th {
                3200.0 / (1.0 + density / 2.0).max(1.0)
            } else {
                3200.0 * (1.0 - 0.25 * (pump / p_th))
            };

            let blueshift = (self.params.polariton_interaction_uev_um2 * density) * 1.0e-3;

            results.push(InputOutputCurvePoint {
                pump_power_uw_um2: pump,
                emission_intensity_arb: intensity,
                linewidth_khz: linewidth,
                blueshift_mev: blueshift,
            });
        }

        results
    }

    /// Computes first-order temporal coherence |g^(1)(tau)| decaying over delay time tau.
    pub fn compute_temporal_coherence(&self, points: usize) -> Vec<TemporalCoherencePoint> {
        let pts = points.max(16);
        let tau_c = self.evaluate_metrics().temporal_coherence_time_ps;
        let tau_max = tau_c * 3.0;
        let mut results = Vec::with_capacity(pts);

        for i in 0..pts {
            let frac = (i as f64) / ((pts - 1) as f64);
            let tau = frac * tau_max;
            let g1 = (-tau / tau_c).exp();

            results.push(TemporalCoherencePoint {
                delay_time_ps: tau,
                first_order_coherence_g1: g1,
            });
        }

        results
    }

    /// Computes angular chiral emission distribution in the plane.
    /// Returns (theta_deg, normalized_intensity) in [0.0, 360.0].
    pub fn compute_chiral_angular_profile(&self, points: usize) -> Vec<(f64, f64)> {
        let pts = points.max(24);
        let chi = self.params.pump_chirality_factor;
        let mut results = Vec::with_capacity(pts);

        for i in 0..pts {
            let frac = (i as f64) / (pts as f64);
            let theta_deg = frac * 360.0;
            let theta_rad = theta_deg.to_radians();

            // Cardioid profile tilted by chirality
            let val = (1.0 + chi * theta_rad.cos()).max(0.01).powi(3);

            results.push((theta_deg, val));
        }

        results
    }
}
