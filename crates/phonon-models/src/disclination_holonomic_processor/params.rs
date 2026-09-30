#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for quantum acoustic
//! non-Abelian higher-order disclination bound states and chiral holonomic anyon processors.

/// Physical parameter configuration for quantum acoustic non-Abelian higher-order
/// disclination bound states and chiral holonomic anyon processors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DisclinationHolonomicProcessorParams {
    /// Frank angle of the hexagonal lattice disclination in radians (clamp 0.40 to 2.20, default 1.0472).
    pub disclination_frank_angle_rad: f64,
    /// Higher-order topological mass gap energy in meV (clamp 2.0 to 45.0, default 21.0).
    pub higher_order_topological_mass_mev: f64,
    /// Fundamental acoustic drive frequency in GHz (clamp 1.0 to 12.0, default 6.0).
    pub acoustic_drive_frequency_ghz: f64,
    /// Acoustic surface wave holonomic shuttling velocity in m/s (clamp 200.0 to 3000.0, default 1400.0).
    pub holonomic_shuttling_speed_m_per_s: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave control power for non-Abelian state synthesis in micro-watts (clamp 0.5 to 30.0, default 5.2).
    pub microwave_control_power_uw: f64,
    /// Physical core radius of the topological disclination defect in nanometers (clamp 10.0 to 180.0, default 45.0).
    pub disclination_core_radius_nm: f64,
    /// Synthetic hexagonal lattice strain parameter (clamp 0.01 to 0.25, default 0.08).
    pub lattice_hexagonal_strain: f64,
}

impl Default for DisclinationHolonomicProcessorParams {
    fn default() -> Self {
        Self {
            disclination_frank_angle_rad: 1.0472,
            higher_order_topological_mass_mev: 21.0,
            acoustic_drive_frequency_ghz: 6.0,
            holonomic_shuttling_speed_m_per_s: 1400.0,
            cryogenic_temperature_mk: 10.0,
            microwave_control_power_uw: 5.2,
            disclination_core_radius_nm: 45.0,
            lattice_hexagonal_strain: 0.08,
        }
    }
}

impl DisclinationHolonomicProcessorParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        disclination_frank_angle_rad: f64,
        higher_order_topological_mass_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        holonomic_shuttling_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_control_power_uw: f64,
        disclination_core_radius_nm: f64,
        lattice_hexagonal_strain: f64,
    ) -> Self {
        Self {
            disclination_frank_angle_rad: disclination_frank_angle_rad.clamp(0.40, 2.20),
            higher_order_topological_mass_mev: higher_order_topological_mass_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            holonomic_shuttling_speed_m_per_s: holonomic_shuttling_speed_m_per_s.clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_control_power_uw: microwave_control_power_uw.clamp(0.5, 30.0),
            disclination_core_radius_nm: disclination_core_radius_nm.clamp(10.0, 180.0),
            lattice_hexagonal_strain: lattice_hexagonal_strain.clamp(0.01, 0.25),
        }
    }
}

/// Multi-physics evaluation metrics for quantum acoustic non-Abelian higher-order
/// disclination bound states and chiral holonomic anyon processors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DisclinationHolonomicProcessorMetrics {
    /// Quantum acoustic non-Abelian holonomic gate fidelity (target >= 0.9980).
    pub holonomic_gate_fidelity: f64,
    /// Disclination fractional bound state retention fraction (target >= 0.9970).
    pub disclination_state_retention_fraction: f64,
    /// Topological protection energy gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-disclination crosstalk acoustic isolation in decibels (target >= 54.0 dB).
    pub inter_disclination_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
