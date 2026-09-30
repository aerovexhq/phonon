#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Mediated Spin-Valley Polariton Multiplexer &
//! 2D Valleytronics Engine.

/// Physical parameter configuration for the universal multi-scale visual studio
/// autonomous acoustically mediated spin-valley polariton multiplexer and 2D valleytronics engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpinValleyParams {
    /// Valley-orbit exchange coupling energy in meV (clamp 1.0 to 35.0, default 25.5).
    pub valley_coupling_mev: f64,
    /// Topological valley Hall bandgap energy in meV (clamp 2.0 to 45.0, default 31.5).
    pub topological_valley_gap_mev: f64,
    /// Surface acoustic wave drive frequency in GHz (clamp 1.0 to 12.0, default 10.2).
    pub acoustic_drive_frequency_ghz: f64,
    /// Chiral valley polariton dispatch speed in m/s (clamp 200.0 to 3000.0, default 2250.0).
    pub polariton_dispatch_speed_m_per_s: f64,
    /// Cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave valley probe power in microwatts (clamp 0.5 to 30.0, default 10.2).
    pub microwave_probe_power_uw: f64,
    /// Synthetic transition-metal dichalcogenide valley layers factor (clamp 1.0 to 8.0, default 4.0).
    pub synthetic_valley_layers_factor: f64,
    /// Acoustic valley multiplexer pitch in micrometers (clamp 0.5 to 20.0, default 9.2).
    pub multiplexer_pitch_um: f64,
}

impl Default for SpinValleyParams {
    fn default() -> Self {
        Self {
            valley_coupling_mev: 25.5,
            topological_valley_gap_mev: 31.5,
            acoustic_drive_frequency_ghz: 10.2,
            polariton_dispatch_speed_m_per_s: 2250.0,
            cryogenic_temperature_mk: 10.0,
            microwave_probe_power_uw: 10.2,
            synthetic_valley_layers_factor: 4.0,
            multiplexer_pitch_um: 9.2,
        }
    }
}

impl SpinValleyParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        valley_coupling_mev: f64,
        topological_valley_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        polariton_dispatch_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_probe_power_uw: f64,
        synthetic_valley_layers_factor: f64,
        multiplexer_pitch_um: f64,
    ) -> Self {
        Self {
            valley_coupling_mev: valley_coupling_mev.clamp(1.0, 35.0),
            topological_valley_gap_mev: topological_valley_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            polariton_dispatch_speed_m_per_s: polariton_dispatch_speed_m_per_s.clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_probe_power_uw: microwave_probe_power_uw.clamp(0.5, 30.0),
            synthetic_valley_layers_factor: synthetic_valley_layers_factor.clamp(1.0, 8.0),
            multiplexer_pitch_um: multiplexer_pitch_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for the Phonon Universal Multi-Scale Visual Studio
/// Autonomous Acoustically Mediated Spin-Valley Polariton Multiplexer & 2D Valleytronics Engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpinValleyMetrics {
    /// Chiral valley polariton multiplexing fidelity (target >= 0.9980).
    pub multiplexing_fidelity: f64,
    /// Spin-valley polarization retention fraction (target >= 0.9970).
    pub valley_polarization_retention_fraction: f64,
    /// Topological valley Hall protection gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-valley crosstalk isolation in decibels (target >= 55.0 dB).
    pub inter_valley_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
