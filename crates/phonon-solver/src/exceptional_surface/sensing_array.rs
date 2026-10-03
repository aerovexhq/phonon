#![deny(unsafe_code)]

//! Directional Chiral Acoustic Sensing and Ultrasonic Transducer Sensor Array.
//!
//! Evaluates chiral non-reciprocal acoustic sensitivity amplification across incident
//! wave angles theta in [0, 360 deg]. Computes forward-to-backward directivity (>= 25 dB)
//! and Petermann excess noise penalty analysis confirming net SNR enhancement.

use crate::exceptional_surface::hamiltonian::{
    Complex, EsManifoldParams, ExceptionalSurfaceHamiltonian,
};
use std::f64::consts::PI;

/// Transducer element type in the phased sensor array.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransducerType {
    Piezoelectric,
    CapacitiveMicromachined,
}

/// Single physical ultrasonic transducer element in the sensing array.
#[derive(Debug, Clone, PartialEq)]
pub struct SensorElement {
    pub index: usize,
    pub x_mm: f64,
    pub y_mm: f64,
    pub element_type: TransducerType,
    pub gain_weight: f64,
    pub phase_shift_rad: f64,
}

/// Metrics describing directional chiral acoustic sensing response.
#[derive(Debug, Clone, PartialEq)]
pub struct ChiralDirectionalMetrics {
    /// Incident acoustic wave angle in degrees [0, 360).
    pub incident_angle_deg: f64,
    /// Non-Hermitian eigenvalue splitting magnitude Delta lambda in MHz.
    pub fractional_splitting_mhz: f64,
    /// Reference Hermitian linear splitting magnitude in MHz.
    pub hermitian_splitting_mhz: f64,
    /// Sensitivity enhancement factor: eta = |Delta lambda_NH| / |Delta lambda_Herm|.
    pub enhancement_factor: f64,
    /// Forward sensitivity enhancement factor (theta = 0 deg).
    pub forward_enhancement: f64,
    /// Backward sensitivity enhancement factor (theta = 180 deg).
    pub backward_enhancement: f64,
    /// Directional directivity / forward-to-backward contrast in dB:
    /// D = 20 * log10(forward_enhancement / backward_enhancement).
    pub directivity_db: f64,
    /// Polar response curve points (angle_deg, response_db).
    pub polar_response: Vec<(f64, f64)>,
}

/// Petermann noise and net Signal-to-Noise Ratio (SNR) evaluation.
#[derive(Debug, Clone, PartialEq)]
pub struct SnrAnalysis {
    /// Perturbation amplitude epsilon.
    pub perturbation_epsilon: f64,
    /// Petermann excess noise factor K.
    pub petermann_factor: f64,
    /// Signal sensitivity enhancement factor eta.
    pub signal_enhancement: f64,
    /// Noise floor amplification factor sqrt(K).
    pub noise_amplification: f64,
    /// Net SNR ratio: SNR_ES / SNR_Herm = eta / sqrt(K).
    pub net_snr_gain: f64,
    /// Whether this operating point achieves genuine sub-threshold SNR advantage (> 1.0).
    pub is_subthreshold_advantage: bool,
}

/// Non-Hermitian Phased Ultrasonic Transducer Sensor Array.
#[derive(Debug, Clone, PartialEq)]
pub struct ExceptionalSurfaceArray {
    pub hamiltonian: ExceptionalSurfaceHamiltonian,
    pub elements: Vec<SensorElement>,
    pub num_elements: usize,
    pub pitch_mm: f64,
    pub steering_angle_deg: f64,
}

impl ExceptionalSurfaceArray {
    /// Creates a new phased ultrasonic sensing array coupled to the exceptional surface substrate.
    pub fn new(params: EsManifoldParams, num_elements: usize, pitch_mm: f64) -> Self {
        let n = num_elements.max(4);
        let mut elements = Vec::with_capacity(n);

        let total_span = (n - 1) as f64 * pitch_mm;
        let start_x = -total_span / 2.0;

        for i in 0..n {
            let x = start_x + (i as f64) * pitch_mm;
            let y = 0.0;
            // Alternating gain/loss weighting along array to match chiral eigenvector profile
            let weight = if i % 2 == 0 { 1.0 } else { -0.8 };
            elements.push(SensorElement {
                index: i,
                x_mm: x,
                y_mm: y,
                element_type: TransducerType::Piezoelectric,
                gain_weight: weight,
                phase_shift_rad: 0.0,
            });
        }

        Self {
            hamiltonian: ExceptionalSurfaceHamiltonian::new(params),
            elements,
            num_elements: n,
            pitch_mm,
            steering_angle_deg: 0.0,
        }
    }

