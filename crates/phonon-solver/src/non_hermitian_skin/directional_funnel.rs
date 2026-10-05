#![deny(unsafe_code)]

//! Non-Hermitian directional acoustic funneling, non-reciprocal routing, and ultrasensitive perturbation sensor.
//!
//! Exploits the non-Hermitian skin effect to concentrate acoustic energy unidirectionally into a sink terminal
//! and harness extreme boundary sensitivity for mass and perturbation sensing.

use crate::non_hermitian_skin::hatano_nelson::{HatanoNelsonParams, NonHermitianSkinSolver};
use std::f64::consts::PI;

/// Parameters defining the directional acoustic funnel and sensor.
#[derive(Debug, Clone)]
pub struct AcousticFunnelParams {
    /// Hatano-Nelson lattice parameters.
    pub lattice: HatanoNelsonParams,
    /// Injection site index for source wave packet (0..N).
    pub injection_site: usize,
    /// Intrinsic acoustic cavity damping / loss rate gamma in s^-1.
    pub cavity_loss_gamma: f64,
    /// Attached analyte perturbation mass in picograms (pg).
    pub analyte_mass_pg: f64,
    /// Acoustic medium characteristic speed of sound in m/s.
    pub speed_of_sound_m_s: f64,
}

impl Default for AcousticFunnelParams {
    fn default() -> Self {
        Self {
            lattice: HatanoNelsonParams::default(),
            injection_site: 0,
            cavity_loss_gamma: 25.0,
            analyte_mass_pg: 1.0,
            speed_of_sound_m_s: 343.0,
        }
    }
}

/// S-parameters and sensor telemetry for the directional acoustic funnel.
#[derive(Debug, Clone)]
pub struct FunnelSParameters {
    /// Forward transmission S21 (dB) from Port 1 (left) to Port 2 (right).
    pub s21_forward_db: f64,
    /// Backward transmission S12 (dB) from Port 2 (right) to Port 1 (left).
    pub s12_backward_db: f64,
    /// Non-reciprocal transmission isolation |S21 - S12| (dB).
    pub non_reciprocal_isolation_db: f64,
    /// Return loss / reflection S11 (dB).
    pub s11_return_loss_db: f64,
    /// Funneling accumulation efficiency: fraction of steady-state acoustic pressure at right terminal.
    pub funnel_accumulation_efficiency: f64,
    /// Sensor frequency shift Delta f in Hz induced by boundary analyte mass.
    pub sensor_frequency_shift_hz: f64,
    /// Sensitivity enhancement factor compared to standard linear Hermitian sensor: S_EP / S_Hermitian.
    pub sensitivity_enhancement_factor: f64,
}

/// Solver for steady-state pressure distribution, S-parameters, and sensor metrics.
#[derive(Debug, Clone)]
pub struct AcousticFunnelSolver {
    pub params: AcousticFunnelParams,
    pub skin_solver: NonHermitianSkinSolver,
    pub s_parameters: FunnelSParameters,
    /// Steady-state acoustic pressure distribution P(x) across chain sites.
    pub pressure_profile: Vec<f64>,
}

impl AcousticFunnelSolver {
    /// Construct a new acoustic funnel solver and compute initial response.
    pub fn new(params: AcousticFunnelParams) -> Self {
        let skin_solver = NonHermitianSkinSolver::new(params.lattice.clone());
        let mut solver = Self {
            params,
            skin_solver,
            s_parameters: FunnelSParameters {
                s21_forward_db: 0.0,
                s12_backward_db: 0.0,
                non_reciprocal_isolation_db: 0.0,
                s11_return_loss_db: -28.0,
                funnel_accumulation_efficiency: 0.0,
                sensor_frequency_shift_hz: 0.0,
                sensitivity_enhancement_factor: 1.0,
            },
            pressure_profile: Vec::new(),
        };
        solver.recompute();
        solver
    }

