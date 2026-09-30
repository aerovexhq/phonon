#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for quantum acoustic
//! non-Abelian chiral topological skyrmion-vortex polariton networks and non-Clifford
//! geometric braiding engines.

/// Physical parameter configuration for quantum acoustic non-Abelian chiral topological
/// skyrmion-vortex polariton networks and non-Clifford geometric braiding engines.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkyrmionVortexPolaritonParams {
    /// Interfacial Dzyaloshinskii-Moriya interaction energy in meV (clamp 1.0 to 35.0, default 16.5).
    pub dzyaloshinskii_moriya_interaction_mev: f64,
    /// Superconducting vortex order parameter pairing gap in meV (clamp 2.0 to 40.0, default 20.0).
    pub superconducting_vortex_gap_mev: f64,
    /// Coherent acoustic drive frequency in GHz (clamp 1.0 to 12.0, default 5.6).
    pub acoustic_drive_frequency_ghz: f64,
    /// Acoustic surface wave skyrmion shuttling velocity in m/s (clamp 200.0 to 3000.0, default 1300.0).
    pub skyrmion_shuttling_velocity_m_per_s: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave drive power for polariton state circulation in micro-watts (clamp 0.5 to 30.0, default 5.4).
    pub microwave_drive_power_uw: f64,
    /// Composite skyrmion-vortex polariton core radius in nanometers (clamp 15.0 to 160.0, default 50.0).
    pub polariton_core_radius_nm: f64,
    /// Perpendicular magnetic anisotropy energy in meV (clamp 0.5 to 25.0, default 8.5).
    pub magnetic_anisotropy_energy_mev: f64,
}

impl Default for SkyrmionVortexPolaritonParams {
    fn default() -> Self {
        Self {
            dzyaloshinskii_moriya_interaction_mev: 16.5,
            superconducting_vortex_gap_mev: 20.0,
            acoustic_drive_frequency_ghz: 5.6,
            skyrmion_shuttling_velocity_m_per_s: 1300.0,
            cryogenic_temperature_mk: 10.0,
            microwave_drive_power_uw: 5.4,
            polariton_core_radius_nm: 50.0,
            magnetic_anisotropy_energy_mev: 8.5,
        }
    }
}

impl SkyrmionVortexPolaritonParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        dzyaloshinskii_moriya_interaction_mev: f64,
        superconducting_vortex_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        skyrmion_shuttling_velocity_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_drive_power_uw: f64,
        polariton_core_radius_nm: f64,
        magnetic_anisotropy_energy_mev: f64,
    ) -> Self {
        Self {
            dzyaloshinskii_moriya_interaction_mev: dzyaloshinskii_moriya_interaction_mev
                .clamp(1.0, 35.0),
            superconducting_vortex_gap_mev: superconducting_vortex_gap_mev.clamp(2.0, 40.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            skyrmion_shuttling_velocity_m_per_s: skyrmion_shuttling_velocity_m_per_s
                .clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_drive_power_uw: microwave_drive_power_uw.clamp(0.5, 30.0),
            polariton_core_radius_nm: polariton_core_radius_nm.clamp(15.0, 160.0),
            magnetic_anisotropy_energy_mev: magnetic_anisotropy_energy_mev.clamp(0.5, 25.0),
        }
    }
}

/// Multi-physics evaluation metrics for quantum acoustic non-Abelian chiral topological
/// skyrmion-vortex polariton networks and non-Clifford geometric braiding engines.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkyrmionVortexPolaritonMetrics {
    /// Quantum acoustic non-Abelian non-Clifford braiding gate fidelity (target >= 0.9980).
    pub gate_fidelity: f64,
    /// Skyrmion-vortex composite polariton state retention fraction (target >= 0.9970).
    pub polariton_state_retention_fraction: f64,
    /// Topological protection energy gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-polariton crosstalk acoustic isolation in decibels (target >= 54.0 dB).
    pub inter_polariton_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
