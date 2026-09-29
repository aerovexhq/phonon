#![deny(unsafe_code)]

//! Non-Hermitian higher-order topological phononic lasers and chiral quadrupole
//! acoustical frequency synthesizer solver.

use phonon_models::non_hermitian_quadrupole_laser::{
    NonHermitianQuadrupoleLaserMetrics, NonHermitianQuadrupoleLaserParams,
};

/// Multi-physics solver evaluating corner mode lasing fidelity, fractional frequency instability,
/// side-mode suppression ratio, topological corner mode lifetime, and PT-symmetry confinement.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NonHermitianQuadrupoleLaserSolver {
    pub params: NonHermitianQuadrupoleLaserParams,
}

impl NonHermitianQuadrupoleLaserSolver {
    /// Creates a new solver instance with the specified physical parameters.
    pub fn new(params: NonHermitianQuadrupoleLaserParams) -> Self {
        Self { params }
    }

    /// Computes the single-mode topological corner mode lasing fidelity (target >= 0.9970).
    pub fn compute_corner_mode_lasing_fidelity(&self) -> f64 {
        let p = &self.params;
        let quad_ratio = p.quadrupole_coupling_mhz / 28.0;
        let conf_ratio = p.corner_confinement_factor / 0.88;
        let net_gain_margin = (p.pump_gain_rate_khz - p.loss_dissipation_rate_khz) / 10.0;
        let size_ratio = (p.lattice_dimension as f64) / 12.0;
        let temp_ratio = p.cryogenic_temp_mk / 15.0;
        let nonlin_ratio = p.non_linear_saturation_parameter / 0.012;

        let fidelity = 0.9982
            + 0.0006 * (quad_ratio - 1.0)
            + 0.0005 * (conf_ratio - 1.0)
            + 0.0003 * (net_gain_margin.clamp(0.2, 5.0) - 1.0)
            + 0.0002 * (size_ratio - 1.0)
            - 0.0004 * (temp_ratio - 1.0)
            - 0.0003 * (nonlin_ratio - 1.0);
        fidelity.clamp(0.980, 0.9999)
    }

    /// Computes the fractional frequency instability Allan deviation floor (target <= 1.5e-12).
    pub fn compute_fractional_frequency_instability(&self) -> f64 {
        let p = &self.params;
        let base_instability = 8.5e-13;
        let temp_factor = (p.cryogenic_temp_mk / 15.0).sqrt();
        let beta_factor = (p.non_linear_saturation_parameter / 0.012).powf(0.25);
        let loss_factor = (p.loss_dissipation_rate_khz / 110.0).powf(0.2);
        let freq_factor = (4.8 / p.acoustic_resonator_frequency_ghz).powf(0.3);
        let conf_factor = (0.88 / p.corner_confinement_factor).powf(0.5);
        let quad_factor = (28.0 / p.quadrupole_coupling_mhz).powf(0.2);
        let size_factor = (12.0 / (p.lattice_dimension as f64)).powf(0.15);

        let instability = base_instability
            * temp_factor
            * beta_factor
            * loss_factor
            * freq_factor
            * conf_factor
            * quad_factor
            * size_factor;
        instability.clamp(1.0e-14, 1.0e-10)
    }

    /// Computes the side-mode suppression ratio against bulk and edge modes in dB (target >= 45.0).
    pub fn compute_side_mode_suppression_ratio_db(&self) -> f64 {
        let p = &self.params;
        let quad_ratio = p.quadrupole_coupling_mhz / 28.0;
        let conf_ratio = p.corner_confinement_factor / 0.88;
        let size_ratio = (p.lattice_dimension as f64) / 12.0;
        let pump_ratio = p.pump_gain_rate_khz / 120.0;
        let loss_ratio = p.loss_dissipation_rate_khz / 110.0;
        let temp_ratio = p.cryogenic_temp_mk / 15.0;
        let nonlin_ratio = p.non_linear_saturation_parameter / 0.012;

        let smsr = 53.5
            + 5.0 * (quad_ratio - 1.0)
            + 6.0 * (conf_ratio - 1.0)
            + 3.5 * (size_ratio - 1.0)
            + 2.5 * (pump_ratio - 1.0)
            - 2.5 * (loss_ratio - 1.0)
            - 2.0 * (temp_ratio - 1.0)
            - 1.5 * (nonlin_ratio - 1.0);
        smsr.clamp(20.0, 90.0)
    }

