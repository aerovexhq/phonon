#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for quantum acoustic
//! non-Abelian chiral topological axion string-vortex entanglement networks and chiral
//! gauge-symmetric quantum memristors.

/// Physical parameter configuration for quantum acoustic non-Abelian chiral topological
/// axion string-vortex entanglement networks and chiral gauge-symmetric quantum memristors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxionStringMemristorParams {
    /// Axion coupling constant in meV (clamp 1.0 to 35.0, default 16.8).
    pub axion_coupling_constant_mev: f64,
    /// Superconducting vortex pairing gap in meV (clamp 2.0 to 45.0, default 22.5).
    pub superconducting_vortex_gap_mev: f64,
    /// Acoustic drive frequency in GHz (clamp 1.0 to 12.0, default 5.7).
    pub acoustic_drive_frequency_ghz: f64,
    /// Dynamical axion angle theta in radians (clamp 0.1 to 3.14, default 1.57).
    pub dynamical_axion_angle_rad: f64,
    /// Strain modulation velocity in m/s (clamp 200.0 to 3000.0, default 1450.0).
    pub strain_modulation_velocity_m_per_s: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave write pulse power in micro-watts (clamp 0.5 to 30.0, default 6.0).
    pub microwave_write_power_uw: f64,
    /// Axion string network density in string defects per um^2 (clamp 0.1 to 10.0, default 3.2).
    pub string_network_density_um2: f64,
}

impl Default for AxionStringMemristorParams {
    fn default() -> Self {
        Self {
            axion_coupling_constant_mev: 16.8,
            superconducting_vortex_gap_mev: 22.5,
            acoustic_drive_frequency_ghz: 5.7,
            dynamical_axion_angle_rad: 1.57,
            strain_modulation_velocity_m_per_s: 1450.0,
            cryogenic_temperature_mk: 10.0,
            microwave_write_power_uw: 6.0,
            string_network_density_um2: 3.2,
        }
    }
}

impl AxionStringMemristorParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        axion_coupling_constant_mev: f64,
        superconducting_vortex_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        dynamical_axion_angle_rad: f64,
        strain_modulation_velocity_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_write_power_uw: f64,
        string_network_density_um2: f64,
    ) -> Self {
        Self {
            axion_coupling_constant_mev: axion_coupling_constant_mev.clamp(1.0, 35.0),
            superconducting_vortex_gap_mev: superconducting_vortex_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            dynamical_axion_angle_rad: dynamical_axion_angle_rad.clamp(0.1, 3.14),
            strain_modulation_velocity_m_per_s: strain_modulation_velocity_m_per_s.clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_write_power_uw: microwave_write_power_uw.clamp(0.5, 30.0),
            string_network_density_um2: string_network_density_um2.clamp(0.1, 10.0),
        }
    }
}

/// Multi-physics evaluation metrics for quantum acoustic non-Abelian chiral topological
/// axion string-vortex entanglement networks and chiral gauge-symmetric quantum memristors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxionStringMemristorMetrics {
    /// Chiral gauge-symmetric quantum memristive retention fidelity (target >= 0.9980).
    pub memristive_retention_fidelity: f64,
    /// Axion string-vortex bound state retention fraction (target >= 0.9970).
    pub string_vortex_state_retention_fraction: f64,
    /// Topological protection energy gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-string crosstalk acoustic isolation in decibels (target >= 54.0 dB).
    pub inter_string_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
