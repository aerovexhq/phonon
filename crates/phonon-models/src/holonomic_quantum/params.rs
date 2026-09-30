#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Non-Abelian Holonomic Quantum Computing Gate Synthesizer &
//! Geometric Phase Engine.

/// Physical parameter configuration for the universal multi-scale visual studio
/// autonomous non-Abelian holonomic quantum computing gate synthesizer and geometric phase engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HolonomicQuantumParams {
    /// Non-Abelian holonomic coupling energy in meV (clamp 1.0 to 35.0, default 26.0).
    pub holonomic_coupling_mev: f64,
    /// Topological holonomic bandgap energy in meV (clamp 2.0 to 45.0, default 32.0).
    pub topological_holonomic_gap_mev: f64,
    /// Surface acoustic wave drive frequency in GHz (clamp 1.0 to 12.0, default 10.5).
    pub acoustic_drive_frequency_ghz: f64,
    /// Gate synthesis dispatch speed in m/s (clamp 200.0 to 3000.0, default 2300.0).
    pub gate_synthesis_dispatch_speed_m_per_s: f64,
    /// Cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave probe power in microwatts (clamp 0.5 to 30.0, default 10.5).
    pub microwave_probe_power_uw: f64,
    /// Synthetic holonomic loops factor (clamp 1.0 to 8.0, default 4.0).
    pub synthetic_holonomic_loops_factor: f64,
    /// Gate loop pitch in micrometers (clamp 0.5 to 20.0, default 9.5).
    pub gate_loop_pitch_um: f64,
}

impl Default for HolonomicQuantumParams {
    fn default() -> Self {
        Self {
            holonomic_coupling_mev: 26.0,
            topological_holonomic_gap_mev: 32.0,
            acoustic_drive_frequency_ghz: 10.5,
            gate_synthesis_dispatch_speed_m_per_s: 2300.0,
            cryogenic_temperature_mk: 10.0,
            microwave_probe_power_uw: 10.5,
            synthetic_holonomic_loops_factor: 4.0,
            gate_loop_pitch_um: 9.5,
        }
    }
}

impl HolonomicQuantumParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        holonomic_coupling_mev: f64,
        topological_holonomic_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        gate_synthesis_dispatch_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_probe_power_uw: f64,
        synthetic_holonomic_loops_factor: f64,
        gate_loop_pitch_um: f64,
    ) -> Self {
        Self {
            holonomic_coupling_mev: holonomic_coupling_mev.clamp(1.0, 35.0),
            topological_holonomic_gap_mev: topological_holonomic_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            gate_synthesis_dispatch_speed_m_per_s: gate_synthesis_dispatch_speed_m_per_s.clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_probe_power_uw: microwave_probe_power_uw.clamp(0.5, 30.0),
            synthetic_holonomic_loops_factor: synthetic_holonomic_loops_factor.clamp(1.0, 8.0),
            gate_loop_pitch_um: gate_loop_pitch_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for the Phonon Universal Multi-Scale Visual Studio
/// Autonomous Non-Abelian Holonomic Quantum Computing Gate Synthesizer & Geometric Phase Engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HolonomicQuantumMetrics {
    /// Holonomic gate synthesis fidelity (target >= 0.9980).
    pub gate_synthesis_fidelity: f64,
    /// Geometric phase retention fraction (target >= 0.9970).
    pub geometric_phase_retention_fraction: f64,
    /// Topological protection gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-gate crosstalk isolation in decibels (target >= 55.0 dB).
    pub inter_gate_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
