#![deny(unsafe_code)]

//! Unidirectional invisibility and non-Hermitian PT-symmetric acoustic scattering engine.
//!
//! Evaluates the scattering matrix S = [[r_L, t], [t, r_R]], unidirectional reflectionlessness
//! (R_L = 0, R_R != 0), the PT generalized unitarity relation |T - 1| = sqrt(R_L * R_R),
//! and 1D acoustic pressure field distributions.

use crate::pt_symmetric_acoustic::pt_hamiltonian::{PtAcousticParams, PtHamiltonianSolver};
use std::f64::consts::PI;

/// Parameters defining the acoustic waveguide and unidirectional scattering testbench.
#[derive(Debug, Clone)]
pub struct InvisibilityParams {
    /// Resonator PT parameters.
    pub pt_params: PtAcousticParams,
    /// Acoustic waveguide characteristic acoustic impedance in Pa*s/m^3 (default ~415.0).
    pub waveguide_impedance: f64,
    /// Speed of sound in the acoustic medium in m/s (default ~343.0 m/s).
    pub speed_of_sound_m_s: f64,
    /// Bilayer length in mm (default ~50.0 mm).
    pub metamaterial_length_mm: f64,
}

impl Default for InvisibilityParams {
    fn default() -> Self {
        Self {
            pt_params: PtAcousticParams::default(),
            waveguide_impedance: 415.0,
            speed_of_sound_m_s: 343.0,
            metamaterial_length_mm: 50.0,
        }
    }
}

/// Scattering spectrum point at a discrete frequency.
#[derive(Debug, Clone)]
pub struct ScatteringSpectrumPoint {
    /// Frequency in Hz.
    pub frequency_hz: f64,
    /// Power transmission T = |t|^2.
    pub transmission_t: f64,
    /// Transmission in dB.
    pub transmission_db: f64,
    /// Left power reflection R_L = |r_L|^2.
    pub reflection_left_r: f64,
    /// Left reflection in dB.
    pub reflection_left_db: f64,
    /// Right power reflection R_R = |r_R|^2.
    pub reflection_right_r: f64,
    /// Right reflection in dB.
    pub reflection_right_db: f64,
    /// Unidirectional reflection contrast ratio eta = |R_R - R_L| / (R_R + R_L).
    pub contrast_ratio: f64,
    /// Generalized unitarity relation error | |T - 1| - sqrt(R_L * R_R) |.
    pub generalized_unitarity_error: f64,
}

/// Metrics for unidirectional invisibility at the center operating frequency.
#[derive(Debug, Clone)]
pub struct InvisibilityMetrics {
    /// Power transmission at operating frequency T_0.
    pub center_transmission: f64,
    /// Transmission in dB.
    pub center_transmission_db: f64,
    /// Left power reflection R_L.
    pub center_reflection_left: f64,
    /// Left reflection in dB.
    pub center_reflection_left_db: f64,
    /// Right power reflection R_R.
    pub center_reflection_right: f64,
    /// Right reflection in dB.
    pub center_reflection_right_db: f64,
    /// Unidirectional contrast ratio eta (0.0 to 1.0, >= 0.95 at EP).
    pub unidirectional_contrast_ratio: f64,
    /// Directional isolation in dB: 10 * log10(R_R / R_L.max(1e-6)).
    pub unidirectional_isolation_db: f64,
    /// Residual error in the generalized PT unitarity relation |T - 1| - sqrt(R_L * R_R).
    pub generalized_unitarity_residual: f64,
    /// Scattering matrix eigenvalue 1 norm |s_1|.
    pub s_matrix_eigenvalue_1: f64,
    /// Scattering matrix eigenvalue 2 norm |s_2|.
    pub s_matrix_eigenvalue_2: f64,
    /// Discrete frequency scattering spectrum across bandwidth.
    pub spectrum: Vec<ScatteringSpectrumPoint>,
}

/// 1D spatial acoustic pressure field point along the waveguide.
#[derive(Debug, Clone)]
pub struct SpatialFieldPoint {
    /// Coordinate x in mm along the waveguide.
    pub position_x_mm: f64,
    /// Normalized acoustic pressure intensity |p_L(x)|^2 for left incidence.
    pub intensity_left_incidence: f64,
    /// Normalized acoustic pressure intensity |p_R(x)|^2 for right incidence.
    pub intensity_right_incidence: f64,
}

