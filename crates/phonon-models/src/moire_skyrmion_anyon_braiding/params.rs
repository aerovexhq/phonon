#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for non-Abelian quantum
//! acoustic anyonic braiding in moire skyrmion crystals and chiral topological spin-Peierls
//! transducers.

/// Physical parameter configuration for non-Abelian quantum acoustic anyonic braiding
/// in moire skyrmion crystals and chiral topological spin-Peierls transducers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MoireSkyrmionAnyonBraidingParams {
    /// Superlattice twist angle in degrees (clamp 0.80 to 2.50, default 1.25).
    pub twist_angle_degrees: f64,
    /// Chiral spin-Peierls acoustic phonon coupling constant (clamp 0.10 to 0.95, default 0.65).
    pub spin_peierls_coupling_constant: f64,
    /// Interfacial Dzyaloshinskii-Moriya interaction in meV (clamp 1.0 to 15.0, default 5.8).
    pub dzyaloshinskii_moriya_interaction_mev: f64,
    /// Heisenberg nearest-neighbor exchange coupling J in meV (clamp 5.0 to 40.0, default 18.0).
    pub heisenberg_exchange_coupling_j_mev: f64,
    /// Surface acoustic wave (SAW) drive frequency in GHz (clamp 1.0 to 15.0, default 4.5).
    pub surface_acoustic_wave_frequency_ghz: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 12.0).
    pub cryogenic_temperature_mk: f64,
    /// Inter-skyrmion lattice pitch in nanometers (clamp 30.0 to 300.0, default 110.0).
    pub inter_skyrmion_pitch_nm: f64,
    /// Anyon adiabatic braiding path length in micrometers (clamp 0.5 to 8.0, default 2.2).
    pub braiding_path_length_um: f64,
}

impl Default for MoireSkyrmionAnyonBraidingParams {
    fn default() -> Self {
        Self {
            twist_angle_degrees: 1.25,
            spin_peierls_coupling_constant: 0.65,
            dzyaloshinskii_moriya_interaction_mev: 5.8,
            heisenberg_exchange_coupling_j_mev: 18.0,
            surface_acoustic_wave_frequency_ghz: 4.5,
            cryogenic_temperature_mk: 12.0,
            inter_skyrmion_pitch_nm: 110.0,
            braiding_path_length_um: 2.2,
        }
    }
}

impl MoireSkyrmionAnyonBraidingParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        twist_angle_degrees: f64,
        spin_peierls_coupling_constant: f64,
        dzyaloshinskii_moriya_interaction_mev: f64,
        heisenberg_exchange_coupling_j_mev: f64,
        surface_acoustic_wave_frequency_ghz: f64,
        cryogenic_temperature_mk: f64,
        inter_skyrmion_pitch_nm: f64,
        braiding_path_length_um: f64,
    ) -> Self {
        Self {
            twist_angle_degrees: twist_angle_degrees.clamp(0.80, 2.50),
            spin_peierls_coupling_constant: spin_peierls_coupling_constant.clamp(0.10, 0.95),
            dzyaloshinskii_moriya_interaction_mev: dzyaloshinskii_moriya_interaction_mev
                .clamp(1.0, 15.0),
            heisenberg_exchange_coupling_j_mev: heisenberg_exchange_coupling_j_mev
                .clamp(5.0, 40.0),
            surface_acoustic_wave_frequency_ghz: surface_acoustic_wave_frequency_ghz
                .clamp(1.0, 15.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            inter_skyrmion_pitch_nm: inter_skyrmion_pitch_nm.clamp(30.0, 300.0),
            braiding_path_length_um: braiding_path_length_um.clamp(0.5, 8.0),
        }
    }
}

/// Multi-physics evaluation metrics for non-Abelian quantum acoustic anyonic braiding
/// in moire skyrmion crystals and chiral topological spin-Peierls transducers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MoireSkyrmionAnyonBraidingMetrics {
    /// Anyonic braiding geometric phase fidelity (target >= 0.9980).
    pub anyonic_braiding_phase_fidelity: f64,
    /// Topological protection energy gap in MHz (target >= 42.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Skyrmion topological charge stability fraction (target >= 0.9970).
    pub skyrmion_topological_stability_fraction: f64,
    /// Inter-skyrmion acoustic crosstalk isolation in decibels (target >= 53.0 dB).
    pub inter_skyrmion_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 16.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
