#![deny(unsafe_code)]

//! Topological acoustic skyrmion lattices and chiral phononic neuromorphic processing engine solver.

use phonon_models::topological_acoustic_skyrmion::{
    TopologicalAcousticSkyrmionMetrics, TopologicalAcousticSkyrmionParams,
};

/// Multi-physics solver evaluating synaptic state fidelity, acoustic skyrmion propagation velocity,
/// topological charge quantization error, neuromorphic energy dissipation, and non-volatile state retention isolation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TopologicalAcousticSkyrmionSolver {
    pub params: TopologicalAcousticSkyrmionParams,
}

impl TopologicalAcousticSkyrmionSolver {
    /// Creates a new solver instance with the specified physical parameters.
    pub fn new(params: TopologicalAcousticSkyrmionParams) -> Self {
        Self { params }
    }

    /// Computes the synaptic weight encoding and state retention fidelity (target >= 0.9960).
    pub fn compute_synaptic_state_fidelity(&self) -> f64 {
        let p = &self.params;
        let d_ratio = p.dmi_strength_mj_m2 / 2.2;
        let a_ratio = p.exchange_stiffness_pj_m / 15.0;
        let k_ratio = p.anisotropy_mj_m3 / 0.8;
        let alpha_ratio = p.gilbert_damping_alpha / 0.015;
        let temp_ratio = p.cryogenic_temp_k / 1.5;
        let geom_factor = (p.skyrmion_diameter_nm / p.lattice_constant_nm) / (42.0 / 65.0);

        let fidelity = 0.9978
            + 0.0008 * (d_ratio - 1.0)
            + 0.0004 * (a_ratio - 1.0)
            + 0.0004 * (k_ratio - 1.0)
            - 0.0006 * (alpha_ratio - 1.0)
            - 0.0008 * (temp_ratio - 1.0)
            + 0.0004 * (geom_factor.min(1.5).max(0.5) - 1.0);
        fidelity.clamp(0.980, 0.9999)
    }

    /// Computes the steady-state acoustic skyrmion propagation velocity in m/s (target >= 850.0).
    pub fn compute_skyrmion_propagation_velocity_mps(&self) -> f64 {
        let p = &self.params;
        let j_ratio = p.acoustic_drive_current_ma_um2 / 3.5;
        let alpha_ratio = p.gilbert_damping_alpha / 0.015;
        let dmi_ratio = p.dmi_strength_mj_m2 / 2.2;
        let diam_ratio = (p.skyrmion_diameter_nm / 42.0).powf(0.1);
        let temp_factor = (1.5 / p.cryogenic_temp_k.max(0.01)).powf(0.05);

        // Nominal baseline velocity ~ 960.0 m/s
        let velocity = 960.0
            * j_ratio.powf(0.45)
            * dmi_ratio.powf(0.2)
            * diam_ratio
            * temp_factor
            / alpha_ratio.powf(0.15);
        velocity.clamp(100.0, 3500.0)
    }

    /// Computes the real-space topological charge quantization error |Q - 1.0| (target <= 0.0030).
    pub fn compute_topological_charge_quantization_error(&self) -> f64 {
        let p = &self.params;
        let discretization_ratio = (p.lattice_constant_nm / 65.0) / (p.skyrmion_diameter_nm / 42.0);
        let a_ratio = p.exchange_stiffness_pj_m / 15.0;
        let dmi_ratio = p.dmi_strength_mj_m2 / 2.2;
        let k_ratio = p.anisotropy_mj_m3 / 0.8;
        let temp_ratio = p.cryogenic_temp_k / 1.5;

        let error = 0.0015
            * discretization_ratio.powi(2)
            * temp_ratio.powf(0.35)
            / (a_ratio.sqrt() * dmi_ratio.powf(0.3) * k_ratio.powf(0.2));
        error.clamp(0.0001, 0.050)
    }

    /// Computes the energy dissipated per neuromorphic synaptic event in attojoules (aJ, target <= 15.0).
    pub fn compute_neuromorphic_energy_dissipation_aj(&self) -> f64 {
        let p = &self.params;
        let alpha_ratio = p.gilbert_damping_alpha / 0.015;
        let j_ratio = p.acoustic_drive_current_ma_um2 / 3.5;
        let diam_ratio = p.skyrmion_diameter_nm / 42.0;
        let a_ratio = p.exchange_stiffness_pj_m / 15.0;
        let temp_ratio = p.cryogenic_temp_k / 1.5;

        let dissipation = 7.5
            * alpha_ratio.powf(0.7)
            * j_ratio.powf(1.2)
            * diam_ratio.powf(0.5)
            * (0.8 + 0.2 * temp_ratio)
            / a_ratio.powf(0.2);
        dissipation.clamp(0.1, 50.0)
    }

    /// Computes the non-volatile state retention isolation against depinning and crosstalk in dB (target >= 42.0).
    pub fn compute_state_retention_isolation_db(&self) -> f64 {
        let p = &self.params;
        let k_ratio = p.anisotropy_mj_m3 / 0.8;
        let dmi_ratio = p.dmi_strength_mj_m2 / 2.2;
        let a_ratio = p.exchange_stiffness_pj_m / 15.0;
        let temp_ratio = p.cryogenic_temp_k / 1.5;
        let spacing_ratio = p.lattice_constant_nm / 65.0;

        let isolation = 46.5
            + 4.0 * (k_ratio - 1.0)
            + 3.5 * (dmi_ratio - 1.0)
            + 2.0 * (a_ratio - 1.0)
            + 1.5 * (spacing_ratio - 1.0)
            - 3.0 * (temp_ratio - 1.0);
        isolation.clamp(20.0, 75.0)
    }

    /// Evaluates complete multi-physics metrics and physical compliance status.
    pub fn evaluate_metrics(&self) -> TopologicalAcousticSkyrmionMetrics {
        let synaptic_state_fidelity = self.compute_synaptic_state_fidelity();
        let skyrmion_propagation_velocity_mps = self.compute_skyrmion_propagation_velocity_mps();
        let topological_charge_quantization_error =
            self.compute_topological_charge_quantization_error();
        let neuromorphic_energy_dissipation_aj = self.compute_neuromorphic_energy_dissipation_aj();
        let state_retention_isolation_db = self.compute_state_retention_isolation_db();

        let is_physically_compliant = synaptic_state_fidelity >= 0.9960
            && skyrmion_propagation_velocity_mps >= 850.0
            && topological_charge_quantization_error <= 0.0030
            && neuromorphic_energy_dissipation_aj <= 15.0
            && state_retention_isolation_db >= 42.0;

        TopologicalAcousticSkyrmionMetrics {
            synaptic_state_fidelity,
            skyrmion_propagation_velocity_mps,
            topological_charge_quantization_error,
            neuromorphic_energy_dissipation_aj,
            state_retention_isolation_db,
            is_physically_compliant,
        }
    }
}
