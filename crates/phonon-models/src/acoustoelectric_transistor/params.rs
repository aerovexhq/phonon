#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Quantum Acoustoelectric Metamaterial
//! Transistor & Non-Reciprocal Microwave Isolator Engine.

/// Physical parameter configuration for the universal multi-scale visual studio
/// autonomous quantum acoustoelectric metamaterial transistor & non-reciprocal microwave isolator engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcoustoelectricTransistorParams {
    /// Acoustoelectric transistor coupling energy in meV (clamp 1.0 to 35.0, default 27.5).
    pub transistor_coupling_mev: f64,
    /// Topological isolation bandgap energy in meV (clamp 2.0 to 45.0, default 33.5).
    pub topological_isolation_gap_mev: f64,
    /// Acoustic drive frequency in GHz (clamp 1.0 to 12.0, default 11.2).
    pub acoustic_drive_frequency_ghz: f64,
    /// Acoustoelectric charge dispatch speed in m/s (clamp 200.0 to 3000.0, default 2450.0).
    pub charge_dispatch_speed_m_per_s: f64,
    /// Cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave probe power in microwatts (clamp 0.5 to 30.0, default 11.2).
    pub microwave_probe_power_uw: f64,
    /// Synthetic gate electrodes factor (clamp 1.0 to 8.0, default 4.0).
    pub synthetic_gate_electrodes_factor: f64,
    /// Transistor pitch in micrometers (clamp 0.5 to 20.0, default 10.2).
    pub transistor_pitch_um: f64,
}

impl Default for AcoustoelectricTransistorParams {
    fn default() -> Self {
        Self {
            transistor_coupling_mev: 27.5,
            topological_isolation_gap_mev: 33.5,
            acoustic_drive_frequency_ghz: 11.2,
            charge_dispatch_speed_m_per_s: 2450.0,
            cryogenic_temperature_mk: 10.0,
            microwave_probe_power_uw: 11.2,
            synthetic_gate_electrodes_factor: 4.0,
            transistor_pitch_um: 10.2,
        }
    }
}

impl AcoustoelectricTransistorParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        transistor_coupling_mev: f64,
        topological_isolation_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        charge_dispatch_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_probe_power_uw: f64,
        synthetic_gate_electrodes_factor: f64,
        transistor_pitch_um: f64,
    ) -> Self {
        Self {
            transistor_coupling_mev: transistor_coupling_mev.clamp(1.0, 35.0),
            topological_isolation_gap_mev: topological_isolation_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            charge_dispatch_speed_m_per_s: charge_dispatch_speed_m_per_s.clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_probe_power_uw: microwave_probe_power_uw.clamp(0.5, 30.0),
            synthetic_gate_electrodes_factor: synthetic_gate_electrodes_factor.clamp(1.0, 8.0),
            transistor_pitch_um: transistor_pitch_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for the Phonon Universal Multi-Scale Visual Studio
/// Autonomous Quantum Acoustoelectric Metamaterial Transistor & Non-Reciprocal Microwave Isolator Engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcoustoelectricTransistorMetrics {
    /// Transistor switching fidelity (target >= 0.9980).
    pub switching_fidelity: f64,
    /// Acoustoelectric charge state retention fraction (target >= 0.9970).
    pub charge_state_retention_fraction: f64,
    /// Topological protection gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-gate crosstalk isolation in decibels (target >= 55.0 dB).
    pub inter_gate_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