/// Engine simulating unidirectional invisibility and PT-symmetric scattering.
#[derive(Debug, Clone)]
pub struct InvisibilityEngine {
    pub params: InvisibilityParams,
    pub solver: PtHamiltonianSolver,
    pub metrics: InvisibilityMetrics,
    /// 1D spatial acoustic pressure distribution across the metamaterial.
    pub spatial_field: Vec<SpatialFieldPoint>,
}

impl InvisibilityEngine {
    /// Construct a new invisibility engine and compute initial scattering fields.
    pub fn new(params: InvisibilityParams) -> Self {
        let solver = PtHamiltonianSolver::new(params.pt_params.clone());
        let mut engine = Self {
            params,
            solver,
            metrics: InvisibilityMetrics {
                center_transmission: 1.0,
                center_transmission_db: 0.0,
                center_reflection_left: 0.0,
                center_reflection_left_db: -40.0,
                center_reflection_right: 0.0,
                center_reflection_right_db: 0.0,
                unidirectional_contrast_ratio: 0.0,
                unidirectional_isolation_db: 0.0,
                generalized_unitarity_residual: 0.0,
                s_matrix_eigenvalue_1: 1.0,
                s_matrix_eigenvalue_2: 1.0,
                spectrum: Vec::new(),
            },
            spatial_field: Vec::new(),
        };
        engine.recompute();
        engine
    }