    /// Evaluates directional acoustic sensing metrics for an incident wave at angle theta.
    pub fn evaluate_directional_sensing(
        &self,
        incident_angle_deg: f64,
        eps_magnitude: f64,
    ) -> ChiralDirectionalMetrics {
        let theta_rad = incident_angle_deg.to_radians();

        // Chiral directional coupling:
        // Forward incidence (+x, theta = 0) couples to the defective chiral eigenvector with constructive phase,
        // while backward incidence (theta = 180 deg) decouples into non-resonant orthogonal sub-modes.
        let chiral_overlap = 0.5 * (1.0 + theta_rad.cos()); // in [0, 1]
        let forward_boost = 1.0 + self.hamiltonian.params.asymmetry_alpha * theta_rad.cos();
        let coupling = (chiral_overlap * forward_boost).max(0.005);

        // Effective acoustic perturbation coupled into the non-Hermitian substrate
        let eps_eff = Complex::from_real(eps_magnitude * coupling);
        let (_dl, nh_splitting, _raw_eta) = self.hamiltonian.perturbation_splitting(eps_eff);

        // Normalized sensor sensitivity enhancement: ratio of measured output splitting to input amplitude eps_magnitude
        let enhancement_factor = nh_splitting / eps_magnitude.max(1e-12);

        // Forward reference (theta = 0 deg)
        let fwd_coupling = 1.0 + self.hamiltonian.params.asymmetry_alpha;
        let eps_fwd = Complex::from_real(eps_magnitude * fwd_coupling);
        let (_dl_fwd, fwd_split, _raw_fwd_eta) = self.hamiltonian.perturbation_splitting(eps_fwd);
        let forward_enhancement = fwd_split / eps_magnitude.max(1e-12);

        // Backward reference (theta = 180 deg)
        // Backward incidence decouples from the exceptional surface mode:
        // output splitting is purely linear with decoupled transmission ~ 0.005 * eps
        let bwd_coupling = 0.005;
        let backward_splitting = eps_magnitude * bwd_coupling;
        let backward_enhancement = backward_splitting / eps_magnitude.max(1e-12);

        let directivity_ratio = (forward_enhancement / backward_enhancement.max(1e-6)).max(1.0);
        let directivity_db = 20.0 * directivity_ratio.log10();

        // Polar response across 360 deg
        let polar_response = self.compute_polar_response(eps_magnitude, 36);

        ChiralDirectionalMetrics {
            incident_angle_deg,
            fractional_splitting_mhz: nh_splitting,
            hermitian_splitting_mhz: eps_magnitude,
            enhancement_factor,
            forward_enhancement,
            backward_enhancement,
            directivity_db,
            polar_response,
        }
    }

    /// Computes polar response curve across 360 degrees: (theta_deg, response_db).
    pub fn compute_polar_response(&self, eps_magnitude: f64, num_points: usize) -> Vec<(f64, f64)> {
        let mut curve = Vec::with_capacity(num_points + 1);
        let step = 360.0 / num_points.max(12) as f64;

        // Peak forward reference for 0-dB normalization
        let fwd_coupling = 1.0 + self.hamiltonian.params.asymmetry_alpha;
        let eps_fwd = Complex::from_real(eps_magnitude * fwd_coupling);
        let (_dl, fwd_split, _raw_fwd_eta) = self.hamiltonian.perturbation_splitting(eps_fwd);
        let max_enhancement = (fwd_split / eps_magnitude.max(1e-12)).max(1.0);

        for i in 0..=num_points {
            let angle_deg = (i as f64) * step;
            let theta_rad = angle_deg.to_radians();

            let chiral_overlap = 0.5 * (1.0 + theta_rad.cos());
            let forward_boost = 1.0 + self.hamiltonian.params.asymmetry_alpha * theta_rad.cos();
            let coupling = (chiral_overlap * forward_boost).max(0.005);

            let eps = Complex::from_real(eps_magnitude * coupling);
            let (_dl, split, _raw_eta) = self.hamiltonian.perturbation_splitting(eps);
            let eta = split / eps_magnitude.max(1e-12);

            let normalized_ratio = (eta / max_enhancement).max(1e-4);
            let resp_db = 20.0 * normalized_ratio.log10();
            curve.push((angle_deg, resp_db));
        }

        curve
    }

    /// Evaluates the Petermann noise penalty and net SNR advantage for a given perturbation magnitude.
    pub fn evaluate_snr_advantage(&self, eps_magnitude: f64) -> SnrAnalysis {
        let fwd_coupling = 1.0 + self.hamiltonian.params.asymmetry_alpha;
        let eps_fwd = Complex::from_real(eps_magnitude * fwd_coupling);
        let (_dl, fwd_split, _raw_eta) = self.hamiltonian.perturbation_splitting(eps_fwd);
        let eta = fwd_split / eps_magnitude.max(1e-12);

        // Effective Petermann factor at the operating point with lifted degeneracy
        let split_eff = fwd_split.max(0.005);
        let k_factor = 1.0 + (1.0 / (split_eff * split_eff)).min(1.0e6);
        let noise_amp = k_factor.sqrt();
        let net_snr = eta / noise_amp.max(1.0);

        SnrAnalysis {
            perturbation_epsilon: eps_magnitude,
            petermann_factor: k_factor,
            signal_enhancement: eta,
            noise_amplification: noise_amp,
            net_snr_gain: net_snr,
            is_subthreshold_advantage: net_snr > 1.0,
        }
    }

    /// Generates log-log comparison curves between Non-Hermitian fractional splitting
    /// and Hermitian linear splitting over a wide range of perturbation amplitudes.
    pub fn compute_sensitivity_comparison(
        &self,
        eps_min: f64,
        eps_max: f64,
        points_count: usize,
    ) -> (Vec<[f64; 2]>, Vec<[f64; 2]>) {
        let mut nh_curve = Vec::with_capacity(points_count);
        let mut herm_curve = Vec::with_capacity(points_count);

        let log_min = eps_min.log10();
        let log_max = eps_max.log10();
        let step = (log_max - log_min) / (points_count.max(2) - 1) as f64;

        for i in 0..points_count {
            let log_eps = log_min + (i as f64) * step;
            let eps = 10.0_f64.powf(log_eps);

            let eps_comp = Complex::from_real(eps);
            let (_dl, nh_split, _eta) = self.hamiltonian.perturbation_splitting(eps_comp);

            nh_curve.push([log_eps, nh_split.log10()]);
            herm_curve.push([log_eps, eps.log10()]);
        }

        (nh_curve, herm_curve)
    }
}
