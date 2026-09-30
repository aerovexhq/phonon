#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for chiral acoustic
//! quantum Hall metamaterials and non-Abelian Moore-Read Pfaffian edge waveguide synthesizers.

/// Physical parameter configuration for chiral acoustic quantum Hall metamaterials
/// and non-Abelian Moore-Read Pfaffian edge waveguide synthesizers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChiralQuantumHallPfaffianParams {
    /// Perpendicular quantizing magnetic field in Tesla (clamp 2.0 to 18.0, default 8.5).
    pub magnetic_field_tesla: f64,
    /// Fractional quantum Hall filling factor nu (clamp 0.40 to 2.80, default 2.50).
    pub fractional_filling_factor: f64,
    /// Non-Abelian Moore-Read Pfaffian quasiparticle pairing gap in MHz (clamp 5.0 to 60.0, default 24.0).
    pub pfaffian_pairing_gap_mhz: f64,
    /// Piezoelectric acoustic coupling efficiency eta_piezo (clamp 0.50 to 0.99, default 0.92).
    pub piezoelectric_acoustic_coupling_efficiency: f64,
    /// Chiral edge waveguide channel length in micrometers (clamp 1.0 to 25.0, default 6.5).
    pub waveguide_channel_length_um: f64,
    /// Operating cryogenic temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Spatial inter-edge spacing between counter-propagating chiral boundaries in nanometers (clamp 50.0 to 500.0, default 180.0).
    pub inter_edge_spacing_nm: f64,
    /// Coherent acoustic driving frequency in GHz (clamp 1.0 to 12.0, default 4.2).
    pub acoustic_driving_frequency_ghz: f64,
}

impl Default for ChiralQuantumHallPfaffianParams {
    fn default() -> Self {
        Self {
            magnetic_field_tesla: 8.5,
            fractional_filling_factor: 2.50,
            pfaffian_pairing_gap_mhz: 24.0,
            piezoelectric_acoustic_coupling_efficiency: 0.92,
            waveguide_channel_length_um: 6.5,
            cryogenic_temperature_mk: 10.0,
            inter_edge_spacing_nm: 180.0,
            acoustic_driving_frequency_ghz: 4.2,
        }
    }
}

impl ChiralQuantumHallPfaffianParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        magnetic_field_tesla: f64,
        fractional_filling_factor: f64,
        pfaffian_pairing_gap_mhz: f64,
        piezoelectric_acoustic_coupling_efficiency: f64,
        waveguide_channel_length_um: f64,
        cryogenic_temperature_mk: f64,
        inter_edge_spacing_nm: f64,
        acoustic_driving_frequency_ghz: f64,
    ) -> Self {
        Self {
            magnetic_field_tesla: magnetic_field_tesla.clamp(2.0, 18.0),
            fractional_filling_factor: fractional_filling_factor.clamp(0.40, 2.80),
            pfaffian_pairing_gap_mhz: pfaffian_pairing_gap_mhz.clamp(5.0, 60.0),
            piezoelectric_acoustic_coupling_efficiency: piezoelectric_acoustic_coupling_efficiency
                .clamp(0.50, 0.99),
            waveguide_channel_length_um: waveguide_channel_length_um.clamp(1.0, 25.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            inter_edge_spacing_nm: inter_edge_spacing_nm.clamp(50.0, 500.0),
            acoustic_driving_frequency_ghz: acoustic_driving_frequency_ghz.clamp(1.0, 12.0),
        }
    }
}

/// Multi-physics evaluation metrics for chiral acoustic quantum Hall metamaterials
/// and non-Abelian Moore-Read Pfaffian edge waveguide synthesizers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChiralQuantumHallPfaffianMetrics {
    /// Topological state fidelity of the synthesized non-Abelian Pfaffian ground state (target >= 0.9970).
    pub pfaffian_topological_state_fidelity: f64,
    /// Chiral edge channel isolation against inter-edge backscattering in dB (target >= 46.0 dB).
    pub edge_channel_isolation_db: f64,
    /// Propagation velocity of the neutral chiral Majorana edge mode in m/s (target >= 1400.0 m/s).
    pub neutral_mode_transmission_speed_mps: f64,
    /// Normalized quantization error of the thermal Hall conductance relative to c=5/2 (target <= 0.0020).
    pub thermal_hall_quantization_error: f64,
    /// Quasiparticle braiding interferometric visibility in non-Abelian edge interferometers (target >= 0.985).
    pub quasiparticle_braiding_visibility: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
