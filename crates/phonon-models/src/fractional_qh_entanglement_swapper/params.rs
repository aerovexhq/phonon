#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for quantum acoustic
//! topological chiral fractional quantum Hall phonon entanglement swappers and non-Abelian
//! anyon teleportation bridges.

/// Physical parameter configuration for quantum acoustic topological chiral fractional
/// quantum Hall phonon entanglement swappers and non-Abelian anyon teleportation bridges.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FractionalQHEntanglementSwapperParams {
    /// Fractional quantum Hall filling factor nu (clamp 0.2 to 2.5, default 0.3333).
    pub filling_factor_nu: f64,
    /// Topological tunneling amplitude in meV (clamp 2.0 to 40.0, default 18.0).
    pub topological_tunneling_amplitude_mev: f64,
    /// Acoustic edge velocity in meters per second (clamp 500.0 to 4500.0, default 2100.0).
    pub acoustic_edge_velocity_m_per_s: f64,
    /// Anyon shuttling distance in micrometers (clamp 0.5 to 25.0, default 4.2).
    pub anyon_shuttling_distance_um: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave drive power in micro-watts (clamp 0.5 to 30.0, default 6.5).
    pub microwave_drive_power_uw: f64,
    /// Heterostructure relative dielectric constant (clamp 8.0 to 25.0, default 13.1).
    pub heterostructure_dielectric_constant: f64,
    /// Chiral edge channel separation in micrometers (clamp 0.2 to 10.0, default 1.8).
    pub channel_separation_um: f64,
}

impl Default for FractionalQHEntanglementSwapperParams {
    fn default() -> Self {
        Self {
            filling_factor_nu: 0.3333,
            topological_tunneling_amplitude_mev: 18.0,
            acoustic_edge_velocity_m_per_s: 2100.0,
            anyon_shuttling_distance_um: 4.2,
            cryogenic_temperature_mk: 10.0,
            microwave_drive_power_uw: 6.5,
            heterostructure_dielectric_constant: 13.1,
            channel_separation_um: 1.8,
        }
    }
}

impl FractionalQHEntanglementSwapperParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        filling_factor_nu: f64,
        topological_tunneling_amplitude_mev: f64,
        acoustic_edge_velocity_m_per_s: f64,
        anyon_shuttling_distance_um: f64,
        cryogenic_temperature_mk: f64,
        microwave_drive_power_uw: f64,
        heterostructure_dielectric_constant: f64,
        channel_separation_um: f64,
    ) -> Self {
        Self {
            filling_factor_nu: filling_factor_nu.clamp(0.2, 2.5),
            topological_tunneling_amplitude_mev: topological_tunneling_amplitude_mev.clamp(2.0, 40.0),
            acoustic_edge_velocity_m_per_s: acoustic_edge_velocity_m_per_s.clamp(500.0, 4500.0),
            anyon_shuttling_distance_um: anyon_shuttling_distance_um.clamp(0.5, 25.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_drive_power_uw: microwave_drive_power_uw.clamp(0.5, 30.0),
            heterostructure_dielectric_constant: heterostructure_dielectric_constant.clamp(8.0, 25.0),
            channel_separation_um: channel_separation_um.clamp(0.2, 10.0),
        }
    }
}

/// Multi-physics evaluation metrics for quantum acoustic topological chiral fractional
/// quantum Hall phonon entanglement swappers and non-Abelian anyon teleportation bridges.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FractionalQHEntanglementSwapperMetrics {
    /// Bell state measurement fidelity (target >= 0.9980).
    pub bell_state_measurement_fidelity: f64,
    /// Entanglement teleportation fidelity (target >= 0.9980).
    pub entanglement_teleportation_fidelity: f64,
    /// Topological protection gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-channel acoustic and electromagnetic crosstalk isolation in decibels (target >= 55.0 dB).
    pub inter_channel_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
