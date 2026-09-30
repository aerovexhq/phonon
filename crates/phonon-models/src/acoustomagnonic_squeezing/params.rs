#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Cavity Acoustomagnonic Squeezing &
//! Quantum Entangled Spin-Phonon Comb Engine.

/// Physical parameter configuration for the universal multi-scale visual studio
/// autonomous cavity acoustomagnonic squeezing and quantum entangled spin-phonon comb engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcoustomagnonicSqueezingParams {
    /// Acoustomagnonic squeezing coupling energy in meV (clamp 1.0 to 35.0, default 26.5).
    pub squeezing_coupling_mev: f64,
    /// Topological magnon gap energy in meV (clamp 2.0 to 45.0, default 32.5).
    pub topological_magnon_gap_mev: f64,
    /// Acoustic drive frequency in GHz (clamp 1.0 to 12.0, default 10.8).
    pub acoustic_drive_frequency_ghz: f64,
    /// Entanglement state dispatch speed in m/s (clamp 200.0 to 3000.0, default 2350.0).
    pub entanglement_dispatch_speed_m_per_s: f64,
    /// Cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave probe power in microwatts (clamp 0.5 to 30.0, default 10.8).
    pub microwave_probe_power_uw: f64,
    /// Synthetic squeezing modes factor (clamp 1.0 to 8.0, default 4.0).
    pub synthetic_squeezing_modes_factor: f64,
    /// Cavity pitch in micrometers (clamp 0.5 to 20.0, default 9.8).
    pub cavity_pitch_um: f64,
}

impl Default for AcoustomagnonicSqueezingParams {
    fn default() -> Self {
        Self {
            squeezing_coupling_mev: 26.5,
            topological_magnon_gap_mev: 32.5,
            acoustic_drive_frequency_ghz: 10.8,
            entanglement_dispatch_speed_m_per_s: 2350.0,
            cryogenic_temperature_mk: 10.0,
            microwave_probe_power_uw: 10.8,
            synthetic_squeezing_modes_factor: 4.0,
            cavity_pitch_um: 9.8,
        }
    }
}

impl AcoustomagnonicSqueezingParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        squeezing_coupling_mev: f64,
        topological_magnon_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        entanglement_dispatch_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_probe_power_uw: f64,
        synthetic_squeezing_modes_factor: f64,
        cavity_pitch_um: f64,
    ) -> Self {
        Self {
            squeezing_coupling_mev: squeezing_coupling_mev.clamp(1.0, 35.0),
            topological_magnon_gap_mev: topological_magnon_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            entanglement_dispatch_speed_m_per_s: entanglement_dispatch_speed_m_per_s.clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_probe_power_uw: microwave_probe_power_uw.clamp(0.5, 30.0),
            synthetic_squeezing_modes_factor: synthetic_squeezing_modes_factor.clamp(1.0, 8.0),
            cavity_pitch_um: cavity_pitch_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for the Phonon Universal Multi-Scale Visual Studio
/// Autonomous Cavity Acoustomagnonic Squeezing & Quantum Entangled Spin-Phonon Comb Engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcoustomagnonicSqueezingMetrics {
    /// Acoustomagnonic squeezing fidelity (target >= 0.9980).
    pub squeezing_fidelity: f64,
    /// Quantum entanglement state retention fraction (target >= 0.9970).
    pub entanglement_state_retention_fraction: f64,
    /// Topological protection gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-mode crosstalk isolation in decibels (target >= 55.0 dB).
    pub inter_mode_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
