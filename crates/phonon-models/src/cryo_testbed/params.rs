#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for the Phonon
//! Universal Multi-Scale Visual Studio Full-Stack Hardware-in-the-Loop Cryogenic
//! Dilution Refrigerator Testbed Integration & Automated Qubit Calibration Engine.

/// Physical parameter configuration for the universal multi-scale visual studio
/// full-stack hardware-in-the-loop cryogenic dilution refrigerator testbed integration
/// and automated qubit calibration engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CryoTestbedParams {
    /// Cryogenic drive line coupling energy in meV (clamp 1.0 to 35.0, default 20.0).
    pub cryo_drive_coupling_mev: f64,
    /// Topological qubit calibration bandgap energy in meV (clamp 2.0 to 45.0, default 26.0).
    pub topological_calibration_gap_mev: f64,
    /// Acoustic drive carrier frequency in GHz (clamp 1.0 to 12.0, default 7.5).
    pub acoustic_drive_frequency_ghz: f64,
    /// Real-time pulse synthesis and dispatch speed in m/s (clamp 200.0 to 3000.0, default 1700.0).
    pub pulse_synthesis_dispatch_speed_m_per_s: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Dispersive microwave readout probe power in microwatts (clamp 0.5 to 30.0, default 7.5).
    pub microwave_probe_power_uw: f64,
    /// Synthetic hardware-in-the-loop (HIL) channels scaling factor (clamp 1.0 to 8.0, default 4.0).
    pub synthetic_hil_channels_factor: f64,
    /// Multi-qubit testbed spatial routing pitch in micrometers (clamp 0.5 to 20.0, default 6.5).
    pub testbed_pitch_um: f64,
}

impl Default for CryoTestbedParams {
    fn default() -> Self {
        Self {
            cryo_drive_coupling_mev: 20.0,
            topological_calibration_gap_mev: 26.0,
            acoustic_drive_frequency_ghz: 7.5,
            pulse_synthesis_dispatch_speed_m_per_s: 1700.0,
            cryogenic_temperature_mk: 10.0,
            microwave_probe_power_uw: 7.5,
            synthetic_hil_channels_factor: 4.0,
            testbed_pitch_um: 6.5,
        }
    }
}

impl CryoTestbedParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        cryo_drive_coupling_mev: f64,
        topological_calibration_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        pulse_synthesis_dispatch_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_probe_power_uw: f64,
        synthetic_hil_channels_factor: f64,
        testbed_pitch_um: f64,
    ) -> Self {
        Self {
            cryo_drive_coupling_mev: cryo_drive_coupling_mev.clamp(1.0, 35.0),
            topological_calibration_gap_mev: topological_calibration_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            pulse_synthesis_dispatch_speed_m_per_s: pulse_synthesis_dispatch_speed_m_per_s
                .clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_probe_power_uw: microwave_probe_power_uw.clamp(0.5, 30.0),
            synthetic_hil_channels_factor: synthetic_hil_channels_factor.clamp(1.0, 8.0),
            testbed_pitch_um: testbed_pitch_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for the Phonon Universal Multi-Scale Visual Studio
/// Full-Stack Hardware-in-the-Loop Cryogenic Dilution Refrigerator Testbed Integration
/// & Automated Qubit Calibration Engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CryoTestbedMetrics {
    /// Automated qubit calibration fidelity (target >= 0.9980).
    pub calibration_fidelity: f64,
    /// Qubit state retention fraction across closed-loop feedback (target >= 0.9970).
    pub qubit_state_retention_fraction: f64,
    /// Multi-physics topological protection gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-channel crosstalk isolation in decibels (target >= 55.0 dB).
    pub inter_channel_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
