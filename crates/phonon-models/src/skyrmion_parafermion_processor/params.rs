#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Driven Skyrmion-Parafermion
//! Topological Quantum Processor & Surface Code Hub Engine (Phase 290 Milestone).

/// Physical parameter configuration for the universal multi-scale visual studio
/// autonomous acoustically driven skyrmion-parafermion topological quantum processor
/// and surface code hub engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkyrmionParafermionProcessorParams {
    /// Processor coupling energy in meV (clamp 1.0 to 35.0, default 35.0).
    pub processor_coupling_mev: f64,
    /// Topological hybrid gap energy in meV (clamp 2.0 to 45.0, default 45.0).
    pub topological_hybrid_gap_mev: f64,
    /// Acoustic drive frequency in GHz (clamp 1.0 to 12.0, default 12.0).
    pub acoustic_drive_frequency_ghz: f64,
    /// Processor dispatch speed in m/s (clamp 200.0 to 3000.0, default 3000.0).
    pub processor_dispatch_speed_m_per_s: f64,
    /// Cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave probe power in microwatts (clamp 0.5 to 30.0, default 29.5).
    pub microwave_probe_power_uw: f64,
    /// Synthetic stabilizers factor (clamp 1.0 to 8.0, default 4.0).
    pub synthetic_stabilizers_factor: f64,
    /// Processor lattice pitch in micrometers (clamp 0.5 to 25.0, default 25.0).
    pub processor_lattice_pitch_um: f64,
}

impl Default for SkyrmionParafermionProcessorParams {
    fn default() -> Self {
        Self {
            processor_coupling_mev: 35.0,
            topological_hybrid_gap_mev: 45.0,
            acoustic_drive_frequency_ghz: 12.0,
            processor_dispatch_speed_m_per_s: 3000.0,
            cryogenic_temperature_mk: 10.0,
            microwave_probe_power_uw: 29.5,
            synthetic_stabilizers_factor: 4.0,
            processor_lattice_pitch_um: 25.0,
        }
    }
}

impl SkyrmionParafermionProcessorParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        processor_coupling_mev: f64,
        topological_hybrid_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        processor_dispatch_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_probe_power_uw: f64,
        synthetic_stabilizers_factor: f64,
        processor_lattice_pitch_um: f64,
    ) -> Self {
        Self {
            processor_coupling_mev: processor_coupling_mev.clamp(1.0, 35.0),
            topological_hybrid_gap_mev: topological_hybrid_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            processor_dispatch_speed_m_per_s: processor_dispatch_speed_m_per_s.clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_probe_power_uw: microwave_probe_power_uw.clamp(0.5, 30.0),
            synthetic_stabilizers_factor: synthetic_stabilizers_factor.clamp(1.0, 8.0),
            processor_lattice_pitch_um: processor_lattice_pitch_um.clamp(0.5, 25.0),
        }
    }
}

/// Multi-physics evaluation metrics for the Phonon Universal Multi-Scale Visual Studio
/// Autonomous Acoustically Driven Skyrmion-Parafermion Topological Quantum Processor &
/// Surface Code Hub Engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkyrmionParafermionProcessorMetrics {
    /// Processor fidelity (target >= 0.9980).
    pub processor_fidelity: f64,
    /// Topological state retention fraction (target >= 0.9970).
    pub topological_state_retention_fraction: f64,
    /// Topological protection gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-node crosstalk isolation in decibels (target >= 55.0 dB).
    pub inter_node_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
