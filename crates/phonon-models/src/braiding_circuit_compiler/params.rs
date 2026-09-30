#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for quantum acoustic
//! non-Abelian chiral topological anyon braiding circuit compilers and topological QASM synthesizers.

/// Physical parameter configuration for quantum acoustic non-Abelian chiral topological
/// anyon braiding circuit compilers and topological QASM synthesizers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BraidingCircuitCompilerParams {
    /// Compiler coupling energy in meV (clamp 1.0 to 35.0, default 16.5).
    pub compiler_coupling_energy_mev: f64,
    /// Topological braiding energy gap in meV (clamp 2.0 to 45.0, default 22.0).
    pub topological_braiding_gap_mev: f64,
    /// Acoustic drive frequency in GHz (clamp 1.0 to 12.0, default 5.8).
    pub acoustic_drive_frequency_ghz: f64,
    /// Braiding execution speed in m/s (clamp 200.0 to 3000.0, default 1400.0).
    pub braiding_execution_speed_m_per_s: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave synthesis power in microwatts (clamp 0.5 to 30.0, default 5.8).
    pub microwave_synthesis_power_uw: f64,
    /// Synthetic braid depth order in dimensionless units (clamp 1.0 to 8.0, default 4.0).
    pub synthetic_braid_depth_order: f64,
    /// Braiding channel pitch in micrometers (clamp 0.5 to 20.0, default 4.8).
    pub braiding_channel_pitch_um: f64,
}

impl Default for BraidingCircuitCompilerParams {
    fn default() -> Self {
        Self {
            compiler_coupling_energy_mev: 16.5,
            topological_braiding_gap_mev: 22.0,
            acoustic_drive_frequency_ghz: 5.8,
            braiding_execution_speed_m_per_s: 1400.0,
            cryogenic_temperature_mk: 10.0,
            microwave_synthesis_power_uw: 5.8,
            synthetic_braid_depth_order: 4.0,
            braiding_channel_pitch_um: 4.8,
        }
    }
}

impl BraidingCircuitCompilerParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        compiler_coupling_energy_mev: f64,
        topological_braiding_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        braiding_execution_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_synthesis_power_uw: f64,
        synthetic_braid_depth_order: f64,
        braiding_channel_pitch_um: f64,
    ) -> Self {
        Self {
            compiler_coupling_energy_mev: compiler_coupling_energy_mev.clamp(1.0, 35.0),
            topological_braiding_gap_mev: topological_braiding_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            braiding_execution_speed_m_per_s: braiding_execution_speed_m_per_s.clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_synthesis_power_uw: microwave_synthesis_power_uw.clamp(0.5, 30.0),
            synthetic_braid_depth_order: synthetic_braid_depth_order.clamp(1.0, 8.0),
            braiding_channel_pitch_um: braiding_channel_pitch_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for quantum acoustic non-Abelian chiral topological
/// anyon braiding circuit compilers and topological QASM synthesizers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BraidingCircuitCompilerMetrics {
    /// Compiling fidelity across the anyon braiding circuit compiler (target >= 0.9980).
    pub compiling_fidelity: f64,
    /// Braiding quantum state retention fraction (target >= 0.9970).
    pub braiding_state_retention_fraction: f64,
    /// Topological protection energy gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-channel crosstalk acoustic isolation in decibels (target >= 55.0 dB).
    pub inter_channel_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
