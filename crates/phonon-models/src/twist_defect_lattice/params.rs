#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for quantum acoustic
//! non-Abelian chiral topological twist-defect Majorana braiding lattices and gauge-invariant
//! state teleporters.

/// Physical parameter configuration for quantum acoustic non-Abelian chiral topological
/// twist-defect Majorana braiding lattices and gauge-invariant state teleporters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TwistDefectLatticeParams {
    /// Dislocation Burgers vector magnitude in nanometers (clamp 0.2 to 5.0, default 1.8).
    pub dislocation_burgers_vector_nm: f64,
    /// Screw dislocation twist angle in radians (clamp 0.05 to 0.80, default 0.35).
    pub screw_twist_angle_rad: f64,
    /// Topological superconducting pairing gap in meV (clamp 2.0 to 45.0, default 22.0).
    pub topological_pairing_gap_mev: f64,
    /// Coherent acoustic drive frequency in GHz (clamp 1.0 to 12.0, default 5.8).
    pub acoustic_drive_frequency_ghz: f64,
    /// Piezo-acoustic strain shuttling speed in m/s (clamp 200.0 to 3000.0, default 1400.0).
    pub strain_shuttling_speed_m_per_s: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave control power for gauge flux shuttling in micro-watts (clamp 0.5 to 30.0, default 6.2).
    pub microwave_control_power_uw: f64,
    /// Separation distance between adjacent twist defects in micrometers (clamp 0.5 to 15.0, default 4.5).
    pub defect_separation_um: f64,
}

impl Default for TwistDefectLatticeParams {
    fn default() -> Self {
        Self {
            dislocation_burgers_vector_nm: 1.8,
            screw_twist_angle_rad: 0.35,
            topological_pairing_gap_mev: 22.0,
            acoustic_drive_frequency_ghz: 5.8,
            strain_shuttling_speed_m_per_s: 1400.0,
            cryogenic_temperature_mk: 10.0,
            microwave_control_power_uw: 6.2,
            defect_separation_um: 4.5,
        }
    }
}

impl TwistDefectLatticeParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        dislocation_burgers_vector_nm: f64,
        screw_twist_angle_rad: f64,
        topological_pairing_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        strain_shuttling_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_control_power_uw: f64,
        defect_separation_um: f64,
    ) -> Self {
        Self {
            dislocation_burgers_vector_nm: dislocation_burgers_vector_nm.clamp(0.2, 5.0),
            screw_twist_angle_rad: screw_twist_angle_rad.clamp(0.05, 0.80),
            topological_pairing_gap_mev: topological_pairing_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            strain_shuttling_speed_m_per_s: strain_shuttling_speed_m_per_s.clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_control_power_uw: microwave_control_power_uw.clamp(0.5, 30.0),
            defect_separation_um: defect_separation_um.clamp(0.5, 15.0),
        }
    }
}

/// Multi-physics evaluation metrics for quantum acoustic non-Abelian chiral topological
/// twist-defect Majorana braiding lattices and gauge-invariant state teleporters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TwistDefectLatticeMetrics {
    /// Quantum acoustic non-Abelian state teleportation fidelity across non-local channels (target >= 0.9980).
    pub teleportation_fidelity: f64,
    /// Twist-defect bound Majorana state retention fraction (target >= 0.9970).
    pub twist_defect_retention_fraction: f64,
    /// Topological protection energy gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-defect crosstalk acoustic isolation in decibels (target >= 55.0 dB).
    pub inter_defect_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