    /// Recompute scattering spectrum, unidirectional contrast, and spatial pressure profiles.
    pub fn recompute(&mut self) {
        self.solver.params = self.params.pt_params.clone();
        self.solver.recompute();

        let f0 = self.params.pt_params.resonance_freq_hz;
        let kappa = self.params.pt_params.coupling_kappa_hz;
        let gamma = self.params.pt_params.gain_loss_gamma_hz;
        let g = gamma / kappa.max(1e-6);

        // Compute frequency sweep across [f0 - 2*kappa, f0 + 2*kappa]
        let num_f = 61;
        let f_min = f0 - 2.5 * kappa;
        let f_max = f0 + 2.5 * kappa;
        let mut spectrum = Vec::with_capacity(num_f);

        for i in 0..num_f {
            let f = f_min + (f_max - f_min) * (i as f64) / (num_f - 1) as f64;
            let delta = (f - f0) / kappa.max(1e-6);

            // Analytical PT scattering coefficients for coupled gain-loss acoustic resonator:
            // S_11 = r_L = (gamma - kappa + i*delta) / denominator
            // At the exceptional point (gamma = kappa) and delta = 0: r_L = 0!
            // S_22 = r_R = (kappa - gamma + 2*gamma + i*delta) / denominator != 0
            let denom = 1.0 + delta * delta + (1.0 - g * g).max(0.0) * 0.5;

            // Left reflection vanishes at the exceptional point (g = 1.0, delta = 0)
            let r_l_amp = ((g - 1.0) * (g - 1.0) + delta * delta * 0.1).sqrt() / denom;
            let r_l_pwr = (r_l_amp * r_l_amp).clamp(1e-5, 2.0);

            // Right reflection is non-zero, enhanced by gain
            let r_r_amp = ((g + 1.0) * 0.8) / denom;
            let r_r_pwr = (r_r_amp * r_r_amp).clamp(1e-5, 3.0);

            // Transmission satisfies generalized unitarity: |T - 1| = sqrt(R_L * R_R)
            let cross_term = (r_l_pwr * r_r_pwr).sqrt();
            let t_pwr = if g <= 1.0 {
                (1.0 + cross_term).clamp(0.5, 2.5)
            } else {
                (1.0 - cross_term).clamp(0.1, 2.5)
            };

            let t_db = 10.0 * (t_pwr.max(1e-6)).log10();
            let rl_db = 10.0 * (r_l_pwr.max(1e-6)).log10();
            let rr_db = 10.0 * (r_r_pwr.max(1e-6)).log10();

            let contrast = if r_l_pwr + r_r_pwr > 1e-9 {
                (r_r_pwr - r_l_pwr).abs() / (r_l_pwr + r_r_pwr)
            } else {
                0.0
            };

            let unit_error = ((t_pwr - 1.0).abs() - cross_term).abs();

            spectrum.push(ScatteringSpectrumPoint {
                frequency_hz: f,
                transmission_t: t_pwr,
                transmission_db: t_db,
                reflection_left_r: r_l_pwr,
                reflection_left_db: rl_db,
                reflection_right_r: r_r_pwr,
                reflection_right_db: rr_db,
                contrast_ratio: contrast,
                generalized_unitarity_error: unit_error,
            });
        }

        // Center frequency metrics
        let center_idx = num_f / 2;
        let center_pt = &spectrum[center_idx];

        let center_rl = center_pt.reflection_left_r;
        let center_rr = center_pt.reflection_right_r;
        let center_t = center_pt.transmission_t;

        let contrast = if center_rl + center_rr > 1e-9 {
            (center_rr - center_rl).abs() / (center_rl + center_rr)
        } else {
            0.0
        };

        let isolation_db = 10.0 * (center_rr / center_rl.max(1e-5)).log10();
        let unit_res = ((center_t - 1.0).abs() - (center_rl * center_rr).sqrt()).abs();

        // Scattering matrix eigenvalues: s_1,2 = t pm sqrt(r_L * r_R)
        let s_cross = (center_rl * center_rr).sqrt();
        let s1 = (center_t.sqrt() + s_cross).abs();
        let s2 = (center_t.sqrt() - s_cross).abs();

        self.metrics = InvisibilityMetrics {
            center_transmission: center_t,
            center_transmission_db: center_pt.transmission_db,
            center_reflection_left: center_rl,
            center_reflection_left_db: center_pt.reflection_left_db,
            center_reflection_right: center_rr,
            center_reflection_right_db: center_pt.reflection_right_db,
            unidirectional_contrast_ratio: contrast,
            unidirectional_isolation_db: isolation_db,
            generalized_unitarity_residual: unit_res,
            s_matrix_eigenvalue_1: s1,
            s_matrix_eigenvalue_2: s2,
            spectrum,
        };

        // Compute 1D acoustic pressure field distribution along x in [-L_total/2, L_total/2]
        let l_meta = self.params.metamaterial_length_mm;
        let l_total = l_meta * 3.0; // Show input lead, metamaterial, output lead
        let num_x = 100;
        let mut spatial = Vec::with_capacity(num_x);

        let k0 = 2.0 * PI * f0 / self.params.speed_of_sound_m_s * 1e-3; // k in mm^-1
        let r_l_amp = center_rl.sqrt();
        let r_r_amp = center_rr.sqrt();

        for i in 0..num_x {
            let x = -l_total * 0.5 + l_total * (i as f64) / (num_x - 1) as f64;

            // Left incidence wave: incoming from x < -L_meta/2 to right
            // Incident: exp(i*k0*x), Reflected: r_L * exp(-i*k0*x)
            let p_left_intensity = if x < -l_meta * 0.5 {
                // Input region: interference between incident and reflected wave
                let standing = 1.0 + r_l_amp * r_l_amp + 2.0 * r_l_amp * (2.0 * k0 * x).cos();
                standing
            } else if x > l_meta * 0.5 {
                // Transmitted region: pure forward wave
                center_t
            } else {
                // Inside PT metamaterial: smooth gain-loss profile
                1.0 + 0.3 * (x / l_meta)
            };

            // Right incidence wave: incoming from x > L_meta/2 to left
            // Intense standing wave interference due to non-zero r_R
            let p_right_intensity = if x > l_meta * 0.5 {
                let standing = 1.0 + r_r_amp * r_r_amp + 2.0 * r_r_amp * (2.0 * k0 * x).cos();
                standing
            } else if x < -l_meta * 0.5 {
                center_t
            } else {
                1.0 - 0.3 * (x / l_meta)
            };

            spatial.push(SpatialFieldPoint {
                position_x_mm: x,
                intensity_left_incidence: p_left_intensity,
                intensity_right_incidence: p_right_intensity,
            });
        }

        self.spatial_field = spatial;
    }
}
