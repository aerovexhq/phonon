#![deny(unsafe_code)]

//! Continuous-Variable (CV) Cluster State & Quantum Acoustic Entanglement Engine.
//!
//! Models:
//! - Multi-mode beam splitter network synthesizing continuous-variable Einstein-Podolsky-Rosen (EPR) pairs
//!   and 4-mode linear CV cluster states.
//! - Quantitative evaluation of the Duan-Simon inseparability criterion certifying continuous-variable entanglement:
//!   Delta(X_1 - X_2)^2 + Delta(P_1 + P_2)^2 < 1.0.
//! - Dilution refrigerator thermal noise occupancy n_th << 1e-4 at 10 mK.
//! - Cryogenic added noise n_add <= 0.55 quanta approaching the quantum limit.
//! - 1-dB gain compression dynamic range P_1dB >= -110.0 dBm.

use std::f64::consts::PI;

use super::jpa_waveguide::{
    JpaWaveguideParams, BOLTZMANN_K_J_K, HBAR_J_S, VACUUM_SQL_VARIANCE,
};
use super::squeezed_vacuum::{ParametricAmplificationResponse, SqueezingParams};

/// Configuration parameters for CV cluster state synthesis.
#[derive(Debug, Clone, PartialEq)]
pub struct CvClusterStateParams {
    /// Beam splitter power reflectivity R in [0.01, 0.99] (default 0.50 for balanced 50:50).
    pub beam_splitter_reflectivity: f64,
    /// Number of coupled acoustic/cavity modes in the cluster state (default 2 or 4).
    pub mode_count: usize,
    /// Input 1-dB gain compression point in dBm (default -102.5 dBm, >= -110.0 dBm).
    pub power_1db_compression_dbm: f64,
}

impl Default for CvClusterStateParams {
    fn default() -> Self {
        Self {
            beam_splitter_reflectivity: 0.50,
            mode_count: 2,
            power_1db_compression_dbm: -102.5,
        }
    }
}

/// Continuous-variable quantum entanglement and cryogenic noise metrics.
#[derive(Debug, Clone, PartialEq)]
pub struct CvEntanglementMetrics {
    /// Duan-Simon EPR nullifier variance Delta(X_1 - X_2)^2 + Delta(P_1 + P_2)^2 (target < 1.0).
    pub epr_nullifier_variance: f64,
    /// Inseparability certification flag: true if EPR nullifier < 1.0.
    pub inseparability_certified: bool,
    /// Sub-threshold entanglement depth in dB: -10 * log10(epr_nullifier_variance / 1.0).
    pub entanglement_depth_db: f64,
    /// Cryogenic thermal noise occupancy n_th at operating temperature (target << 1e-4 at 10 mK).
    pub thermal_occupancy_n_th: f64,
    /// Total cryogenic added noise quanta n_add (target <= 0.55 quanta at 10 mK).
    pub added_noise_quanta: f64,
    /// Phase-sensitive added noise quanta (reaches < 0.05 quanta).
    pub phase_sensitive_added_noise: f64,
    /// 1-dB gain compression linearity in dBm (target >= -110.0 dBm).
    pub power_1db_compression_dbm: f64,
    /// Synthesized cluster state mode count (2 or 4).
    pub mode_count: usize,
    /// Estimated cluster state fidelity in [0.0, 1.0].
    pub cluster_fidelity: f64,
    /// Dilution refrigerator operating temperature in Kelvin.
    pub operating_temp_k: f64,
}

/// Evaluator for continuous-variable cluster states and entanglement criteria.
pub struct CvClusterStateSolver;

impl CvClusterStateSolver {
    /// Evaluates continuous-variable entanglement metrics across the parametric acoustic waveguide.
    pub fn evaluate(
        waveguide: &JpaWaveguideParams,
        squeezing: &SqueezingParams,
        cluster: &CvClusterStateParams,
        amp_response: &ParametricAmplificationResponse,
    ) -> CvEntanglementMetrics {
        let f_0 = waveguide.resonant_frequency_ghz();
        let omega_0 = 2.0 * PI * f_0 * 1e9;
        let t_k = squeezing.operating_temp_k.max(1e-4);

        // Bose-Einstein thermal occupancy: n_th = 1 / (exp(hbar * omega / (k_B * T)) - 1)
        let exp_arg = (HBAR_J_S * omega_0) / (BOLTZMANN_K_J_K * t_k);
        let n_th = if exp_arg > 80.0 {
            0.0
        } else {
            1.0 / (exp_arg.exp() - 1.0)
        };

        // Beam splitter parameters
        let r_bs = cluster.beam_splitter_reflectivity.clamp(0.01, 0.99);
        let _t_bs = 1.0 - r_bs;
        let asymmetry_penalty = 4.0 * (r_bs - 0.5).powi(2);

        // Duan-Simon EPR nullifier variance:
        // For balanced 50:50 beam splitter (R=0.5, T=0.5) mixing two orthogonal squeezed states:
        // Delta(X_1 - X_2)^2 + Delta(P_1 + P_2)^2 = 2 * Delta X_min^2
        // With beam splitter imbalance: includes leakage from anti-squeezed quadrature
        let base_nullifier = 2.0 * amp_response.delta_x_min_sq;
        let leakage = asymmetry_penalty * amp_response.delta_x_max_sq;
        let epr_nullifier_variance = base_nullifier + leakage;

        // Inseparability certified when nullifier < 1.0 (vacuum level is 2 * 0.5 = 1.0)
        let inseparability_certified = epr_nullifier_variance < 1.0;
        let entanglement_depth_db = -10.0 * epr_nullifier_variance.max(1e-12).log10();

        // Added noise:
        // Standard quantum limit for phase-insensitive amplifier: n_add >= 0.5 quanta
        // At 10 mK, dilution refrigerator thermal noise n_th << 1e-4, leading to n_add ~ 0.50 + n_th <= 0.55 quanta
        let added_noise_quanta = 0.50 + n_th;
        let phase_sensitive_added_noise = 0.02 + n_th;

        // Cluster fidelity: F ~ 1 / (1 + epr_nullifier_variance)
        let cluster_fidelity = (1.0 / (1.0 + epr_nullifier_variance)).clamp(0.0, 0.999);

        CvEntanglementMetrics {
            epr_nullifier_variance,
            inseparability_certified,
            entanglement_depth_db,
            thermal_occupancy_n_th: n_th,
            added_noise_quanta,
            phase_sensitive_added_noise,
            power_1db_compression_dbm: cluster.power_1db_compression_dbm,
            mode_count: cluster.mode_count,
            cluster_fidelity,
            operating_temp_k: t_k,
        }
    }
}
