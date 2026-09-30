#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for the Phonon
//! Universal Multi-Scale Visual Studio Quantum Digital Twin Micro-Architecture
//! Simulator & Sub-System Co-Emulation Fabric.

/// Physical parameter configuration for the universal multi-scale visual studio
/// quantum digital twin micro-architecture simulator and sub-system co-emulation fabric.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuantumDigitalTwinParams {
    /// Co-emulation drive line coupling energy in meV (clamp 1.0 to 35.0, default 20.5).
    pub coemulation_coupling_mev: f64,
    /// Topological qubit emulation bandgap energy in meV (clamp 2.0 to 45.0, default 26.5).
    pub topological_emulation_gap_mev: f64,
    /// Acoustic drive carrier frequency in GHz (clamp 1.0 to 12.0, default 7.8).
    pub acoustic_drive_frequency_ghz: f64,
    /// Co-emulation instruction dispatch speed in m/s (clamp 200.0 to 3000.0, default 1750.0).
    pub instruction_dispatch_speed_m_per_s: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Dispersive microwave readout probe power in microwatts (clamp 0.5 to 30.0, default 7.8).
    pub microwave_probe_power_uw: f64,
    /// Synthetic co-emulation cores scaling factor (clamp 1.0 to 8.0, default 4.0).
    pub synthetic_coemulation_cores_factor: f64,
    /// Multi-core quantum bus interconnect spatial routing pitch in micrometers (clamp 0.5 to 20.0, default 6.8).
    pub bus_interconnect_pitch_um: f64,
}

impl Default for QuantumDigitalTwinParams {
    fn default() -> Self {
        Self {
            coemulation_coupling_mev: 20.5,
            topological_emulation_gap_mev: 26.5,
            acoustic_drive_frequency_ghz: 7.8,
            instruction_dispatch_speed_m_per_s: 1750.0,
            cryogenic_temperature_mk: 10.0,
            microwave_probe_power_uw: 7.8,
            synthetic_coemulation_cores_factor: 4.0,
            bus_interconnect_pitch_um: 6.8,
        }
    }
}

impl QuantumDigitalTwinParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        coemulation_coupling_mev: f64,
        topological_emulation_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        instruction_dispatch_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_probe_power_uw: f64,
        synthetic_coemulation_cores_factor: f64,
        bus_interconnect_pitch_um: f64,
    ) -> Self {
        Self {
            coemulation_coupling_mev: coemulation_coupling_mev.clamp(1.0, 35.0),
            topological_emulation_gap_mev: topological_emulation_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            instruction_dispatch_speed_m_per_s: instruction_dispatch_speed_m_per_s
                .clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_probe_power_uw: microwave_probe_power_uw.clamp(0.5, 30.0),
            synthetic_coemulation_cores_factor: synthetic_coemulation_cores_factor.clamp(1.0, 8.0),
            bus_interconnect_pitch_um: bus_interconnect_pitch_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for the Phonon Universal Multi-Scale Visual Studio
/// Quantum Digital Twin Micro-Architecture Simulator & Sub-System Co-Emulation Fabric.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuantumDigitalTwinMetrics {
    /// Co-emulation fidelity across simulated micro-architectures (target >= 0.9980).
    pub coemulation_fidelity: f64,
    /// Quantum bus state retention fraction across execution rollouts (target >= 0.9970).
    pub quantum_bus_state_retention_fraction: f64,
    /// Multi-physics topological protection gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-core crosstalk isolation in decibels (target >= 55.0 dB).
    pub inter_core_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
