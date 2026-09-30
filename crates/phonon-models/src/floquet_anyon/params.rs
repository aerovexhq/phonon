#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Floquet-Engineered Non-Abelian Anyon
//! Weaving Fabric & Fractional Quantum Hall Acoustic Engine.

/// Physical parameter configuration for the universal multi-scale visual studio
/// autonomous Floquet-engineered non-Abelian anyon weaving fabric and fractional
/// quantum Hall acoustic engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloquetAnyonParams {
    /// Periodic Floquet drive coupling energy in meV (clamp 1.0 to 35.0, default 24.0).
    pub floquet_coupling_mev: f64,
    /// Topological braiding bandgap energy in meV (clamp 2.0 to 45.0, default 30.0).
    pub topological_braiding_gap_mev: f64,
    /// Acoustic drive carrier frequency in GHz (clamp 1.0 to 12.0, default 9.5).
    pub acoustic_drive_frequency_ghz: f64,
    /// Non-Abelian anyon weaving dispatch speed in m/s (clamp 200.0 to 3000.0, default 2100.0).
    pub weaving_dispatch_speed_m_per_s: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Dispersive microwave readout probe power in microwatts (clamp 0.5 to 30.0, default 9.5).
    pub microwave_probe_power_uw: f64,
    /// Synthetic flux quantum multiplication factor (clamp 1.0 to 8.0, default 4.0).
    pub synthetic_flux_quantum_factor: f64,
    /// Anyon weaving lattice pitch in micrometers (clamp 0.5 to 20.0, default 8.5).
    pub anyon_lattice_pitch_um: f64,
}

impl Default for FloquetAnyonParams {
    fn default() -> Self {
        Self {
            floquet_coupling_mev: 24.0,
            topological_braiding_gap_mev: 30.0,
            acoustic_drive_frequency_ghz: 9.5,
            weaving_dispatch_speed_m_per_s: 2100.0,
            cryogenic_temperature_mk: 10.0,
            microwave_probe_power_uw: 9.5,
            synthetic_flux_quantum_factor: 4.0,
            anyon_lattice_pitch_um: 8.5,
        }
    }
}

impl FloquetAnyonParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        floquet_coupling_mev: f64,
        topological_braiding_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        weaving_dispatch_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_probe_power_uw: f64,
        synthetic_flux_quantum_factor: f64,
        anyon_lattice_pitch_um: f64,
    ) -> Self {
        Self {
            floquet_coupling_mev: floquet_coupling_mev.clamp(1.0, 35.0),
            topological_braiding_gap_mev: topological_braiding_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            weaving_dispatch_speed_m_per_s: weaving_dispatch_speed_m_per_s.clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_probe_power_uw: microwave_probe_power_uw.clamp(0.5, 30.0),
            synthetic_flux_quantum_factor: synthetic_flux_quantum_factor.clamp(1.0, 8.0),
            anyon_lattice_pitch_um: anyon_lattice_pitch_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for the Phonon Universal Multi-Scale Visual Studio
/// Autonomous Floquet-Engineered Non-Abelian Anyon Weaving Fabric & Fractional Quantum Hall Acoustic Engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloquetAnyonMetrics {
    /// Non-Abelian anyon braiding gate fidelity (target >= 0.9980).
    pub braiding_fidelity: f64,
    /// Anyonic topological state quantum retention fraction (target >= 0.9970).
    pub anyonic_state_retention_fraction: f64,
    /// Multi-physics topological protection gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-braid crosstalk isolation in decibels (target >= 55.0 dB).
    pub inter_braid_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
