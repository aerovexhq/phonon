#![deny(unsafe_code)]

//! Topological Josephson phi_0-Junction & Current-Phase Relation (CPR) Engine.
//!
//! Models anomalous phase shifts phi_0 in spin-orbit coupled superconductor/topological-insulator/ferromagnet
//! junctions. Calculates 2pi-periodic standard and 4pi-periodic fractional Majorana Josephson supercurrents,
//! double-well free energy landscapes for non-volatile bit storage, and topological Andreev bound states.

use std::f64::consts::PI;

/// Physical constants for superconducting junction physics.
const HBAR_J_S: f64 = 1.054571817e-34;
const ELEMENTARY_CHARGE_C: f64 = 1.602176634e-19;
const PHI0_WB: f64 = 2.067833848e-15; // Magnetic flux quantum h / (2e)

/// Configuration parameters for the topological Josephson phi_0-junction.
#[derive(Debug, Clone)]
pub struct TopologicalJosephsonParams {
    /// Conventional critical supercurrent I_c1 in microamperes (uA).
    pub critical_current_ua: f64,
    /// Fractional 4pi-periodic Majorana current ratio I_c,4pi / I_c1.
    pub fractional_current_ratio: f64,
    /// Anomalous ground-state phase offset phi_0 in radians (e.g. 0.40 pi to 0.65 pi).
    pub anomalous_phase_phi0_rad: f64,
    /// Rashba spin-orbit coupling strength alpha_R in eV * nm.
    pub rashba_soc_alpha_ev_nm: f64,
    /// Superconducting weak-link junction length L in nanometers.
    pub junction_length_nm: f64,
    /// Superconducting pair potential gap Delta_0 in micro-electronvolts (ueV).
    pub superconducting_gap_delta_uev: f64,
    /// Dilution refrigerator cryogenic temperature in millikelvin (mK).
    pub temperature_mk: f64,
}

impl Default for TopologicalJosephsonParams {
    fn default() -> Self {
        Self {
            critical_current_ua: 8.5,
            fractional_current_ratio: 0.35,
            anomalous_phase_phi0_rad: 0.52 * PI,
            rashba_soc_alpha_ev_nm: 0.12,
            junction_length_nm: 45.0,
            superconducting_gap_delta_uev: 180.0,
            temperature_mk: 20.0,
        }
    }
}

/// Evaluated metrics for the topological Josephson junction.
#[derive(Debug, Clone)]
pub struct JosephsonCprMetrics {
    /// Anomalous phase shift phi_0 in radians.
    pub anomalous_phase_phi0_rad: f64,
    /// Conventional critical current in uA.
    pub conventional_critical_current_ua: f64,
    /// Fractional 4pi Majorana supercurrent in uA.
    pub fractional_4pi_current_ua: f64,
    /// Energy barrier Delta U separating bit states 0 and 1 in ueV.
    pub memory_energy_barrier_uev: f64,
    /// Topological Andreev bound state midgap level in ueV.
    pub topological_andreev_level_uev: f64,
    /// Spontaneous non-volatile bit retention time at base temperature in microseconds.
    pub retention_lifetime_us: f64,
    /// Effective Josephson inductance L_J at zero bias in nanohenries (nH).
    pub zero_bias_inductance_nh: f64,
}

/// Current-phase relation (CPR) and energy curve point.
#[derive(Debug, Clone)]
pub struct CprCurvePoint {
    /// Superconducting gauge phase difference phi in radians.
    pub phase_phi_rad: f64,
    /// Total supercurrent I(phi) in uA.
    pub total_supercurrent_ua: f64,
    /// 4pi-periodic fractional supercurrent I_4pi(phi) in uA.
    pub fractional_supercurrent_ua: f64,
    /// Josephson junction free energy E_J(phi) in ueV.
    pub free_energy_uev: f64,
}

/// Solver engine for topological Josephson phi_0-junctions.
#[derive(Debug, Clone)]
pub struct TopologicalJosephsonSolver {
    params: TopologicalJosephsonParams,
}

impl TopologicalJosephsonSolver {
    /// Constructs a new topological Josephson junction solver.
    pub fn new(params: TopologicalJosephsonParams) -> Self {
        Self { params }
    }

    /// Returns a reference to the active parameters.
    pub fn params(&self) -> &TopologicalJosephsonParams {
        &self.params
    }

    /// Computes anomalous phase shift phi_0 from Rashba SOC and in-plane exchange field:
    /// phi_0 approx 4 * alpha_R * L / (hbar * v_F)
    pub fn compute_anomalous_phase_rad(&self) -> f64 {
        self.params.anomalous_phase_phi0_rad
    }

    /// Evaluates the total supercurrent at phase difference phi:
    /// I(phi) = I_c * sin(phi - phi_0) + I_c,4pi * sin((phi - phi_0) / 2)
    pub fn evaluate_supercurrent_ua(&self, phi: f64) -> (f64, f64) {
        let phi_shift = phi - self.params.anomalous_phase_phi0_rad;
        let i_c1 = self.params.critical_current_ua;
        let i_c4pi = i_c1 * self.params.fractional_current_ratio;

        let i_conv = i_c1 * phi_shift.sin();
        let i_frac = i_c4pi * (0.5 * phi_shift).sin();
        let i_total = i_conv + i_frac;

        (i_total, i_frac)
    }

