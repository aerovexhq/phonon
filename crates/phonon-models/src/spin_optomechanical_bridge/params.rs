#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for quantum acoustic
//! non-Abelian chiral topological quantum error-mitigated spin-optomechanical teleportation bridges.

/// Physical parameter configuration for quantum acoustic non-Abelian chiral topological
/// quantum error-mitigated spin-optomechanical teleportation bridges.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpinOptomechanicalBridgeParams {
    /// Spin-optomechanical coupling energy in meV (clamp 1.0 to 35.0, default 16.5).
    pub spin_optomechanical_coupling_mev: f64,
    /// Topological teleportation energy gap in meV (clamp 2.0 to 45.0, default 22.0).
    pub topological_teleportation_gap_mev: f64,
    /// Acoustic drive frequency in GHz (clamp 1.0 to 12.0, default 5.8).
    pub acoustic_drive_frequency_ghz: f64,
    /// Teleportation drift speed in m/s (clamp 200.0 to 3000.0, default 1400.0).
    pub teleportation_drift_speed_m_per_s: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Optical entanglement pump power in microwatts (clamp 0.5 to 30.0, default 5.8).
    pub optical_entanglement_pump_power_uw: f64,
    /// Quantum error mitigation order in dimensionless units (clamp 1.0 to 8.0, default 4.0).
    pub error_mitigation_order: f64,
    /// Bridge channel pitch in micrometers (clamp 0.5 to 20.0, default 4.8).
    pub bridge_channel_pitch_um: f64,
}

impl Default for SpinOptomechanicalBridgeParams {
    fn default() -> Self {
        Self {
            spin_optomechanical_coupling_mev: 16.5,
            topological_teleportation_gap_mev: 22.0,
            acoustic_drive_frequency_ghz: 5.8,
            teleportation_drift_speed_m_per_s: 1400.0,
            cryogenic_temperature_mk: 10.0,
            optical_entanglement_pump_power_uw: 5.8,
            error_mitigation_order: 4.0,
            bridge_channel_pitch_um: 4.8,
        }
    }
}

impl SpinOptomechanicalBridgeParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        spin_optomechanical_coupling_mev: f64,
        topological_teleportation_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        teleportation_drift_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        optical_entanglement_pump_power_uw: f64,
        error_mitigation_order: f64,
        bridge_channel_pitch_um: f64,
    ) -> Self {
        Self {
            spin_optomechanical_coupling_mev: spin_optomechanical_coupling_mev.clamp(1.0, 35.0),
            topological_teleportation_gap_mev: topological_teleportation_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            teleportation_drift_speed_m_per_s: teleportation_drift_speed_m_per_s.clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            optical_entanglement_pump_power_uw: optical_entanglement_pump_power_uw.clamp(0.5, 30.0),
            error_mitigation_order: error_mitigation_order.clamp(1.0, 8.0),
            bridge_channel_pitch_um: bridge_channel_pitch_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for quantum acoustic non-Abelian chiral topological
/// quantum error-mitigated spin-optomechanical teleportation bridges.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpinOptomechanicalBridgeMetrics {
    /// Teleportation fidelity across the spin-optomechanical bridge (target >= 0.9980).
    pub teleportation_fidelity: f64,
    /// Spin quantum state retention fraction (target >= 0.9970).
    pub spin_state_retention_fraction: f64,
    /// Topological protection energy gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-channel crosstalk acoustic isolation in decibels (target >= 55.0 dB).
    pub inter_channel_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