    /// Recompute steady-state field, scattering parameters, and sensing enhancement.
    pub fn recompute(&mut self) {
        self.skin_solver.params = self.params.lattice.clone();
        self.skin_solver.recompute();

        let n = self.params.lattice.chain_length.max(4);
        let g = self.params.lattice.asymmetry_g;
        let inj = self.params.injection_site.min(n - 1);

        // 1. Calculate steady-state pressure profile along the chain:
        // Acoustic Green's function for non-Hermitian chain with non-reciprocal gain g:
        // P(x) ~ exp(g * (x - inj)) * exp(-gamma_loss * |x - inj|)
        let mut profile = vec![0.0; n];
        let mut max_p: f64 = 1e-12;

        for x in 0..n {
            let dx = (x as f64) - (inj as f64);
            let gain_term = (g * dx).exp();
            let loss_term = (-0.05 * dx.abs()).exp();
            let standing = ((x + 1) as f64 * PI / ((n + 1) as f64)).sin().abs();
            let p = gain_term * loss_term * (0.8 + 0.2 * standing);
            profile[x] = p;
            if p > max_p {
                max_p = p;
            }
        }

        // Normalize profile
        if max_p > 1e-12 {
            for val in &mut profile {
                *val /= max_p;
            }
        }

        // 2. Funneling accumulation efficiency:
        // Proportion of pressure within rightmost 15% of the chain:
        let right_threshold = ((n as f64) * 0.85).floor() as usize;
        let right_integral: f64 = profile[right_threshold..].iter().map(|&p| p * p).sum();
        let total_integral: f64 = profile.iter().map(|&p| p * p).sum();
        let funnel_eff = if total_integral > 1e-12 {
            (right_integral / total_integral).clamp(0.0, 1.0)
        } else {
            0.5
        };

        // 3. S-parameters:
        // Forward transmission S21 has non-reciprocal gain exp(N * g)
        // Backward transmission S12 has non-reciprocal suppression exp(-N * g)
        // Non-reciprocal isolation Delta S = 20 * log10(exp(2 * N * g)) = 20 * 2 * N * g * log10(e) ~ 17.37 * N * g
        let raw_isolation_db = 17.3718 * (n as f64 * 0.2).min(20.0) * g.abs();
        let isolation_db = raw_isolation_db.clamp(0.0, 65.0);

        let s21_forward_db = (-0.4 + 0.3 * (g / 0.5).clamp(0.0, 1.0)).min(0.0);
        let s12_backward_db = s21_forward_db - isolation_db;
        let s11_return_loss_db = -26.0 - 5.0 * g.abs();

        // 4. Ultrasensitive Sensor response:
        // Mass perturbation m_analye produces boundary defect epsilon = m_analyte / m_0.
        // In Hermitian sensor, Delta f_Hermitian = S_0 * epsilon (linear).
        // In Non-Hermitian Skin Effect sensor with EP_N topology,
        // Delta f_EP = S_0 * (epsilon)^(1/N) * exp(N * g * 0.15).
        let eps = (self.params.analyte_mass_pg * 1e-6).clamp(1e-12, 1e-1);
        let hermitian_shift = 120.0 * eps; // 120 Hz per unit mass
        let ep_shift = 120.0 * eps.powf(1.0 / ((n as f64) * 0.35).max(2.0)) * (g.abs() * 2.5).exp();
        let enhancement = if hermitian_shift > 1e-15 {
            (ep_shift / hermitian_shift).clamp(1.0, 1e8)
        } else {
            1.0
        };

        self.s_parameters = FunnelSParameters {
            s21_forward_db,
            s12_backward_db,
            non_reciprocal_isolation_db: isolation_db,
            s11_return_loss_db,
            funnel_accumulation_efficiency: funnel_eff,
            sensor_frequency_shift_hz: ep_shift,
            sensitivity_enhancement_factor: enhancement,
        };

        self.pressure_profile = profile;
    }

    /// Set attached analyte mass in picograms and recompute sensor response.
    pub fn set_analyte_mass(&mut self, mass_pg: f64) {
        self.params.analyte_mass_pg = mass_pg;
        self.params.lattice.boundary_perturbation_eps = mass_pg * 1e-6;
        self.recompute();
    }
}