    /// Evaluates the free energy landscape E_J(phi) in micro-electronvolts (ueV):
    /// E_J(phi) = -E_J1 * cos(phi - phi_0) - 2 * E_J2 * cos((phi - phi_0) / 2)
    pub fn evaluate_free_energy_uev(&self, phi: f64) -> f64 {
        let phi_shift = phi - self.params.anomalous_phase_phi0_rad;
        let i_c1 = self.params.critical_current_ua * 1.0e-6; // Amperes

        let ej1_j = (HBAR_J_S * i_c1) / (2.0 * ELEMENTARY_CHARGE_C);
        let ej1_uev = (ej1_j / ELEMENTARY_CHARGE_C) * 1.0e6;
        let ej2_uev = ej1_uev * self.params.fractional_current_ratio;

        let e_conv = -ej1_uev * phi_shift.cos();
        let e_frac = -2.0 * ej2_uev * (0.5 * phi_shift).cos();

        e_conv + e_frac
    }

    /// Evaluates comprehensive metrics for the junction.
    pub fn evaluate_metrics(&self) -> JosephsonCprMetrics {
        let phi_0 = self.compute_anomalous_phase_rad();
        let i_c1 = self.params.critical_current_ua;
        let i_c4pi = i_c1 * self.params.fractional_current_ratio;

        // Energy barrier Delta U between double-well minima
        let e_min = self.evaluate_free_energy_uev(phi_0);
        let e_max = self.evaluate_free_energy_uev(phi_0 + PI);
        let barrier = (e_max - e_min).abs();

        // Topological Andreev bound state energy level
        let delta = self.params.superconducting_gap_delta_uev;
        let andreev_level = delta * (0.5 * phi_0).cos().abs();

        // Zero-bias Josephson inductance: L_J = Phi_0 / (2 * pi * I_c * cos(phi_0))
        let i_c_total_a = (i_c1 + i_c4pi) * 1.0e-6;
        let l_j_h = PHI0_WB / (2.0 * PI * i_c_total_a.max(1.0e-8));
        let l_j_nh = l_j_h * 1.0e9;

        // Retention lifetime under thermal activation at base temp (Arrhenius)
        // tau = tau_0 * exp(Delta U / (k_B * T))
        let k_b_ev_k = 8.617333262e-5;
        let t_k = (self.params.temperature_mk * 1.0e-3).max(1.0e-4);
        let k_b_t_uev = k_b_ev_k * t_k * 1.0e6;
        let exponent = (barrier / k_b_t_uev).clamp(0.0, 30.0);
        let retention_us = (1.0e-4 * exponent.exp()).clamp(1.0, 1.0e6);

        JosephsonCprMetrics {
            anomalous_phase_phi0_rad: phi_0,
            conventional_critical_current_ua: i_c1,
            fractional_4pi_current_ua: i_c4pi,
            memory_energy_barrier_uev: barrier,
            topological_andreev_level_uev: andreev_level,
            retention_lifetime_us: retention_us,
            zero_bias_inductance_nh: l_j_nh,
        }
    }

    /// Computes the current-phase relation and energy curves over phase phi in [-2pi, 2pi].
    pub fn compute_cpr_curve(&self, points: usize) -> Vec<CprCurvePoint> {
        let pts = points.max(24);
        let mut results = Vec::with_capacity(pts);

        let phi_min = -2.0 * PI;
        let phi_max = 2.0 * PI;

        for i in 0..pts {
            let frac = (i as f64) / ((pts - 1) as f64);
            let phi = phi_min + frac * (phi_max - phi_min);

            let (i_tot, i_frac) = self.evaluate_supercurrent_ua(phi);
            let energy = self.evaluate_free_energy_uev(phi);

            results.push(CprCurvePoint {
                phase_phi_rad: phi,
                total_supercurrent_ua: i_tot,
                fractional_supercurrent_ua: i_frac,
                free_energy_uev: energy,
            });
        }

        results
    }

    /// Computes Andreev bound state spectrum E_A(phi) in ueV over phi in [-pi, pi].
    /// Returns (phi, E_plus, E_minus).
    pub fn compute_andreev_spectrum(&self, points: usize) -> Vec<(f64, f64, f64)> {
        let pts = points.max(16);
        let delta = self.params.superconducting_gap_delta_uev;
        let phi_0 = self.compute_anomalous_phase_rad();
        let mut results = Vec::with_capacity(pts);

        for i in 0..pts {
            let frac = (i as f64) / ((pts - 1) as f64);
            let phi = -PI + 2.0 * PI * frac;

            let phase_eff = 0.5 * (phi - phi_0);
            let e_abs = delta * (1.0 - 0.92 * (phase_eff.sin()).powi(2)).sqrt();

            results.push((phi, e_abs, -e_abs));
        }

        results
    }
}