    /// Computes the topological corner mode coherence lifetime in milliseconds (target >= 80.0).
    pub fn compute_topological_corner_mode_lifetime_ms(&self) -> f64 {
        let p = &self.params;
        let loss_factor = (110.0 / p.loss_dissipation_rate_khz).powf(0.65);
        let temp_factor = (15.0 / p.cryogenic_temp_mk).powf(0.35);
        let conf_factor = (p.corner_confinement_factor / 0.88).powf(0.5);
        let quad_factor = (p.quadrupole_coupling_mhz / 28.0).powf(0.25);
        let size_factor = ((p.lattice_dimension as f64) / 12.0).powf(0.2);
        let gain_margin_factor = 1.0 + 0.1 * (p.pump_gain_rate_khz / 120.0 - 1.0);

        let lifetime = 118.0
            * loss_factor
            * temp_factor
            * conf_factor
            * quad_factor
            * size_factor
            * gain_margin_factor;
        lifetime.clamp(10.0, 500.0)
    }

    /// Computes the parity-time (PT) symmetry confinement ratio in the quadrupole sector (target >= 0.920).
    pub fn compute_pt_symmetry_confinement_ratio(&self) -> f64 {
        let p = &self.params;
        let gain_loss_balance = 1.0
            - (p.pump_gain_rate_khz - p.loss_dissipation_rate_khz).abs()
                / (p.pump_gain_rate_khz + p.loss_dissipation_rate_khz);
        let balance_factor = gain_loss_balance / (1.0 - 10.0 / 230.0);
        let conf_ratio = p.corner_confinement_factor / 0.88;
        let quad_ratio = p.quadrupole_coupling_mhz / 28.0;
        let size_ratio = (p.lattice_dimension as f64) / 12.0;
        let temp_ratio = p.cryogenic_temp_mk / 15.0;
        let nonlin_ratio = p.non_linear_saturation_parameter / 0.012;

        let pt_ratio = 0.958
            + 0.015 * (conf_ratio - 1.0)
            + 0.010 * (quad_ratio - 1.0)
            + 0.008 * (balance_factor - 1.0)
            + 0.005 * (size_ratio - 1.0)
            - 0.006 * (temp_ratio - 1.0)
            - 0.004 * (nonlin_ratio - 1.0);
        pt_ratio.clamp(0.80, 0.999)
    }

    /// Evaluates complete multi-physics metrics and physical compliance status.
    pub fn evaluate_metrics(&self) -> NonHermitianQuadrupoleLaserMetrics {
        let corner_mode_lasing_fidelity = self.compute_corner_mode_lasing_fidelity();
        let fractional_frequency_instability = self.compute_fractional_frequency_instability();
        let side_mode_suppression_ratio_db = self.compute_side_mode_suppression_ratio_db();
        let topological_corner_mode_lifetime_ms =
            self.compute_topological_corner_mode_lifetime_ms();
        let pt_symmetry_confinement_ratio = self.compute_pt_symmetry_confinement_ratio();

        let is_physically_compliant = corner_mode_lasing_fidelity >= 0.9970
            && fractional_frequency_instability <= 1.5e-12
            && side_mode_suppression_ratio_db >= 45.0
            && topological_corner_mode_lifetime_ms >= 80.0
            && pt_symmetry_confinement_ratio >= 0.920;

        NonHermitianQuadrupoleLaserMetrics {
            corner_mode_lasing_fidelity,
            fractional_frequency_instability,
            side_mode_suppression_ratio_db,
            topological_corner_mode_lifetime_ms,
            pt_symmetry_confinement_ratio,
            is_physically_compliant,
        }
    }
}
