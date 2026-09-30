#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Photonic-Phononic Quantum Memory
//! Register and Non-Volatile Polariton Qubit Synthesizer.

/// Physical parameter configuration for the universal multi-scale visual studio
/// autonomous photonic-phononic quantum memory register and non-volatile polariton qubit synthesizer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolaritonQuantumMemoryParams {
    /// Memory coupling energy in meV (clamp 1.0 to 35.0, default 30.0).
    pub memory_coupling_mev: f64,
    /// Topological memory gap energy in meV (clamp 2.0 to 45.0, default 36.0).
    pub topological_memory_gap_mev: f64,
    /// Acoustic drive frequency in GHz (clamp 1.0 to 12.0, default 12.0).
    pub acoustic_drive_frequency_ghz: f64,
    /// Memory dispatch speed in m/s (clamp 200.0 to 3000.0, default 2700.0).
    pub memory_dispatch_speed_m_per_s: f64,
    /// Cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave probe power in microwatts (clamp 0.5 to 30.0, default 12.5).
    pub microwave_probe_power_uw: f64,
    /// Synthetic memory cells factor (clamp 1.0 to 8.0, default 4.0).
    pub synthetic_memory_cells_factor: f64,
    /// Memory cell pitch in micrometers (clamp 0.5 to 20.0, default 11.5).
    pub memory_cell_pitch_um: f64,
}

impl Default for PolaritonQuantumMemoryParams {
    fn default() -> Self {
        Self {
            memory_coupling_mev: 30.0,
            topological_memory_gap_mev: 36.0,
            acoustic_drive_frequency_ghz: 12.0,
            memory_dispatch_speed_m_per_s: 2700.0,
            cryogenic_temperature_mk: 10.0,
            microwave_probe_power_uw: 12.5,
            synthetic_memory_cells_factor: 4.0,
            memory_cell_pitch_um: 11.5,
        }
    }
}

impl PolaritonQuantumMemoryParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        memory_coupling_mev: f64,
        topological_memory_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        memory_dispatch_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_probe_power_uw: f64,
        synthetic_memory_cells_factor: f64,
        memory_cell_pitch_um: f64,
    ) -> Self {
        Self {
            memory_coupling_mev: memory_coupling_mev.clamp(1.0, 35.0),
            topological_memory_gap_mev: topological_memory_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            memory_dispatch_speed_m_per_s: memory_dispatch_speed_m_per_s.clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_probe_power_uw: microwave_probe_power_uw.clamp(0.5, 30.0),
            synthetic_memory_cells_factor: synthetic_memory_cells_factor.clamp(1.0, 8.0),
            memory_cell_pitch_um: memory_cell_pitch_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for the Phonon Universal Multi-Scale Visual Studio
/// Autonomous Photonic-Phononic Quantum Memory Register and Non-Volatile Polariton Qubit Synthesizer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolaritonQuantumMemoryMetrics {
    /// Quantum memory storage fidelity (target >= 0.9980).
    pub memory_storage_fidelity: f64,
    /// Polariton qubit state retention fraction (target >= 0.9970).
    pub polariton_qubit_retention_fraction: f64,
    /// Topological protection gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-cell crosstalk isolation in decibels (target >= 55.0 dB).
    pub inter_cell_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
