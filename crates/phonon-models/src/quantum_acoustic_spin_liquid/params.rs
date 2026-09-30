#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for non-Abelian
//! quantum acoustic fractional spin liquids and topological resonating valence bond networks.

/// Physical parameter configuration for non-Abelian quantum acoustic fractional spin liquids
/// and topological resonating valence bond (RVB) networks on frustrated lattices.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuantumAcousticSpinLiquidParams {
    /// Nearest-neighbor Heisenberg exchange coupling J1 in MHz (clamp 10.0 to 150.0, default 55.0 MHz).
    pub heisenberg_exchange_coupling_mhz: f64,
    /// Next-nearest-neighbor frustration ratio J2/J1 (clamp 0.05 to 0.60, default 0.28).
    pub frustration_ratio_j2_j1: f64,
    /// Coherent spinon-phonon acoustic gauge coupling in MHz (clamp 1.0 to 30.0, default 8.5 MHz).
    pub spinon_phonon_coupling_mhz: f64,
    /// Chiral three-spin scalar chirality J_chi in MHz (clamp 0.5 to 20.0, default 4.2 MHz).
    pub chiral_three_spin_scalar_chirality: f64,
    /// Number of Kagome or frustrated plaquettes in the simulation cluster (clamp 8 to 64, default 24).
    pub kagome_plaquette_count: usize,
    /// Resonant acoustic driving frequency in GHz (clamp 1.0 to 12.0, default 4.6 GHz).
    pub acoustic_driving_frequency_ghz: f64,
    /// Cryogenic operating temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0 mK).
    pub cryogenic_temperature_mk: f64,
    /// Frustrated lattice geometry type (0: Kagome, 1: Triangular, clamp 0 to 1, default 0).
    pub lattice_geometry_type: usize,
}

impl Default for QuantumAcousticSpinLiquidParams {
    fn default() -> Self {
        Self {
            heisenberg_exchange_coupling_mhz: 55.0,
            frustration_ratio_j2_j1: 0.28,
            spinon_phonon_coupling_mhz: 8.5,
            chiral_three_spin_scalar_chirality: 4.2,
            kagome_plaquette_count: 24,
            acoustic_driving_frequency_ghz: 4.6,
            cryogenic_temperature_mk: 10.0,
            lattice_geometry_type: 0,
        }
    }
}

impl QuantumAcousticSpinLiquidParams {
    /// Creates a new parameter configuration with rigorous physical boundary clamping.
    pub fn new(
        heisenberg_exchange_coupling_mhz: f64,
        frustration_ratio_j2_j1: f64,
        spinon_phonon_coupling_mhz: f64,
        chiral_three_spin_scalar_chirality: f64,
        kagome_plaquette_count: usize,
        acoustic_driving_frequency_ghz: f64,
        cryogenic_temperature_mk: f64,
        lattice_geometry_type: usize,
    ) -> Self {
        Self {
            heisenberg_exchange_coupling_mhz: heisenberg_exchange_coupling_mhz.clamp(10.0, 150.0),
            frustration_ratio_j2_j1: frustration_ratio_j2_j1.clamp(0.05, 0.60),
            spinon_phonon_coupling_mhz: spinon_phonon_coupling_mhz.clamp(1.0, 30.0),
            chiral_three_spin_scalar_chirality: chiral_three_spin_scalar_chirality.clamp(0.5, 20.0),
            kagome_plaquette_count: kagome_plaquette_count.clamp(8, 64),
            acoustic_driving_frequency_ghz: acoustic_driving_frequency_ghz.clamp(1.0, 12.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            lattice_geometry_type: lattice_geometry_type.clamp(0, 1),
        }
    }
}

/// Multi-physics evaluation metrics for non-Abelian quantum acoustic fractional spin liquids
/// and topological resonating valence bond networks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuantumAcousticSpinLiquidMetrics {
    /// Resonant spinon excitation fidelity (target >= 0.9960).
    pub spinon_excitation_fidelity: f64,
    /// Topological entanglement entropy S_topo = gamma (target >= 0.6793).
    pub topological_entanglement_entropy: f64,
    /// Topological entanglement entropy extraction error |Delta S_topo| (target <= 0.0020).
    pub topological_entropy_error: f64,
    /// Spin-mechanical crosstalk isolation ratio in dB (target >= 44.0 dB).
    pub spin_mechanical_crosstalk_isolation_db: f64,
    /// Ground-state topological degeneracy protection in dB (target >= 40.0 dB).
    pub ground_state_degeneracy_protection_db: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
