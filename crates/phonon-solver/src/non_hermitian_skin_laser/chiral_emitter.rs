#![deny(unsafe_code)]

//! Chiral Directional Acoustic Edge Emitter & Radiation Antenna.
//!
//! Models unidirectional acoustic emission from topological boundary channels,
//! high front-to-back directivity, far-field beam steering, and topological defect immunity.

use std::f64::consts::PI;

/// Parameters for the chiral edge acoustic emitter.
#[derive(Debug, Clone, PartialEq)]
pub struct ChiralEmitterParams {
    /// Coupling efficiency from laser cavity to boundary emitter (0.0 to 1.0).
    pub coupling_efficiency: f64,
    /// Target front-to-back directivity ratio (dB).
    pub directivity_db: f64,
    /// Number of phased antenna elements along the edge.
    pub antenna_elements: usize,
    /// Antenna element spacing in mm.
    pub element_spacing_mm: f64,
    /// Main beam steering angle in degrees.
    pub beam_angle_deg: f64,
    /// Whether an acoustic obstacle/defect is inserted in the boundary channel.
    pub defect_present: bool,
    /// Attenuation induced by the defect (dB).
    pub defect_loss_db: f64,
}

impl Default for ChiralEmitterParams {
    fn default() -> Self {
        Self {
            coupling_efficiency: 0.88,
            directivity_db: 28.5,
            antenna_elements: 8,
            element_spacing_mm: 2.5,
            beam_angle_deg: 0.0,
            defect_present: false,
            defect_loss_db: 0.15,
        }
    }
}

/// A sample point in the far-field polar radiation pattern.
#[derive(Debug, Clone, PartialEq)]
pub struct RadiationPatternPoint {
    /// Azimuth angle in degrees [-180, 180].
    pub angle_deg: f64,
    /// Normalized power in dB (relative to peak).
    pub power_db: f64,
    /// Linear normalized directivity amplitude.
    pub directivity_lin: f64,
}

/// Evaluated metrics for the chiral acoustic emitter.
#[derive(Debug, Clone, PartialEq)]
pub struct ChiralEmitterMetrics {
    /// Forward emitted acoustic power (mW).
    pub forward_power_mw: f64,
    /// Backward leaked acoustic power (mW).
    pub backward_power_mw: f64,
    /// Front-to-back directivity ratio (dB).
    pub front_to_back_directivity_db: f64,
    /// Half-Power Beam Width (HPBW) in degrees.
    pub hpbw_deg: f64,
    /// Defect transmission ratio T_defect / T_clean (0.0 to 1.0).
    pub defect_transmission_ratio: f64,
    /// Boundary antenna radiation efficiency (0.0 to 1.0).
    pub antenna_efficiency: f64,
}

/// Solver for the chiral directional acoustic emitter.
#[derive(Debug, Clone)]
pub struct ChiralEmitterSolver {
    pub params: ChiralEmitterParams,
}

impl ChiralEmitterSolver {
    /// Creates a new solver with specified parameters.
    pub fn new(params: ChiralEmitterParams) -> Self {
        Self { params }
    }

    /// Evaluates operational metrics given an input laser power.
    pub fn evaluate_metrics(&self, input_laser_power_mw: f64) -> ChiralEmitterMetrics {
        let p = &self.params;
        let base_power = input_laser_power_mw.max(0.1);

        // Transmission penalty from boundary defect
        let defect_transmission_ratio = if p.defect_present {
            10.0f64.powf(-p.defect_loss_db / 10.0)
        } else {
            1.0
        };

        let forward_power_mw = base_power * p.coupling_efficiency * defect_transmission_ratio;
        let front_to_back_directivity_db = p.directivity_db.max(10.0);
        let backward_power_mw = forward_power_mw * 10.0f64.powf(-front_to_back_directivity_db / 10.0);

        // Half-Power Beam Width for an N-element linear array with d = lambda / 2:
        // HPBW approx 50.8 / (N * d / lambda) = 101.6 / N (degrees)
        let n = p.antenna_elements.max(2) as f64;
        let hpbw_deg = (132.0 / n).clamp(10.0, 45.0); // ~16.5 deg for N=8

        let antenna_efficiency = p.coupling_efficiency * defect_transmission_ratio;

        ChiralEmitterMetrics {
            forward_power_mw,
            backward_power_mw,
            front_to_back_directivity_db,
            hpbw_deg,
            defect_transmission_ratio,
            antenna_efficiency,
        }
    }

    /// Computes the complete far-field angular radiation pattern.
    pub fn compute_radiation_pattern(&self, point_count: usize) -> Vec<RadiationPatternPoint> {
        let p = &self.params;
        let count = point_count.max(36);
        let mut pattern = Vec::with_capacity(count);

        let n = p.antenna_elements.max(2);
        let beam_rad = p.beam_angle_deg.to_radians();
        let directivity_ratio = 10.0f64.powf(-p.directivity_db / 10.0);

        for i in 0..count {
            let theta_deg = -180.0 + (i as f64 / count as f64) * 360.0;
            let theta_rad = theta_deg.to_radians();

            // Phased array factor: AF = sin(N * psi / 2) / (N * sin(psi / 2))
            // where psi = pi * (sin(theta) - sin(beam_angle))
            let psi = PI * (theta_rad.sin() - beam_rad.sin());
            let array_factor = if psi.abs() < 1e-4 {
                1.0
            } else {
                let denom = (n as f64) * (0.5 * psi).sin();
                if denom.abs() < 1e-6 {
                    1.0
                } else {
                    ((0.5 * (n as f64) * psi).sin() / denom).abs()
                }
            };

            // Element cardioid factor enforcing front-to-back unidirectionality
            let is_forward = theta_deg.abs() <= 90.0;
            let element_factor = if is_forward {
                (0.5 * (1.0 + theta_rad.cos())).max(0.0)
            } else {
                directivity_ratio.sqrt() * (0.5 * (1.0 + theta_rad.cos())).max(0.01)
            };

            let total_amplitude = (array_factor * element_factor).clamp(1e-4, 1.0);
            let power_db = (20.0 * total_amplitude.log10()).max(-45.0);

            pattern.push(RadiationPatternPoint {
                angle_deg: theta_deg,
                power_db,
                directivity_lin: total_amplitude,
            });
        }

        pattern
    }
}
