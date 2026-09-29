#![deny(unsafe_code)]

//! Physical parameters and multi-physics evaluation metrics for topological
//! acoustic skyrmion lattices and chiral phononic neuromorphic processing engines.

/// Physical parameter configuration for topological acoustic skyrmion lattices.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TopologicalAcousticSkyrmionParams {
    /// Interfacial Dzyaloshinskii-Moriya interaction (DMI) strength in mJ/m^2 (clamp 0.5 to 5.0, default 2.2).
    pub dmi_strength_mj_m2: f64,
    /// Symmetric Heisenberg exchange stiffness in pJ/m (clamp 5.0 to 30.0, default 15.0).
    pub exchange_stiffness_pj_m: f64,
    /// Perpendicular magnetocrystalline/strain anisotropy in MJ/m^3 (clamp 0.1 to 2.5, default 0.8).
    pub anisotropy_mj_m3: f64,
    /// Gilbert damping parameter alpha (clamp 0.001 to 0.08, default 0.015).
    pub gilbert_damping_alpha: f64,
    /// Acoustic drive current density in mA/um^2 (clamp 0.1 to 10.0, default 3.5).
    pub acoustic_drive_current_ma_um2: f64,
    /// Phononic skyrmion crystal lattice constant a_0 in nm (clamp 20.0 to 200.0, default 65.0).
    pub lattice_constant_nm: f64,
    /// Isolated topological acoustic skyrmion core diameter in nm (clamp 15.0 to 120.0, default 42.0).
    pub skyrmion_diameter_nm: f64,
    /// Operating cryogenic temperature in Kelvin (clamp 0.01 to 10.0, default 1.5).
    pub cryogenic_temp_k: f64,
}

impl Default for TopologicalAcousticSkyrmionParams {
    fn default() -> Self {
        Self {
            dmi_strength_mj_m2: 2.2,
            exchange_stiffness_pj_m: 15.0,
            anisotropy_mj_m3: 0.8,
            gilbert_damping_alpha: 0.015,
            acoustic_drive_current_ma_um2: 3.5,
            lattice_constant_nm: 65.0,
            skyrmion_diameter_nm: 42.0,
            cryogenic_temp_k: 1.5,
        }
    }
}

impl TopologicalAcousticSkyrmionParams {
    /// Creates a new parameter configuration with physical boundary clamping.
    pub fn new(
        dmi_strength_mj_m2: f64,
        exchange_stiffness_pj_m: f64,
        anisotropy_mj_m3: f64,
        gilbert_damping_alpha: f64,
        acoustic_drive_current_ma_um2: f64,
        lattice_constant_nm: f64,
        skyrmion_diameter_nm: f64,
        cryogenic_temp_k: f64,
    ) -> Self {
        Self {
            dmi_strength_mj_m2: dmi_strength_mj_m2.clamp(0.5, 5.0),
            exchange_stiffness_pj_m: exchange_stiffness_pj_m.clamp(5.0, 30.0),
            anisotropy_mj_m3: anisotropy_mj_m3.clamp(0.1, 2.5),
            gilbert_damping_alpha: gilbert_damping_alpha.clamp(0.001, 0.08),
            acoustic_drive_current_ma_um2: acoustic_drive_current_ma_um2.clamp(0.1, 10.0),
            lattice_constant_nm: lattice_constant_nm.clamp(20.0, 200.0),
            skyrmion_diameter_nm: skyrmion_diameter_nm.clamp(15.0, 120.0),
            cryogenic_temp_k: cryogenic_temp_k.clamp(0.01, 10.0),
        }
    }
}

/// Multi-physics performance evaluation metrics for topological acoustic skyrmion
/// lattices and chiral phononic neuromorphic processing engines.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TopologicalAcousticSkyrmionMetrics {
    /// Synaptic weight encoding and state retention fidelity (target >= 0.9960).
    pub synaptic_state_fidelity: f64,
    /// Steady-state acoustic skyrmion propagation velocity in m/s (target >= 850.0).
    pub skyrmion_propagation_velocity_mps: f64,
    /// Real-space topological charge quantization error |Q - 1.0| (target <= 0.0030).
    pub topological_charge_quantization_error: f64,
    /// Energy dissipated per neuromorphic synaptic event in attojoules (aJ, target <= 15.0).
    pub neuromorphic_energy_dissipation_aj: f64,
    /// Non-volatile state retention isolation against depinning and crosstalk in dB (target >= 42.0).
    pub state_retention_isolation_db: f64,
    /// Overall physical compliance flag across all roadmap targets.
    pub is_physically_compliant: bool,
}
